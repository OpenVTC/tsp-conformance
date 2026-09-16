//! Suite 3 — negative: a corrupted message must be refused.

use crate::cesr::{self, MessageMap};
use crate::ctx::Ctx;
use crate::driver::{Dir, Reply};
use crate::model::{Ident, b64};
use crate::result::Outcome;
use serde_json::json;

const SUITE: &str = "negative";

struct Source {
    label: String,
    scheme: &'static str,
    kind: &'static str,
    sender: Ident,
    receiver: Ident,
    text: String,
}

struct Mutation {
    name: &'static str,
    spec: &'static str,
    /// Codes a precise implementation would give; any other `ok:false` passes
    /// with a warning.
    expect: &'static [&'static str],
    apply: fn(&mut Ctx, &Source, &MessageMap) -> Option<Mutated>,
}

struct Mutated {
    text: String,
    receiver: Ident,
    sender: Ident,
}

fn same(s: &Source, text: String) -> Option<Mutated> {
    Some(Mutated {
        text,
        receiver: s.receiver.clone(),
        sender: s.sender.clone(),
    })
}

fn mid(start: usize, end: usize) -> usize {
    start + ((end - start) / 2) / 4 * 4 + 1
}

fn with_frame_count(s: &Source, m: &MessageMap, delta: i64) -> Option<Mutated> {
    let code = &s.text[m.frame_code.0..m.frame_code.1];
    let q = m.frame_quadlets as i64 + delta;
    if q < 0 {
        return None;
    }
    let new_code = if code.starts_with("--") {
        format!("--E{}", cesr::b64_chars(q as u64, 5))
    } else {
        if q > 4095 {
            return None;
        }
        format!("-E{}", cesr::b64_chars(q as u64, 2))
    };
    same(s, format!("{new_code}{}", &s.text[m.frame_code.1..]))
}

fn mutations() -> Vec<Mutation> {
    vec![
        Mutation {
            name: "flip-ciphertext",
            spec: "§3.7 step 6, §8.2.2",
            expect: &["decrypt", "signature"],
            apply: |_, s, m| {
                let ct = m.ciphertext.as_ref()?;
                same(s, cesr::flip_char(&s.text, mid(ct.data_start, ct.end)))
            },
        },
        Mutation {
            name: "flip-signature",
            spec: "§3.4, §3.7 step 5, §9.5",
            expect: &["signature"],
            apply: |_, s, _| same(s, cesr::flip_char(&s.text, s.text.len() - 9)),
        },
        Mutation {
            name: "flip-envelope-sender",
            spec: "§3.7 steps 3–5, §8.2.2 aad",
            expect: &["signature", "sender", "decrypt"],
            apply: |_, s, m| same(s, cesr::flip_char(&s.text, mid(m.sender.data_start, m.sender.end))),
        },
        Mutation {
            name: "flip-version-minor",
            spec: "§9.1 (MINOR is covered by the signature and aad)",
            expect: &["signature", "decrypt", "version"],
            apply: |_, s, m| same(s, cesr::flip_char(&s.text, m.version_start + 3)),
        },
        Mutation {
            name: "unknown-major-version",
            spec: "§9.1 MAJOR gates processability",
            expect: &["version"],
            apply: |_, s, m| {
                let mut t = s.text.clone().into_bytes();
                t[m.version_start + 1] = b'B';
                same(s, String::from_utf8(t).unwrap())
            },
        },
        Mutation {
            name: "truncate-one-triplet",
            spec: "§9.1 -E count, §9.5",
            expect: &["malformed"],
            apply: |_, s, _| same(s, s.text[..s.text.len() - 4].to_string()),
        },
        Mutation {
            name: "truncate-half",
            spec: "§9.1",
            expect: &["malformed"],
            apply: |_, s, _| same(s, s.text[..s.text.len() / 8 * 4].to_string()),
        },
        Mutation {
            name: "trailing-bytes",
            spec: "§9.1, §9.5 (the attachment ends the message)",
            expect: &["malformed"],
            apply: |_, s, _| same(s, format!("{}AAAA", s.text)),
        },
        Mutation {
            name: "frame-count-too-large",
            spec: "§9.1 (-E count must match the signable content)",
            expect: &["malformed"],
            apply: |_, s, m| with_frame_count(s, m, 1),
        },
        Mutation {
            name: "frame-count-too-small",
            spec: "§9.1",
            expect: &["malformed"],
            apply: |_, s, m| with_frame_count(s, m, -1),
        },
        Mutation {
            name: "signature-index-nonzero",
            spec: "§9.5.1 (B# names a key index the VID does not have; the index is not signed)",
            expect: &["signature", "malformed"],
            apply: |_, s, m| {
                // -C## -K## B<index> <sig>: the index character is not covered by the signature
                let at = m.content_end + 9;
                if s.text.get(m.content_end + 8..at) != Some("B") {
                    return None;
                }
                same(s, cesr::flip_char(&s.text, at))
            },
        },
        Mutation {
            name: "attachment-count-too-large",
            spec: "§9.5 (-C## counts the attachment)",
            expect: &["malformed", "signature"],
            apply: |_, s, m| {
                let at = m.content_end;
                let q = cesr::b64_value(s.text.get(at + 2..at + 4)?)?;
                let mut t = s.text.clone();
                t.replace_range(at + 2..at + 4, &cesr::b64_chars(q + 1, 2));
                same(s, t)
            },
        },
        Mutation {
            name: "wrong-receiver-key",
            spec: "§3.7 step 6, §8.2.2",
            expect: &["decrypt", "receiver"],
            apply: |ctx, s, m| {
                m.ciphertext.as_ref()?;
                if s.receiver.enc_key_type != "X25519" {
                    return None;
                }
                let other = ctx.keys.ident("impostor").renamed(s.receiver.id.clone());
                Some(Mutated {
                    text: s.text.clone(),
                    receiver: other,
                    sender: s.sender.clone(),
                })
            },
        },
        Mutation {
            name: "wrong-sender-key",
            spec: "§3.7 step 5",
            expect: &["signature", "sender"],
            apply: |ctx, s, _| {
                if s.sender.sig_key_type != "Ed25519" {
                    return None;
                }
                let other = ctx.keys.ident("impostor").renamed(s.sender.id.clone());
                Some(Mutated {
                    text: s.text.clone(),
                    receiver: s.receiver.clone(),
                    sender: other,
                })
            },
        },
    ]
}

fn sources(ctx: &mut Ctx, r: usize) -> Vec<Result<Source, (String, Outcome)>> {
    let mut out = vec![];
    // the implementation's own HPKE-Base output
    let a = ctx.keys.ident("alice");
    let b = ctx.keys.ident("bob");
    let caps = vec!["hpke-base".to_string(), "payload.scs".to_string()];
    let label = "self/hpke-base".to_string();
    match ctx.gate(r, &caps, Dir::Pack) {
        Some(o) => out.push(Err((label, o))),
        None => match ctx.pack(r, "hpke-base", &a, &b, &json!({"type": "scs", "data": b64(b"negative")})) {
            Ok(p) => out.push(Ok(Source {
                label,
                scheme: "hpke-base",
                kind: "scs",
                sender: a,
                receiver: b,
                text: p.message,
            })),
            Err(o) => out.push(Err((label, o))),
        },
    }
    for (name, scheme, kind) in [
        ("direct-hpke-base", "hpke-base", "scs"),
        ("direct-signed-only", "signed-only", "scs"),
        ("direct-sealed-box", "sealed-box", "scs"),
        ("control-rfi-direct", "hpke-base", "rfi"),
        ("direct-hpke-base-pq", "hpke-pq", "scs"),
    ] {
        let v = ctx.fixture["vectors"][name].clone();
        out.push(Ok(Source {
            label: format!("vector/{name}"),
            scheme,
            kind,
            sender: ctx.fixture_ident(v["sender"].as_str().unwrap()),
            receiver: ctx.fixture_ident(v["receiver"].as_str().unwrap()),
            text: v["message"].as_str().unwrap().to_string(),
        }));
    }
    out
}

pub fn run(ctx: &mut Ctx, r: usize) {
    let rname = ctx.name(r);
    for src in sources(ctx, r) {
        let src = match src {
            Ok(s) => s,
            Err((label, o)) => {
                if ctx.selected(SUITE, &label) {
                    let o = match o.status {
                        crate::result::Status::Skip => o,
                        _ => Outcome::skip(format!("source unavailable: {}", o.detail)),
                    };
                    ctx.record(SUITE, &format!("{label}/*"), "", &label, &rname, o);
                }
                continue;
            }
        };
        let caps = vec![src.scheme.to_string(), format!("payload.{}", src.kind)];
        let gate = ctx.gate(r, &caps, Dir::Open);
        let map = cesr::map_message(&src.text);
        // Baseline: the unmodified message must open, or corruption proves nothing.
        let baseline: Result<(), Outcome> = match (&gate, &map) {
            (Some(o), _) => Err(o.clone()),
            (_, Err(e)) => Err(Outcome::error(format!("harness cannot map source: {e}"))),
            _ => match ctx.open_raw(r, &src.receiver, &src.sender, &src.text) {
                Reply::Ok(_) => Ok(()),
                Reply::Err { code, message } if code == "unsupported" => {
                    Err(Outcome::skip(format!("unsupported: {message}")))
                }
                other => Err(Outcome::skip(format!(
                    "baseline does not open ({}), so corruption proves nothing",
                    other.describe()
                ))),
            },
        };
        for m in mutations() {
            let case = format!("{}/{}", src.label, m.name);
            if !ctx.selected(SUITE, &case) {
                continue;
            }
            let o = match (&baseline, &map) {
                (Err(o), _) => o.clone(),
                (Ok(()), Ok(map)) => match (m.apply)(ctx, &src, map) {
                    None => Outcome::skip("mutation not applicable to this source"),
                    Some(mt) => judge(ctx, r, &mt, m.expect, m.name, &src.label),
                },
                (Ok(()), Err(e)) => Outcome::error(e.clone()),
            };
            ctx.record(SUITE, &case, m.spec, &src.label, &rname, o);
        }
    }
    digest_resigned(ctx, r);
    resigned_signed_only(ctx, r);
}

fn judge(ctx: &mut Ctx, r: usize, mt: &Mutated, expect: &[&str], what: &str, source: &str) -> Outcome {
    let rname = ctx.name(r);
    match ctx.open_raw(r, &mt.receiver, &mt.sender, &mt.text) {
        Reply::Ok(v) => Outcome::fail(
            format!(
                "accepted a message with {what} (source {source}); decoded payload type {}",
                v["payload"]["type"]
            ),
            format!("{rname} accepts a message with {what}"),
        ),
        Reply::Err { code, message } => {
            let mut o = Outcome::pass();
            o.detail = format!("refused: [{code}] {message}");
            if !expect.contains(&code.as_str()) {
                o.warnings.push(format!(
                    "refused with [{code}] ({message}); expected one of {expect:?}"
                ));
            }
            o
        }
        Reply::Broken(m) => Outcome::error(m),
    }
}

/// A signed-only invite whose digest is corrupted and the message re-signed
/// with the sender's key, so that only the SAID check can catch it.
fn digest_resigned(ctx: &mut Ctx, r: usize) {
    let case = "resigned/signed-only-rfi/digest-tampered";
    if !ctx.selected(SUITE, case) {
        return;
    }
    let spec = "§7.2.1 (receiver recomputes the SAID)";
    let rname = ctx.name(r);
    let caps = vec!["signed-only".to_string(), "payload.rfi".to_string()];
    if let Some(o) = ctx.gate(r, &caps, Dir::Open) {
        ctx.record(SUITE, case, spec, "-", &rname, o);
        return;
    }
    let Some(packer) = (0..ctx.drivers.len()).find(|&i| ctx.gate(i, &caps, Dir::Pack).is_none()) else {
        ctx.record(SUITE, case, spec, "-", &rname, Outcome::skip("no driver can pack a signed-only rfi"));
        return;
    };
    let pname = ctx.name(packer);
    let a = ctx.keys.ident("alice");
    let b = ctx.keys.ident("bob");
    let input = json!({"type": "rfi", "replyPath": [], "referral": null});
    let packed = match ctx.pack(packer, "signed-only", &a, &b, &input) {
        Ok(p) => p,
        Err(o) => {
            ctx.record(SUITE, case, spec, &pname, &rname, Outcome::skip(format!("source unavailable: {}", o.detail)));
            return;
        }
    };
    let resign = |text: &str| crate::resign::resign(text, &a);
    // Control: re-signing an untouched message must still open, which proves
    // the tool rather than the implementation.
    let Some(control) = resign(&packed.message) else {
        ctx.record(SUITE, case, spec, &pname, &rname, Outcome::error("harness could not re-sign"));
        return;
    };
    match ctx.open_raw(r, &b, &a, &control) {
        Reply::Ok(_) => {}
        other => {
            ctx.record(SUITE, case, spec, &pname, &rname, Outcome::skip(format!(
                "a re-signed but untouched signed-only rfi does not open here ({})",
                other.describe()
            )));
            return;
        }
    }
    // The digest sits after -Z## XRFI and the ESSR field; find it by walking.
    let map = cesr::map_message(&packed.message).unwrap();
    let mut rd = cesr::Reader::new(&packed.message);
    rd.pos = map.cleartext_payload.unwrap();
    let digest_at = rd
        .count('Z')
        .and_then(|_| rd.code4().map(|_| ()))
        .and_then(|_| rd.var('B'))
        .map(|_| rd.pos);
    let Ok(digest_at) = digest_at else {
        ctx.record(SUITE, case, spec, &pname, &rname, Outcome::error("harness cannot find the digest"));
        return;
    };
    let tampered = cesr::flip_char(&packed.message, digest_at + 20);
    let Some(tampered) = resign(&tampered) else {
        ctx.record(SUITE, case, spec, &pname, &rname, Outcome::error("harness could not re-sign"));
        return;
    };
    let mt = Mutated { text: tampered, receiver: b, sender: a };
    let o = judge(ctx, r, &mt, &["digest"], "a tampered, re-signed rfi digest", &pname);
    ctx.record(SUITE, case, spec, &pname, &rname, o);
}

/// Edits to the published signed-only vector that only a check *after* the
/// signature can catch, because the message is re-signed with alice's
/// published key.
fn resigned_signed_only(ctx: &mut Ctx, r: usize) {
    use crate::resign::{payload, payload_body, resign, var_field, with_payload};
    let rname = ctx.name(r);
    let v = ctx.fixture["vectors"]["direct-signed-only"].clone();
    let alice = ctx.fixture_ident("alice");
    let bob = ctx.fixture_ident("bob");
    let text = v["message"].as_str().unwrap().to_string();
    let label = "resigned/direct-signed-only";
    type Build = fn(&str, &MessageMap, &Ident) -> Option<String>;
    let cases: [(&str, &str, &[&str], Build); 6] = [
        ("non-canonical-lead-byte", "§3.7 (a receiver MUST reject non-zero lead/pad bits)", &["malformed"], |t, m, _| {
            let body = payload_body(t, m)?;
            let mut rd = cesr::Reader::new(body);
            rd.code4().ok()?;
            rd.var('B').ok()?;
            rd.var('B').ok()?;
            rd.count('A').ok()?;
            let at = rd.pos;
            let f = rd.var('B').ok()?;
            if !matches!(&body[at..at + 1], "5" | "6") {
                return None;
            }
            // offset of the body within the whole message text
            let z = body.as_ptr() as usize - t.as_ptr() as usize;
            // the first data character carries the (must-be-zero) lead byte's top bits
            Some(cesr::flip_char(t, z + f.data_start))
        }),
        ("essr-sender-mismatch", "§3.7 step 7, §8.2.2 (ESSR field MUST equal VID_sndr)", &["sender"], |t, m, bob| {
            let body = payload_body(t, m)?;
            let mut rd = cesr::Reader::new(body);
            rd.code4().ok()?;
            let sender = rd.var('B').ok()?;
            let new_body = format!("{}{}{}", &body[..4], var_field('B', bob.id.as_bytes()), &body[sender.end..]);
            Some(with_payload(t, m, &payload(&new_body)))
        }),
        ("payload-count-too-large", "§9.2 (-Z count must match the payload)", &["malformed"], |t, m, _| {
            let body = payload_body(t, m)?;
            let z = crate::resign::count_code('Z', body.len() / 4 + 1);
            Some(with_payload(t, m, &format!("{z}{body}")))
        }),
        ("xscs-body-h-group-json", "tswg-tsp-specification#77; §9.2.3 (XSCS body = -A## holding exactly one Bytes primitive)", &["malformed"], |t, m, _| {
            let json = var_field('B', br#"{"hello":"world"}"#);
            let group = format!("{}{json}", crate::resign::count_code('H', json.len() / 4));
            with_stream(t, m, &group)
        }),
        ("xscs-body-two-bytes-primitives", "tswg-tsp-specification#77; §9.2.3 (exactly one Bytes primitive)", &["malformed"], |t, m, _| {
            with_stream(t, m, &format!("{}{}", var_field('B', b"one"), var_field('B', b"two")))
        }),
        ("xscs-body-data-after-stream", "tswg-tsp-specification#77; §9.2.3 (the -A## stream ends the payload frame)", &["malformed"], |t, m, _| {
            let body = payload_body(t, m)?;
            Some(with_payload(t, m, &payload(&format!("{body}{}", var_field('B', b"x")))))
        }),
    ];
    let gate = ctx.gate(r, &["signed-only".into(), "payload.scs".into()], Dir::Open);
    let control = resign(&text, &alice).map(|c| ctx.open_raw(r, &bob, &alice, &c));
    for (name, spec, expect, build) in cases {
        let case = format!("{label}/{name}");
        if !ctx.selected(SUITE, &case) {
            continue;
        }
        let o = if let Some(o) = &gate {
            o.clone()
        } else if !matches!(control, Some(Reply::Ok(_))) {
            Outcome::skip("the re-signed but untouched vector does not open here")
        } else {
            let map = cesr::map_message(&text).unwrap();
            match build(&text, &map, &bob).and_then(|t| resign(&t, &alice)) {
                None => Outcome::error("harness could not derive the message"),
                Some(t) => {
                    let mt = Mutated { text: t, receiver: bob.clone(), sender: alice.clone() };
                    judge(ctx, r, &mt, expect, name, "direct-signed-only (re-signed)")
                }
            }
        };
        ctx.record(SUITE, &case, spec, "vector (re-signed)", &rname, o);
    }
}

/// Replace the `-A##` stream of a signed-only XSCS message, keeping the type
/// code, ESSR field and padding.
fn with_stream(t: &str, m: &MessageMap, stream: &str) -> Option<String> {
    use crate::resign::{count_code, payload, payload_body, with_payload};
    let body = payload_body(t, m)?;
    let mut rd = cesr::Reader::new(body);
    if rd.code4().ok()? != "XSCS" {
        return None;
    }
    rd.var('B').ok()?;
    rd.var('B').ok()?;
    let head = &body[..rd.pos];
    let new_body = format!("{head}{}{stream}", count_code('A', stream.len() / 4));
    Some(with_payload(t, m, &payload(&new_body)))
}
