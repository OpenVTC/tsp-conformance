//! Suite 1 — Appendix A vectors, per implementation.

use crate::cesr::{self, DecodedPayload};
use crate::compare::{Expect, SenderExpect};
use crate::ctx::{Ctx, payload_caps};
use crate::driver::{Dir, Reply};
use crate::model::{Ident, b64};
use crate::result::{Outcome, normalize};
use serde_json::{Value, json};

const SUITE: &str = "vectors";

fn scheme_of(name: &str) -> &'static str {
    if name.contains("sealed-box") {
        "sealed-box"
    } else if name.contains("signed-only") {
        "signed-only"
    } else if name.contains("-pq") {
        "hpke-pq"
    } else {
        "hpke-base"
    }
}

fn spec_of(name: &str) -> &'static str {
    match name {
        "direct-sealed-box" => "A; §8.3, §9.1, §9.2.8, §9.5",
        "direct-hpke-base" => "A; §8.2, §9.1, §9.2.8",
        "direct-signed-only" => "A; §3.5, §9.1",
        "control-rfi-direct" => "A; §7.2.1, §7.2.2, §9.4.1",
        "control-rfa-direct" => "A; §7.2.2, §9.4.2",
        "control-rfd" => "A; §7.3, §9.4.5",
        "control-rfi-sealed-box" => "A; §7.2.1, §8, §9.4.1",
        "nested-direct" => "A; §4, §9.4.15",
        "routed" => "A; §5.2, §5.3.3, §9.4.16",
        "direct-hpke-base-pq" => "A; §8.2, §8.3, §9.2.8",
        _ => "A",
    }
}

/// Expected payload JSON from the printed `payload` string.
fn expected_payload(p: &DecodedPayload) -> Value {
    let mut v = json!({"type": p.kind, "padding": b64(&p.padding), "payloadSender": p.payload_sender});
    if let Some(d) = &p.data {
        v["data"] = json!(b64(d));
    }
    if let Some(d) = &p.digest {
        match p.kind.as_str() {
            "rfa" => {
                v["digest"] = json!(b64(d));
                v["replyDigest"] = json!(b64(p.reply_digest.as_ref().unwrap()));
            }
            _ => v["digest"] = json!(b64(d)),
        }
        v["digestAlg"] = json!(p.digest_alg.unwrap());
    }
    if let Some(n) = &p.nonce {
        v["nonce"] = json!(b64(n));
    }
    if let Some(rp) = &p.reply_path {
        v["replyPath"] = json!(rp);
    }
    if let Some(r) = &p.referral_vid {
        v["referral"] = match r {
            None => Value::Null,
            Some(vid) => json!({"vid": vid}),
        };
    }
    if let Some(h) = &p.hops {
        v["hops"] = json!(h);
    }
    if let Some(t) = &p.inner_text {
        // the vector prints qb64, which is the base64url of the binary inner
        v["inner"] = json!(t);
    }
    v
}

pub fn run(ctx: &mut Ctx, i: usize) {
    let vectors: Vec<(String, Value)> = ctx.fixture["vectors"]
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    let impl_name = ctx.name(i);
    for (name, v) in vectors {
        let scheme = scheme_of(&name);
        let sender = ctx.fixture_ident(v["sender"].as_str().unwrap());
        let receiver = ctx.fixture_ident(v["receiver"].as_str().unwrap());
        let message = v["message"].as_str().unwrap().to_string();
        let decoded = match cesr::decode_payload(v["payload"].as_str().unwrap()) {
            Ok(d) => d,
            Err(e) => {
                ctx.record(SUITE, &format!("{name}/open"), spec_of(&name), "spec", &impl_name,
                    Outcome::error(format!("harness cannot decode the printed payload: {e}")));
                continue;
            }
        };
        let payload = expected_payload(&decoded);

        // ---- open and compare every field
        let case = format!("{name}/open");
        if ctx.selected(SUITE, &case) {
            let o = open_case(ctx, i, scheme, &sender, &receiver, &message, &payload, &decoded, &name);
            ctx.record(SUITE, &case, spec_of(&name), "spec", &impl_name, o);
        }

        // ---- inner message of nested / routed
        if let Some(inner) = &decoded.inner_text {
            let case = format!("{name}/inner-open");
            if ctx.selected(SUITE, &case) {
                let o = inner_case(ctx, i, inner, &v);
                ctx.record(SUITE, &case, spec_of(&name), "spec", &impl_name, o);
            }
        }

        // ---- byte-exact re-pack from the published ephemeral material
        let case = format!("{name}/pack-exact");
        if ctx.selected(SUITE, &case) {
            let o = pack_exact(ctx, i, scheme, &sender, &receiver, &message, &payload, &v);
            ctx.record(SUITE, &case, spec_of(&name), "spec", &impl_name, o);
        }

        // ---- what an intermediary sees without keys
        let case = format!("{name}/peek");
        if ctx.selected(SUITE, &case) {
            let o = peek_case(ctx, i, scheme, &sender, &receiver, &message);
            ctx.record(SUITE, &case, "§3.1, §5.1 (keyless envelope)", "spec", &impl_name, o);
        }
    }
    derived_signed_only(ctx, i);
}

fn peek_case(ctx: &mut Ctx, i: usize, scheme: &str, s: &Ident, r: &Ident, message: &str) -> Outcome {
    if let Some(o) = ctx.gate(i, &["peek".into()], Dir::Open) {
        return o;
    }
    let name = ctx.name(i);
    match ctx.call(i, "peek", json!({"message": message})) {
        Reply::Ok(v) => {
            let mut diffs = vec![];
            if v["envelopeSender"].as_str() != Some(&s.id) {
                diffs.push(format!("envelopeSender: expected {}, got {}", s.id, v["envelopeSender"]));
            }
            if v["envelopeReceiver"].as_str() != Some(&r.id) {
                diffs.push(format!("envelopeReceiver: expected {}, got {}", r.id, v["envelopeReceiver"]));
            }
            let conf = scheme != "signed-only";
            if v["confidential"].as_bool() != Some(conf) {
                diffs.push(format!("confidential: expected {conf}, got {}", v["confidential"]));
            }
            let mut o = if diffs.is_empty() {
                Outcome::pass()
            } else {
                Outcome::fail(diffs.join("\n"), format!("{name} peek reports a different envelope"))
            };
            if v.get("version").is_none() {
                o.unreported.push("peek.version".into());
            } else if v["version"] != json!({"major": 0, "minor": 2}) {
                o = Outcome::fail(format!("version: got {}", v["version"]), format!("{name} peek reports a different version"));
            }
            o
        }
        Reply::Err { code, message } if code == "unsupported" => Outcome::skip(format!("{name} peek: unsupported — {message}")),
        Reply::Err { code, message } => Outcome::fail(
            format!("[{code}] {message}"),
            format!("{name} cannot peek a vector: [{code}] {}", normalize(&message)),
        ),
        Reply::Broken(m) => Outcome::error(m),
    }
}

/// Messages derived from `direct-signed-only` by an edit that a conformant
/// receiver must *accept*, re-signed with alice's published key.
fn derived_signed_only(ctx: &mut Ctx, i: usize) {
    use crate::resign::{payload, payload_body, resign, var_field, with_payload};
    let impl_name = ctx.name(i);
    let v = ctx.fixture["vectors"]["direct-signed-only"].clone();
    let alice = ctx.fixture_ident("alice");
    let bob = ctx.fixture_ident("bob");
    let text = v["message"].as_str().unwrap().to_string();
    let decoded = cesr::decode_payload(v["payload"].as_str().unwrap()).unwrap();
    let base = expected_payload(&decoded);
    let map = cesr::map_message(&text).unwrap();

    let set_minor = |minor: &str| -> Option<String> {
        let mut t = text.clone();
        t.replace_range(map.version_start + 1..map.version_start + 4, minor);
        Some(t)
    };
    let essr_present = || -> Option<String> {
        let body = payload_body(&text, &map)?;
        let mut rd = cesr::Reader::new(body);
        rd.code4().ok()?;
        let sender = rd.var('B').ok()?;
        let new_body = format!("{}{}{}", &body[..4], var_field('B', alice.id.as_bytes()), &body[sender.end..]);
        Some(with_payload(&text, &map, &payload(&new_body)))
    };
    let cases: Vec<(&str, &str, Option<String>, (u64, u64), Option<String>)> = vec![
        ("minor-ABA-accepted", "§9.1 (MINOR does not gate; tsp_sdk 0.10 emitted YTSP-ABA)", set_minor("ABA"), (0, 64), None),
        ("minor-AAD-accepted", "§9.1 (a later MINOR is still processable)", set_minor("AAD"), (0, 3), None),
        ("essr-sender-present-accepted", "§3.7 step 7 (ESSR field MAY carry VID_sndr)", essr_present(), (0, 2), Some(alice.id.clone())),
    ];
    for (name, spec, derived, version, ps) in cases {
        let case = format!("direct-signed-only/resigned/{name}");
        if !ctx.selected(SUITE, &case) {
            continue;
        }
        let o = if let Some(o) = ctx.gate(i, &payload_caps("signed-only", &base, Dir::Open), Dir::Open) {
            o
        } else {
            match derived.and_then(|t| resign(&t, &alice)) {
                None => Outcome::error("harness could not derive the message"),
                Some(t) => {
                    let exp = Expect {
                        version,
                        sender: alice.id.clone(),
                        receiver: Some(bob.id.clone()),
                        scheme: "signed-only".into(),
                        payload: base.clone(),
                        payload_sender: SenderExpect::Exactly(ps),
                    };
                    match ctx.open_expect("a re-signed vector", i, &bob, &alice, &t, &exp, name) {
                        Ok((_, u)) => {
                            let mut o = Outcome::pass();
                            o.unreported = u;
                            o
                        }
                        Err(o) => o,
                    }
                }
            }
        };
        ctx.record(SUITE, &case, spec, "spec (re-signed)", &impl_name, o);
    }
}

#[allow(clippy::too_many_arguments)]
fn open_case(
    ctx: &mut Ctx,
    i: usize,
    scheme: &str,
    sender: &Ident,
    receiver: &Ident,
    message: &str,
    payload: &Value,
    decoded: &DecodedPayload,
    name: &str,
) -> Outcome {
    if let Some(o) = ctx.gate(i, &payload_caps(scheme, payload, Dir::Open), Dir::Open) {
        return o;
    }
    let exp = Expect {
        version: (0, 2),
        sender: sender.id.clone(),
        receiver: Some(receiver.id.clone()),
        scheme: scheme.into(),
        payload: payload.clone(),
        payload_sender: SenderExpect::Exactly(decoded.payload_sender.clone()),
    };
    match ctx.open_expect("the spec", i, receiver, sender, message, &exp, &format!("{name} vector")) {
        Ok((_, unreported)) => {
            let mut o = Outcome::pass();
            o.unreported = unreported;
            o
        }
        Err(o) => o,
    }
}

fn inner_case(ctx: &mut Ctx, i: usize, inner_text: &str, v: &Value) -> Outcome {
    let map = match cesr::map_message(inner_text) {
        Ok(m) => m,
        Err(e) => return Outcome::error(format!("harness cannot map inner message: {e}")),
    };
    // The inner parties are the nested pair, found by VID among the fixture's identifiers.
    let find = |ctx: &Ctx, vid: &str| -> Option<Ident> {
        ctx.fixture["identifiers"]
            .as_object()?
            .values()
            .find(|x| x["id"].as_str() == Some(vid))
            .map(Ident::from_fixture)
    };
    let (Some(s), Some(r)) = (find(ctx, &map.sender_vid), find(ctx, &map.receiver_vid)) else {
        return Outcome::skip("inner parties are not published identifiers");
    };
    let decoded = match cesr::decode_payload(v["innerPayload"].as_str().unwrap_or("")) {
        Ok(d) => d,
        Err(e) => return Outcome::error(format!("harness cannot decode innerPayload: {e}")),
    };
    let payload = expected_payload(&decoded);
    open_case(ctx, i, "hpke-base", &s, &r, inner_text, &payload, &decoded, "inner")
}

#[allow(clippy::too_many_arguments)]
fn pack_exact(
    ctx: &mut Ctx,
    i: usize,
    scheme: &str,
    sender: &Ident,
    receiver: &Ident,
    message: &str,
    payload: &Value,
    v: &Value,
) -> Outcome {
    let ephemeral = if let Some(ikm) = v.get("ikmE") {
        json!({"ikmE": ikm})
    } else if let Some(sk) = v.get("skEm") {
        json!({"skEm": sk})
    } else if scheme == "signed-only" {
        Value::Null
    } else {
        return Outcome::skip("the vector publishes no ephemeral material (spec: the hybrid KEM draws encapsulation randomness)");
    };
    // A signed-only message has no randomness at all (Ed25519 is deterministic),
    // so its bytes are reproducible by every packer without `deterministic`.
    let mut caps = payload_caps(scheme, payload, Dir::Pack);
    if !ephemeral.is_null() {
        if !ctx.drivers[i].caps.contains("deterministic") {
            return Outcome::skip(format!("{} lacks deterministic", ctx.name(i)));
        }
        caps.push("deterministic".into());
    }
    if let Some(o) = ctx.gate(i, &caps, Dir::Pack) {
        return o;
    }
    // Rebuild the pack input from the printed payload: the digests are computed
    // by the library, the nonce and padding are the vector's.
    let mut input = json!({"type": payload["type"], "padding": payload["padding"]});
    for k in ["data", "nonce", "replyPath", "hops", "inner"] {
        if let Some(x) = payload.get(k) {
            input[k] = x.clone();
        }
    }
    match payload["type"].as_str() {
        Some("rfa") | Some("rfd") => input["digest"] = payload["digest"].clone(),
        Some("rfi") => input["referral"] = Value::Null,
        _ => {}
    }
    if let Some(ps) = payload.get("payloadSender") {
        input["payloadSender"] = ps.clone();
    }
    let name = ctx.name(i);
    let reply = ctx.call(
        i,
        "pack",
        json!({"scheme": scheme, "sender": sender.private(), "receiver": receiver.public(),
               "payload": input, "ephemeral": ephemeral}),
    );
    match reply {
        Reply::Ok(r) => {
            let got = r["message"].as_str().unwrap_or("");
            if got == message {
                Outcome::pass()
            } else {
                let first = got
                    .bytes()
                    .zip(message.bytes())
                    .position(|(a, b)| a != b)
                    .unwrap_or(got.len().min(message.len()));
                Outcome::fail(
                    format!(
                        "bytes differ from the vector at qb64 offset {first}: expected …{}…, got …{}…",
                        &message[first..(first + 24).min(message.len())],
                        &got[first..(first + 24).min(got.len())]
                    ),
                    format!("{name} does not reproduce Appendix A bytes"),
                )
            }
        }
        Reply::Err { code, message } if code == "unsupported" => {
            Outcome::skip(format!("{name} pack: unsupported — {message}"))
        }
        Reply::Err { code, message } => Outcome::fail(
            format!("[{code}] {message}"),
            format!("{name} refuses a deterministic pack: [{code}] {}", normalize(&message)),
        ),
        Reply::Broken(m) => Outcome::error(m),
    }
}
