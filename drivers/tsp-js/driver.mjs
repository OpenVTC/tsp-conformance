#!/usr/bin/env node
// Conformance driver for @openvtc/vti-tsp-js (pnm-browser-plugin/packages/tsp-js).
//
// A thin adapter from the driver protocol (docs/driver-protocol.md) onto the
// package's built `dist`. Every message byte and digest comes from the library.
//
// Endpoint ops: tsp-js deliberately ships the relationship *rules* as pure
// functions (relationship.ts: transition, resolveInviteRace, resolveAccept,
// resolveCancel, admitsApplicationMessage, canSend) and leaves *storage* to the
// wallet. The driver therefore holds a Map as that storage and applies only
// those library functions to it; it adds no rule of its own. Builds without
// `resolveAccept` get no accept-to-invite correlation.
//
// Deterministic pack: tsp-js's test-only `unsafe-testing` subpath derives the
// HPKE-Base ephemeral from a caller `ikmE` and can write the NULL VID in the
// ESSR sender field, which is what reproducing Appendix A needs. It is loaded
// only if the build has it, so the driver still runs against older builds and
// simply does not advertise `deterministic`.

import { createInterface } from "node:readline";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const TSP_JS = process.env.TSP_JS_DIR ?? resolve(here, "../../../pnm-browser-plugin/packages/tsp-js");
const tsp = await import(pathToFileURL(resolve(TSP_JS, "dist/index.js")).href);
const { readFileSync } = await import("node:fs");
const pkg = JSON.parse(readFileSync(resolve(TSP_JS, "package.json"), "utf8"));
const unsafeTesting = await import(pathToFileURL(resolve(TSP_JS, "dist/unsafe-testing.js")).href).catch(
  () => null,
);
const DETERMINISTIC = typeof unsafeTesting?.__unsafeDeterministicPack === "function";

const CAPABILITIES = [
  "hpke-base",
  "payload.scs",
  // XCTL and XPAD decode, but there is no packer for them
  "payload.ctl:open",
  "payload.pad:open",
  "payload.rfi",
  "payload.rfa",
  "payload.rfd",
  "rfi.reply-path",
  // the Referral field is decode-only ("Invite only, and decode-only here")
  "rfi.referral:open",
  "payload.hop",
  "peek",
  "endpoint",
  // `pack` honours `ephemeral.ikmE` (HPKE-Base) and caller nonces
  ...(DETERMINISTIC ? ["deterministic"] : []),
];

class DriverError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}
const unsupported = (m) => new DriverError("unsupported", m);

const b64d = (s) => new Uint8Array(Buffer.from(s, "base64url"));
const b64e = (b) => Buffer.from(b).toString("base64url");
const eq = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);

function classify(e) {
  if (e instanceof DriverError) return { code: e.code, message: e.message };
  const message = e?.message ?? String(e);
  let code = "internal";
  if (e?.code === "E_TSP_TRANSITION") code = "relationship";
  else if (e?.code === "E_TSP_REVISION") code = /MAJOR/.test(message) ? "version" : "malformed";
  else if (/does not support|this implementation does not/.test(message)) code = "unsupported";
  else if (/signature verification failed|signature names key index/.test(message)) code = "signature";
  else if (/TSP_Digest/.test(message)) code = "digest";
  else if (/ESSR sender/.test(message)) code = "sender";
  else if (/invalid tag|tag|decrypt|aead/i.test(message)) code = "decrypt";
  else if (/^tsp:/.test(message)) code = "malformed";
  return { code, message };
}

function identity(v, what) {
  if (!v || typeof v.id !== "string") throw new DriverError("invalid-input", `missing identity ${what}`);
  const sigKeyType = v.sigKeyType ?? "Ed25519";
  const encKeyType = v.encKeyType ?? "X25519";
  if (sigKeyType !== "Ed25519" || encKeyType !== "X25519") {
    throw unsupported(`tsp-js supports Ed25519/X25519 identities only (got ${sigKeyType}/${encKeyType})`);
  }
  return {
    id: v.id,
    pkS: b64d(v.pkS),
    pkE: b64d(v.pkE),
    skS: v.skS ? b64d(v.skS) : undefined,
    skE: v.skE ? b64d(v.skE) : undefined,
  };
}

// ------------------------------------------------------------------ pack

/**
 * The packers for one request: the library's own, or — when the request pins
 * the ephemeral — the `unsafe-testing` ones with that material bound in. Same
 * call shapes either way, so the switch below does not branch on it.
 */
function packersFor(req, p, sender) {
  const nullSender = "payloadSender" in p && p.payloadSender === null;
  if ("payloadSender" in p && p.payloadSender !== null && p.payloadSender !== sender.id) {
    throw unsupported("tsp-js writes the envelope sender's VID (or NULL) in the ESSR field");
  }
  if (req.ephemeral == null) {
    if (nullSender) {
      throw unsupported("tsp-js writes the sender VID in the ESSR field; NULL only with a pinned ephemeral");
    }
    return {
      pack: tsp.pack,
      packInvite: tsp.packInvite,
      packAccept: tsp.packAccept,
      packCancel: tsp.packCancel,
      packNested: tsp.packNested,
      packRouted: tsp.packRouted,
    };
  }
  if (!DETERMINISTIC) throw unsupported("this tsp-js build has no deterministic (unsafe-testing) packer");
  if (req.ephemeral.ikmE == null) {
    throw unsupported("tsp-js pins an HPKE-Base ikmE only (no sealed box, so no skEm)");
  }
  const u = unsafeTesting;
  const det = { __unsafeIkmE: b64d(req.ephemeral.ikmE), nullPayloadSender: nullSender };
  return {
    pack: (body, s, r, k) => u.__unsafeDeterministicPack(body, s, r, k, det),
    packInvite: (s, r, k, opts) => {
      if (!opts.nonce) throw new DriverError("invalid-input", "a deterministic rfi needs a caller nonce");
      return u.__unsafeDeterministicPackInvite(s, r, k, opts, det);
    },
    packAccept: (d, s, r, k) => u.__unsafeDeterministicPackAccept(d, s, r, k, det),
    packCancel: (d, s, r, k) => u.__unsafeDeterministicPackCancel(d, s, r, k, det),
    packNested: (inner, s, r, k) => u.__unsafeDeterministicPackNested(inner, s, r, k, det),
    packRouted: (inner, hops, s, r, k) => u.__unsafeDeterministicPackRouted(inner, hops, s, r, k, det),
  };
}

async function opPack(req) {
  if (req.scheme !== "hpke-base") throw unsupported(`scheme ${req.scheme}: tsp-js packs HPKE-Base only`);
  const sender = identity(req.sender, "sender");
  const receiver = identity(req.receiver, "receiver");
  const p = req.payload ?? {};
  if (p.padding && p.padding.length > 0) throw unsupported("tsp-js always writes an empty padding field");
  const packers = packersFor(req, p, sender);
  const keys = { senderSigningKey: sender.skS, receiverEncryptionKey: receiver.pkE };
  let packed;
  let returnsDigest = false;
  switch (p.type) {
    case "scs":
      packed = await packers.pack(b64d(p.data), sender.id, receiver.id, keys);
      break;
    case "rfi": {
      if (p.referral != null) throw unsupported("tsp-js cannot pack a referral (decode-only)");
      const opts = { route: p.replyPath ?? [] };
      if (p.nonce) opts.nonce = b64d(p.nonce);
      packed = await packers.packInvite(sender.id, receiver.id, keys, opts);
      returnsDigest = true;
      break;
    }
    case "rfa":
      packed = await packers.packAccept(b64d(p.digest), sender.id, receiver.id, keys);
      returnsDigest = true;
      break;
    case "rfd":
      if (p.digestAlg && p.digestAlg !== "sha2-256") throw unsupported("tsp-js digests are SHA2-256 only");
      packed = await packers.packCancel(b64d(p.digest), sender.id, receiver.id, keys);
      break;
    case "hop": {
      const hops = p.hops ?? [];
      const inner = b64d(p.inner);
      packed = hops.length === 0
        ? await packers.packNested(inner, sender.id, receiver.id, keys)
        : await packers.packRouted(inner, hops, sender.id, receiver.id, keys);
      break;
    }
    default:
      throw unsupported(`payload type ${p.type}: no packer in tsp-js`);
  }
  return {
    message: b64e(packed.bytes),
    digest: returnsDigest ? b64e(packed.threadDigest) : null,
    digestAlg: returnsDigest ? "sha2-256" : null,
  };
}

// ------------------------------------------------------------------ open

function payloadJson(u) {
  const c = u.control;
  if (c) {
    if (c.controlType === "invite") {
      return {
        type: "rfi",
        digest: b64e(c.digest),
        digestAlg: "sha2-256",
        nonce: b64e(c.nonce),
        replyPath: c.route,
        referral: c.referral ? { vid: c.referral.newVid, signature: b64e(c.referral.signature) } : null,
      };
    }
    if (c.controlType === "accept") {
      return { type: "rfa", digest: b64e(c.inReplyTo), replyDigest: b64e(c.digest), digestAlg: "sha2-256" };
    }
    return { type: "rfd", digest: b64e(c.inReplyTo), digestAlg: "sha2-256" };
  }
  switch (u.messageType) {
    case "direct":
      return { type: "scs", data: b64e(u.payload) };
    case "control": // XCTL: carried opaquely
      return { type: "ctl", data: b64e(u.payload) };
    case "padding":
      return { type: "pad" };
    case "nested":
    case "routed":
      return { type: "hop", hops: u.hops, inner: b64e(u.payload) };
    default:
      throw new DriverError("internal", `unmapped message type ${u.messageType}`);
  }
}

async function opOpen(req) {
  const receiver = identity(req.receiver, "receiver");
  const sender = identity(req.sender, "sender");
  const bytes = b64d(req.message);
  const u = await tsp.unpack(bytes, {
    receiverDecryptionKey: receiver.skE,
    senderSigningKey: sender.pkS,
  });
  const peeked = tsp.peekRevision(bytes);
  return {
    version: { major: peeked.major, minor: peeked.minor },
    envelopeSender: u.sender,
    envelopeReceiver: u.receiver === "" ? null : u.receiver,
    scheme: "hpke-base",
    payload: payloadJson(u),
  };
}

function opPeek(req) {
  const bytes = b64d(req.message);
  const peeked = tsp.peekRevision(bytes);
  const d = tsp.decodeEnvelope(bytes);
  const at = () => ({ pos: d.headerLen });
  const confidential =
    tsp.cesr.decodeVariableDataRange(tsp.cesr.TSP_HPKE_BASE_CIPHERTEXT, bytes, at()) !== undefined ||
    tsp.cesr.decodeVariableDataRange(tsp.cesr.TSP_SEALED_BOX_CIPHERTEXT, bytes, at()) !== undefined;
  return {
    version: { major: peeked.major, minor: peeked.minor },
    envelopeSender: d.envelope.sender,
    envelopeReceiver: d.envelope.receiver === "" ? null : d.envelope.receiver,
    confidential,
  };
}

// ------------------------------------------------------------------ endpoint

const endpoints = new Map();
let nextEndpoint = 0;

function getEndpoint(req) {
  const ep = endpoints.get(req.endpoint);
  if (!ep) throw new DriverError("invalid-input", `unknown endpoint ${req.endpoint}`);
  return ep;
}

function rel(ep, local, remote) {
  const k = `${local} ${remote}`;
  if (!ep.rels.has(k)) ep.rels.set(k, { state: "none" });
  return ep.rels.get(k);
}

function keysFor(ep, from, to) {
  const me = ep.identities.get(from);
  const peer = ep.peers.get(to);
  if (!me || !peer) throw new DriverError("invalid-input", `unknown VID pair ${from} -> ${to}`);
  return { senderSigningKey: me.skS, receiverEncryptionKey: peer.pkE };
}

const STATE_NAMES = {
  none: "none",
  pending: "invite-sent",
  inviteReceived: "invite-received",
  bidirectional: "bidirectional",
};

async function opEndpoint(op, req) {
  if (op === "endpoint.create") {
    const ep = { identities: new Map(), peers: new Map(), rels: new Map() };
    for (const v of req.identities ?? []) {
      const id = identity(v, "identity");
      ep.identities.set(id.id, id);
    }
    for (const v of req.peers ?? []) {
      const id = identity(v, "peer");
      ep.peers.set(id.id, id);
    }
    const h = `ep-${++nextEndpoint}`;
    endpoints.set(h, ep);
    return { endpoint: h };
  }
  const ep = getEndpoint(req);
  switch (op) {
    case "endpoint.invite": {
      const r = rel(ep, req.from, req.to);
      const next = tsp.transition(r.state, "sendInvite");
      const packed = await tsp.packInvite(req.from, req.to, keysFor(ep, req.from, req.to));
      Object.assign(r, { state: next, inviteDigest: packed.threadDigest, replyDigest: undefined });
      return { message: b64e(packed.bytes), digest: b64e(packed.threadDigest) };
    }
    case "endpoint.accept": {
      const r = rel(ep, req.from, req.to);
      const next = tsp.transition(r.state, "sendAccept");
      const packed = await tsp.packAccept(r.inviteDigest, req.from, req.to, keysFor(ep, req.from, req.to));
      Object.assign(r, { state: next, replyDigest: packed.threadDigest });
      return { message: b64e(packed.bytes), digest: b64e(r.inviteDigest), replyDigest: b64e(packed.threadDigest) };
    }
    case "endpoint.cancel": {
      const r = rel(ep, req.from, req.to);
      const next = tsp.transition(r.state, "sendCancel");
      const named = r.inviteDigest ?? r.replyDigest;
      if (!named) throw new DriverError("relationship", "no digest on record to name");
      const packed = await tsp.packCancel(named, req.from, req.to, keysFor(ep, req.from, req.to));
      ep.rels.set(`${req.from} ${req.to}`, { state: next });
      return { message: b64e(packed.bytes) };
    }
    case "endpoint.send": {
      const r = rel(ep, req.from, req.to);
      if (!tsp.canSend(r.state)) throw new DriverError("relationship", `cannot send in state ${r.state}`);
      const packed = await tsp.pack(b64d(req.data), req.from, req.to, keysFor(ep, req.from, req.to));
      return { message: b64e(packed.bytes) };
    }
    case "endpoint.receive": {
      const bytes = b64d(req.message);
      const env = tsp.decodeEnvelope(bytes).envelope;
      const me = ep.identities.get(env.receiver);
      const peer = ep.peers.get(env.sender);
      if (!me) throw new DriverError("receiver", `not addressed to a local VID: ${env.receiver}`);
      if (!peer) throw new DriverError("sender", `unknown sender ${env.sender}`);
      const u = await tsp.unpack(bytes, { receiverDecryptionKey: me.skE, senderSigningKey: peer.pkS });
      const r = rel(ep, env.receiver, env.sender);
      const base = { from: u.sender, to: u.receiver };
      const c = u.control;
      if (c?.controlType === "invite") {
        if (r.state === "pending") {
          const outcome = tsp.resolveInviteRace(r.inviteDigest, c.digest);
          if (outcome.keep === "ours") throw new DriverError("relationship", outcome.reason);
          Object.assign(r, { state: "inviteReceived", inviteDigest: c.digest });
        } else {
          Object.assign(r, { state: tsp.transition(r.state, "receiveInvite"), inviteDigest: c.digest });
        }
        return { ...base, event: "invite", digest: b64e(c.digest) };
      }
      if (c?.controlType === "accept") {
        // `resolveAccept` (added with tswg-tsp-specification conformance work)
        // correlates the accept with our invite. Older builds lack it and
        // apply the transition unchecked, which the suite reports as a finding.
        if (tsp.resolveAccept) {
          const outcome = tsp.resolveAccept(r.state, c.inReplyTo, r.inviteDigest);
          if (outcome.action === "ignore") throw new DriverError("relationship", outcome.reason);
        }
        Object.assign(r, { state: tsp.transition(r.state, "receiveAccept"), replyDigest: c.digest });
        return { ...base, event: "accept", digest: b64e(c.inReplyTo), replyDigest: b64e(c.digest) };
      }
      if (c?.controlType === "cancel") {
        const known = [r.inviteDigest, r.replyDigest].filter(Boolean);
        const outcome = tsp.resolveCancel(r.state, c.inReplyTo, known);
        if (outcome.action === "ignore") throw new DriverError("relationship", outcome.reason);
        ep.rels.set(`${env.receiver} ${env.sender}`, { state: tsp.transition(r.state, "receiveCancel") });
        return { ...base, event: "cancel", digest: b64e(c.inReplyTo) };
      }
      if (!tsp.admitsApplicationMessage(r.state)) {
        throw new DriverError("relationship", `application message in state ${r.state}`);
      }
      return { ...base, event: "message", data: b64e(u.payload) };
    }
    case "endpoint.state": {
      const r = rel(ep, req.local, req.remote);
      const out = { state: STATE_NAMES[r.state] };
      if (r.inviteDigest) out.digest = b64e(r.inviteDigest);
      if (r.replyDigest) out.replyDigest = b64e(r.replyDigest);
      return out;
    }
    default:
      throw unsupported(`op ${op}`);
  }
}

// ------------------------------------------------------------------ loop

async function handle(req) {
  switch (req.op) {
    case "hello":
      return {
        name: pkg.name,
        version: pkg.version,
        language: "typescript",
        protocol: 1,
        capabilities: CAPABILITIES,
      };
    case "pack":
      return opPack(req);
    case "open":
      return opOpen(req);
    case "peek":
      return opPeek(req);
    default:
      if (typeof req.op === "string" && req.op.startsWith("endpoint.")) return opEndpoint(req.op, req);
      throw unsupported(`op ${req.op}`);
  }
}

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of rl) {
  if (!line.trim()) continue;
  let req;
  let response;
  try {
    req = JSON.parse(line);
    response = { id: req.id, ok: true, result: await handle(req) };
  } catch (e) {
    const error = classify(e);
    if (error.code === "internal") console.error(e?.stack ?? e);
    response = { id: req?.id ?? null, ok: false, error };
  }
  process.stdout.write(JSON.stringify(response) + "\n");
}
