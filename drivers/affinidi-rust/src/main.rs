//! Conformance driver for `affinidi-tsp` (affinidi-tdk-rs).
//!
//! A thin adapter from the driver protocol (docs/driver-protocol.md) onto the
//! crate's public API. Every message byte, digest and decision comes from the
//! library; where the API offers no way to express a request, the answer is
//! `unsupported`.

mod proto;

use affinidi_tsp::message::MessageType;
use affinidi_tsp::message::control::{ControlMessage, ControlType};
use affinidi_tsp::message::direct::{
    self, DecryptionKey, Padding, PackedMessage, UnpackedMessage, VerifyingKey,
};
use affinidi_tsp::message::envelope::Envelope;
use affinidi_tsp::message::meta::MetaEnvelope;
use affinidi_tsp::message::wire;
use affinidi_tsp::vid::{PrivateVid, ResolvedVid};
use affinidi_tsp::{RelationshipState, TspAgent, TspError};
use proto::*;
use serde_json::{Value, json};
use std::collections::HashMap;

const CAPABILITIES: &[&str] = &[
    "hpke-base",
    "signed-only",
    "sealed-box",
    "hpke-pq",
    "payload.scs",
    "payload.ctl",
    "payload.pad",
    "payload.rfi",
    "payload.rfa",
    "payload.rfd",
    "rfi.reply-path",
    "rfi.referral",
    "payload.hop",
    "padding",
    "peek",
    "endpoint",
];

/// Advertised only when the crate's `test-vectors` feature is built in.
fn capabilities() -> Vec<&'static str> {
    let mut caps = CAPABILITIES.to_vec();
    if cfg!(feature = "deterministic") {
        caps.push("deterministic");
    }
    caps
}

fn map_err(e: TspError) -> DErr {
    let message = e.to_string();
    let code = match &e {
        TspError::Cesr(_) | TspError::InvalidMessage(_) | TspError::NotTsp(_) => "malformed",
        TspError::Hpke(_) | TspError::SealedBox(_) => "decrypt",
        TspError::Verification(m) => {
            if m.contains("TSP_Digest") {
                "digest"
            } else if m.contains("ESSR") || m.contains("sender") {
                "sender"
            } else {
                "signature"
            }
        }
        TspError::VersionMismatch { .. } | TspError::RevisionMismatch { .. } => "version",
        TspError::Discarded(_) | TspError::Relationship(_) => "relationship",
        TspError::Vid(_) | TspError::VidNotFound(_) => "invalid-input",
        _ => "internal",
    };
    err(code, message)
}

// ---------------------------------------------------------------- pack

enum Scheme {
    HpkeBase,
    SealedBox,
    SignedOnly,
    HpkePq,
}

impl Scheme {
    fn parse(s: &str) -> DResult<Scheme> {
        Ok(match s {
            "hpke-base" => Scheme::HpkeBase,
            "sealed-box" => Scheme::SealedBox,
            "signed-only" => Scheme::SignedOnly,
            "hpke-pq" => Scheme::HpkePq,
            other => return Err(unsupported(format!("scheme {other}"))),
        })
    }
    /// The digest algorithm the library pairs with the scheme.
    fn digest_alg(&self) -> &'static str {
        match self {
            Scheme::SealedBox => "blake2b-256",
            _ => "sha2-256",
        }
    }
}

struct PackInput {
    body: Vec<u8>,
    kind: MessageType,
    hops: Vec<String>,
    referral_key: Option<[u8; 32]>,
    padding: Vec<u8>,
    returns_digest: bool,
    /// `payloadSender: null` was requested (deterministic packing only).
    #[cfg_attr(not(feature = "deterministic"), allow(dead_code))]
    null_sender: bool,
    /// A caller `pad` nonce (deterministic packing only).
    #[cfg_attr(not(feature = "deterministic"), allow(dead_code))]
    pad_nonce: Option<[u8; 16]>,
}

/// `deterministic` is whether the request goes through the crate's
/// `insecure_deterministic` packer, which can write a NULL ESSR sender and take
/// a `pad` nonce; the public packing functions can do neither.
fn build_pack_input(
    scheme: &Scheme,
    sender: &Identity,
    payload: &Value,
    deterministic: bool,
) -> DResult<PackInput> {
    // The public packing functions always write the ESSR sender field (direct.rs
    // `encode_sender_field`); only the deterministic packer can write NULL.
    let mut null_sender = false;
    if let Some(ps) = payload.get("payloadSender") {
        match ps {
            Value::Null if deterministic => null_sender = true,
            Value::Null => {
                return Err(unsupported(
                    "affinidi-tsp always carries the sender VID in the payload; NULL cannot be requested",
                ));
            }
            Value::String(s) if s != &sender.id => {
                return Err(unsupported("payloadSender other than the envelope sender"));
            }
            _ => {}
        }
    }
    let padding = opt_bytes(payload, "padding")?.unwrap_or_default();
    let ty = str_field(payload, "type")?;
    let mut input = PackInput {
        body: Vec::new(),
        kind: MessageType::Direct,
        hops: Vec::new(),
        referral_key: None,
        padding,
        returns_digest: false,
        null_sender,
        pad_nonce: None,
    };
    match ty {
        "scs" => input.body = bytes_field(payload, "data")?,
        "ctl" => {
            input.body = bytes_field(payload, "data")?;
            input.kind = MessageType::GenericControl;
        }
        "pad" => {
            if let Some(n) = opt_bytes(payload, "nonce")? {
                if !deterministic {
                    return Err(unsupported(
                        "pack_padding_message generates its own nonce; a caller nonce cannot be supplied",
                    ));
                }
                input.pad_nonce = Some(fixed(&n, "nonce")?);
            }
            input.kind = MessageType::PaddingOnly;
        }
        "rfi" => {
            let mut cm = ControlMessage::invite();
            if let Some(n) = opt_bytes(payload, "nonce")? {
                cm.nonce = Some(fixed(&n, "nonce")?);
            }
            if let Some(rp) = payload.get("replyPath").and_then(Value::as_array) {
                cm.route = rp
                    .iter()
                    .map(|v| v.as_str().map(str::to_string))
                    .collect::<Option<Vec<_>>>()
                    .ok_or_else(|| err("invalid-input", "replyPath must be strings"))?;
            }
            match payload.get("referral") {
                None | Some(Value::Null) => {}
                Some(r) => {
                    let vid = str_field(r, "vid")?.to_string();
                    let sk = opt_bytes(r, "skS")?.ok_or_else(|| {
                        unsupported("referral without skS: Signature_new must be made by the library")
                    })?;
                    let nonce = cm.nonce;
                    let route = cm.route.clone();
                    cm = ControlMessage::invite_referral(vid);
                    cm.nonce = nonce;
                    cm.route = route;
                    input.referral_key = Some(fixed(&sk, "referral skS")?);
                }
            }
            input.body = cm.encode();
            input.kind = MessageType::Control;
            input.returns_digest = true;
        }
        "rfa" => {
            let d = bytes_field(payload, "digest")?;
            input.body = ControlMessage::accept(fixed(&d, "digest")?).encode();
            input.kind = MessageType::Control;
            input.returns_digest = true;
        }
        "rfd" => {
            let d = bytes_field(payload, "digest")?;
            if let Some(alg) = payload.get("digestAlg").and_then(Value::as_str)
                && alg != scheme.digest_alg()
            {
                return Err(unsupported(format!(
                    "affinidi-tsp codes the digest by scheme ({}); {alg} cannot be requested",
                    scheme.digest_alg()
                )));
            }
            input.body = ControlMessage::cancel(fixed(&d, "digest")?).encode();
            input.kind = MessageType::Control;
        }
        "hop" => {
            input.body = bytes_field(payload, "inner")?;
            input.hops = payload
                .get("hops")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            input.kind = if input.hops.is_empty() {
                MessageType::Nested
            } else {
                MessageType::Routed
            };
        }
        other => return Err(unsupported(format!("payload type {other}"))),
    }
    Ok(input)
}

fn op_pack(req: &Value) -> DResult<Value> {
    let ephemeral = !matches!(req.get("ephemeral"), None | Some(Value::Null));
    let scheme = Scheme::parse(str_field(req, "scheme")?)?;
    let sender = Identity::field(req, "sender")?;
    let receiver = Identity::field(req, "receiver")?;
    let payload = req
        .get("payload")
        .ok_or_else(|| err("invalid-input", "missing payload"))?;

    // A signed-only message has no randomness, so the deterministic packer is
    // also the one that can honour its NULL payload sender without `ephemeral`.
    let signed_only_null_sender = matches!(scheme, Scheme::SignedOnly)
        && matches!(payload.get("payloadSender"), Some(Value::Null));
    if ephemeral || signed_only_null_sender {
        #[cfg(feature = "deterministic")]
        return deterministic::op_pack(req, &scheme, &sender, &receiver, payload);
        #[cfg(not(feature = "deterministic"))]
        if ephemeral {
            return Err(unsupported(
                "this build of affinidi-tsp has no ephemeral-key injection (feature test-vectors)",
            ));
        }
    }

    let input = build_pack_input(&scheme, &sender, payload, false)?;
    let s = sender.id.as_str();
    let r = receiver.id.as_str();

    let only_plain = |what: &str| -> DResult<()> {
        if !input.padding.is_empty() {
            return Err(unsupported(format!("{what}: no padding option")));
        }
        if input.referral_key.is_some() {
            return Err(unsupported(format!("{what}: no referral option")));
        }
        if input.kind == MessageType::Routed {
            return Err(unsupported(format!("{what}: no routed option")));
        }
        Ok(())
    };

    let packed: PackedMessage = match scheme {
        Scheme::HpkeBase => {
            if sender.is_pq() || receiver.is_pq() {
                return Err(unsupported("hpke-base with post-quantum key types"));
            }
            let sks: [u8; 32] = fixed(sender.sk_s()?, "sender skS")?;
            let pke: [u8; 32] = fixed(&receiver.pk_e, "receiver pkE")?;
            let padding = if input.padding.is_empty() {
                Padding::None
            } else {
                Padding::Exact(input.padding.clone())
            };
            if let Some(new_key) = input.referral_key {
                if !input.padding.is_empty() {
                    return Err(unsupported("referral invite with padding"));
                }
                let cm = ControlMessage::decode(&input.body).map_err(map_err)?;
                direct::pack_referral_invite(&cm, s, r, &sks, &new_key, &pke).map_err(map_err)?
            } else {
                match input.kind {
                    MessageType::Routed => {
                        if !input.padding.is_empty() {
                            return Err(unsupported("routed message with padding"));
                        }
                        affinidi_tsp::message::routed::pack_routed(
                            &input.body,
                            &input.hops,
                            s,
                            r,
                            &sks,
                            &pke,
                        )
                        .map_err(map_err)?
                    }
                    MessageType::PaddingOnly => {
                        direct::pack_padding_message(s, r, &sks, &pke, &padding).map_err(map_err)?
                    }
                    kind => direct::pack_padded(&input.body, kind, s, r, &sks, &pke, &padding)
                        .map_err(map_err)?,
                }
            }
        }
        Scheme::SealedBox => {
            only_plain("pack_sealed_box")?;
            let sks: [u8; 32] = fixed(sender.sk_s()?, "sender skS")?;
            let pke: [u8; 32] = fixed(&receiver.pk_e, "receiver pkE")?;
            direct::pack_sealed_box(&input.body, input.kind, s, r, &sks, &pke).map_err(map_err)?
        }
        Scheme::SignedOnly => {
            only_plain("pack_signed_only")?;
            if sender.sig_key_type != "Ed25519" {
                return Err(unsupported("pack_signed_only signs with Ed25519 only"));
            }
            let sks: [u8; 32] = fixed(sender.sk_s()?, "sender skS")?;
            direct::pack_signed_only(&input.body, input.kind, s, r, &sks).map_err(map_err)?
        }
        Scheme::HpkePq => {
            only_plain("pack_pq")?;
            if sender.sig_key_type != "MlDsa65" || receiver.enc_key_type != "MLKEM768-X25519" {
                return Err(unsupported("pack_pq needs an ML-DSA-65 sender and an MLKEM768-X25519 receiver"));
            }
            let sks: Box<[u8; affinidi_tsp::crypto::ml_dsa::SK_LEN]> = sender
                .sk_s()?
                .to_vec()
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err("invalid-input", "ML-DSA-65 skS must be 4032 bytes"))?;
            let pke: Box<[u8; affinidi_tsp::crypto::hpke_pq::PK_LEN]> = receiver
                .pk_e
                .clone()
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err("invalid-input", "MLKEM768-X25519 pkE must be 1216 bytes"))?;
            direct::pack_pq(&input.body, input.kind, s, r, &sks, &pke).map_err(map_err)?
        }
    };

    let (digest, alg) = if input.returns_digest {
        (json!(b64e(&packed.thread_digest)), json!(scheme.digest_alg()))
    } else {
        (Value::Null, Value::Null)
    };
    Ok(json!({"message": b64e(&packed.bytes), "digest": digest, "digestAlg": alg}))
}

// ---------------------------------------------------------------- deterministic pack

/// `pack` with the protocol's `ephemeral` field, through the crate's
/// `insecure_deterministic` packer (feature `test-vectors`). It exists to
/// reproduce the specification's vectors; the ephemeral material it takes is
/// public, so nothing packed here is confidential.
#[cfg(feature = "deterministic")]
mod deterministic {
    use super::*;
    use affinidi_tsp::message::direct::insecure_deterministic::{
        Options, PayloadSender, Protection, pack_insecure_deterministic,
    };

    pub fn op_pack(
        req: &Value,
        scheme: &Scheme,
        sender: &Identity,
        receiver: &Identity,
        payload: &Value,
    ) -> DResult<Value> {
        let ephemeral = req.get("ephemeral").filter(|v| !v.is_null());
        let key = |name: &str| -> DResult<Option<[u8; 32]>> {
            match ephemeral {
                None => Ok(None),
                Some(e) => opt_bytes(e, name)?
                    .map(|b| fixed(&b, &format!("ephemeral {name}")))
                    .transpose(),
            }
        };
        let (ikm_e, sk_em) = (key("ikmE")?, key("skEm")?);
        let protection = match (scheme, ikm_e, sk_em) {
            (Scheme::HpkeBase, Some(ikm_e), None) => Protection::HpkeBase { ikm_e },
            (Scheme::SealedBox, None, Some(ephemeral_secret)) => {
                Protection::SealedBox { ephemeral_secret }
            }
            (Scheme::SignedOnly, None, None) => Protection::SignedOnly,
            (Scheme::HpkePq, ..) => {
                return Err(unsupported("no deterministic packing for hpke-pq"));
            }
            _ => {
                return Err(err(
                    "invalid-input",
                    "ephemeral must be ikmE for hpke-base, skEm for sealed-box, null for signed-only",
                ));
            }
        };
        if sender.is_pq() || receiver.is_pq() {
            return Err(unsupported("deterministic packing with post-quantum key types"));
        }

        let input = build_pack_input(scheme, sender, payload, true)?;
        let sks: [u8; 32] = fixed(sender.sk_s()?, "sender skS")?;
        let pke: [u8; 32] = match scheme {
            Scheme::SignedOnly => [0u8; 32],
            _ => fixed(&receiver.pk_e, "receiver pkE")?,
        };
        let padding = if input.padding.is_empty() {
            Padding::None
        } else {
            Padding::Exact(input.padding.clone())
        };
        let packed = pack_insecure_deterministic(
            &input.body,
            input.kind,
            &sender.id,
            &receiver.id,
            &sks,
            &pke,
            protection,
            &Options {
                payload_sender: if input.null_sender {
                    PayloadSender::Null
                } else {
                    PayloadSender::Present
                },
                padding,
                hops: &input.hops,
                pad_nonce: input.pad_nonce,
                referral_signing_key: input.referral_key.as_ref(),
            },
        )
        .map_err(map_err)?;

        let (digest, alg) = if input.returns_digest {
            (json!(b64e(&packed.thread_digest)), json!(scheme.digest_alg()))
        } else {
            (Value::Null, Value::Null)
        };
        Ok(json!({"message": b64e(&packed.bytes), "digest": digest, "digestAlg": alg}))
    }
}

// ---------------------------------------------------------------- open
// ---------------------------------------------------------------- open

/// Which scheme sealed a message, read with the crate's own wire decoders.
fn scheme_of(bytes: &[u8], confidential: bool, receiver_pq: bool) -> DResult<&'static str> {
    if !confidential {
        return Ok("signed-only");
    }
    let decoded = Envelope::decode_full(bytes).map_err(map_err)?;
    let mut pos = decoded.header_len;
    if wire::decode_variable_data_range(wire::TSP_SEALED_BOX_CIPHERTEXT, bytes, &mut pos).is_some() {
        return Ok("sealed-box");
    }
    Ok(if receiver_pq { "hpke-pq" } else { "hpke-base" })
}

fn payload_json(u: &UnpackedMessage, alg: &str) -> DResult<Value> {
    Ok(match u.message_type {
        MessageType::Direct => json!({"type": "scs", "data": b64e(&u.payload)}),
        MessageType::GenericControl => json!({"type": "ctl", "data": b64e(&u.payload)}),
        // The padding message's nonce is decoded and discarded by the crate.
        MessageType::PaddingOnly => json!({"type": "pad"}),
        MessageType::Nested | MessageType::Routed => {
            json!({"type": "hop", "hops": u.hops, "inner": b64e(&u.payload)})
        }
        MessageType::Control => {
            let c = u
                .control
                .as_ref()
                .ok_or_else(|| err("internal", "control message without control payload"))?;
            match c.control_type {
                ControlType::RelationshipFormingInvite => {
                    let mut v = json!({
                        "type": "rfi",
                        "digest": b64e(&u.thread_digest),
                        "digestAlg": alg,
                        "replyPath": c.route,
                        "referral": c.referral.as_ref().map(|r| json!({"vid": r.new_vid, "signature": b64e(&r.signature)})),
                    });
                    if let Some(n) = c.nonce {
                        v["nonce"] = json!(b64e(&n));
                    }
                    v
                }
                ControlType::RelationshipFormingAccept => json!({
                    "type": "rfa",
                    "digest": c.reply.map(|d| b64e(&d)),
                    "replyDigest": b64e(&u.thread_digest),
                    "digestAlg": alg,
                }),
                ControlType::RelationshipCancel => json!({
                    "type": "rfd",
                    "digest": c.reply.map(|d| b64e(&d)),
                    "digestAlg": alg,
                }),
            }
        }
    })
}

fn op_open(req: &Value) -> DResult<Value> {
    let receiver = Identity::field(req, "receiver")?;
    let sender = Identity::field(req, "sender")?;
    let bytes = bytes_field(req, "message")?;

    let enc_sk = receiver.sk_e()?.to_vec();
    let enc32: [u8; 32];
    let dk = match receiver.enc_key_type.as_str() {
        "X25519" => {
            enc32 = fixed(&enc_sk, "receiver skE")?;
            DecryptionKey::X25519(&enc32)
        }
        "MLKEM768-X25519" => {
            enc32 = fixed(&enc_sk, "receiver skE")?;
            DecryptionKey::MlKem768X25519(&enc32)
        }
        other => return Err(unsupported(format!("encryption key type {other}"))),
    };
    let sig32: [u8; 32];
    let sigpq: [u8; affinidi_tsp::crypto::ml_dsa::PK_LEN];
    let vk = match sender.sig_key_type.as_str() {
        "Ed25519" => {
            sig32 = fixed(&sender.pk_s, "sender pkS")?;
            VerifyingKey::Ed25519(&sig32)
        }
        "MlDsa65" => {
            sigpq = fixed(&sender.pk_s, "sender pkS")?;
            VerifyingKey::MlDsa65(&sigpq)
        }
        other => return Err(unsupported(format!("signature key type {other}"))),
    };

    let u = direct::unpack_with(&bytes, dk, vk).map_err(map_err)?;
    let scheme = scheme_of(&bytes, u.confidential, receiver.enc_key_type != "X25519")?;
    let alg = if scheme == "sealed-box" { "blake2b-256" } else { "sha2-256" };
    let mut pos = 0;
    let (major, minor) = wire::decode_frame_version(&bytes, &mut pos).map_err(map_err)?;
    Ok(json!({
        "version": {"major": major, "minor": minor},
        "envelopeSender": u.sender,
        "envelopeReceiver": if u.receiver.is_empty() { Value::Null } else { json!(u.receiver) },
        "scheme": scheme,
        "payload": payload_json(&u, alg)?,
    }))
}

fn op_peek(req: &Value) -> DResult<Value> {
    let bytes = bytes_field(req, "message")?;
    let meta = MetaEnvelope::parse(&bytes).map_err(map_err)?;
    let mut pos = 0;
    let (major, minor) = wire::decode_frame_version(&bytes, &mut pos).map_err(map_err)?;
    let decoded = Envelope::decode_full(&bytes).map_err(map_err)?;
    let mut p1 = decoded.header_len;
    let mut p2 = decoded.header_len;
    let confidential =
        wire::decode_variable_data_range(wire::TSP_HPKE_BASE_CIPHERTEXT, &bytes, &mut p1).is_some()
            || wire::decode_variable_data_range(wire::TSP_SEALED_BOX_CIPHERTEXT, &bytes, &mut p2)
                .is_some();
    Ok(json!({
        "version": {"major": major, "minor": minor},
        "envelopeSender": meta.sender,
        "envelopeReceiver": if meta.receiver.is_empty() { Value::Null } else { json!(meta.receiver) },
        "confidential": confidential,
    }))
}

// ---------------------------------------------------------------- endpoint

struct Endpoint {
    agent: TspAgent,
    /// Driver bookkeeping only: the invite digest last reported by `receive`
    /// per (local, remote), so `endpoint.accept` can return the `digest` it
    /// echoes — `TspAgent` keeps it in a crate-private store.
    invite_digest: HashMap<(String, String), [u8; 32]>,
}

#[derive(Default)]
struct State {
    endpoints: HashMap<String, Endpoint>,
    next: u64,
}

fn endpoint<'a>(st: &'a mut State, req: &Value) -> DResult<&'a mut Endpoint> {
    let h = str_field(req, "endpoint")?;
    st.endpoints
        .get_mut(h)
        .ok_or_else(|| err("invalid-input", format!("unknown endpoint {h}")))
}

fn op_endpoint(st: &mut State, op: &str, req: &Value) -> DResult<Value> {
    match op {
        "endpoint.create" => {
            let agent = TspAgent::new();
            for v in req.get("identities").and_then(Value::as_array).into_iter().flatten() {
                let id = Identity::parse(v)?;
                if id.is_pq() {
                    return Err(unsupported("TspAgent VIDs are Ed25519/X25519 only"));
                }
                agent.add_private_vid(PrivateVid::from_keys(
                    id.id.clone(),
                    fixed(id.sk_s()?, "skS")?,
                    fixed(id.sk_e()?, "skE")?,
                ));
            }
            for v in req.get("peers").and_then(Value::as_array).into_iter().flatten() {
                let id = Identity::parse(v)?;
                if id.is_pq() {
                    return Err(unsupported("TspAgent VIDs are Ed25519/X25519 only"));
                }
                agent.add_verified_vid(ResolvedVid {
                    id: id.id.clone(),
                    signing_key: fixed(&id.pk_s, "pkS")?,
                    encryption_key: fixed(&id.pk_e, "pkE")?,
                    endpoints: vec![],
                    mediators: vec![],
                });
            }
            st.next += 1;
            let h = format!("ep-{}", st.next);
            st.endpoints.insert(
                h.clone(),
                Endpoint {
                    agent,
                    invite_digest: HashMap::new(),
                },
            );
            Ok(json!({"endpoint": h}))
        }
        "endpoint.invite" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let ep = endpoint(st, req)?;
            let p = ep.agent.send_relationship_invite(from, to).map_err(map_err)?;
            Ok(json!({"message": b64e(&p.bytes), "digest": b64e(&p.thread_digest)}))
        }
        "endpoint.accept" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let ep = endpoint(st, req)?;
            let p = ep.agent.send_relationship_accept(from, to).map_err(map_err)?;
            let invite = ep.invite_digest.get(&(from.to_string(), to.to_string()));
            Ok(json!({
                "message": b64e(&p.bytes),
                "digest": invite.map(|d| b64e(d)),
                "replyDigest": b64e(&p.thread_digest),
            }))
        }
        "endpoint.cancel" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let ep = endpoint(st, req)?;
            let p = ep.agent.send_relationship_cancel(from, to).map_err(map_err)?;
            Ok(json!({"message": b64e(&p.bytes)}))
        }
        "endpoint.send" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let data = bytes_field(req, "data")?;
            let ep = endpoint(st, req)?;
            let p = ep.agent.send(from, to, &data).map_err(map_err)?;
            Ok(json!({"message": b64e(&p.bytes)}))
        }
        "endpoint.receive" => {
            let bytes = bytes_field(req, "message")?;
            let meta = MetaEnvelope::parse(&bytes).map_err(map_err)?;
            let ep = endpoint(st, req)?;
            let m = ep.agent.receive(&meta.receiver, &bytes).map_err(map_err)?;
            let mut out = json!({"from": m.sender, "to": m.receiver});
            match m.control.as_ref().map(|c| c.control_type) {
                Some(ControlType::RelationshipFormingInvite) => {
                    out["event"] = json!("invite");
                    out["digest"] = json!(b64e(&m.thread_digest));
                    ep.invite_digest
                        .insert((m.receiver.clone(), m.sender.clone()), m.thread_digest);
                }
                Some(ControlType::RelationshipFormingAccept) => {
                    out["event"] = json!("accept");
                    out["digest"] = json!(m.control.as_ref().and_then(|c| c.reply).map(|d| b64e(&d)));
                    out["replyDigest"] = json!(b64e(&m.thread_digest));
                }
                Some(ControlType::RelationshipCancel) => {
                    out["event"] = json!("cancel");
                    out["digest"] = json!(m.control.as_ref().and_then(|c| c.reply).map(|d| b64e(&d)));
                }
                None => {
                    out["event"] = json!("message");
                    out["data"] = json!(b64e(&m.payload));
                }
            }
            Ok(out)
        }
        "endpoint.state" => {
            let (local, remote) = (str_field(req, "local")?, str_field(req, "remote")?);
            let ep = endpoint(st, req)?;
            let s = match ep.agent.relationship_state(local, remote) {
                RelationshipState::None => "none",
                RelationshipState::Pending => "invite-sent",
                RelationshipState::InviteReceived => "invite-received",
                RelationshipState::Bidirectional => "bidirectional",
            };
            Ok(json!({"state": s}))
        }
        other => Err(unsupported(format!("op {other}"))),
    }
}

fn main() {
    let mut st = State::default();
    proto::run(|op, req| match op {
        "hello" => Ok(json!({
            "name": "affinidi-tsp",
            "version": "0.2.0",
            "language": "rust",
            "protocol": 1,
            "capabilities": capabilities(),
        })),
        "pack" => op_pack(req),
        "open" => op_open(req),
        "peek" => op_peek(req),
        op if op.starts_with("endpoint.") => op_endpoint(&mut st, op, req),
        other => Err(unsupported(format!("op {other}"))),
    });
}
