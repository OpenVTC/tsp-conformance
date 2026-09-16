//! Suite 2 — interop over every ordered pair (P packs, R opens).

use crate::ctx::{Ctx, Packed, expect_from, payload_caps};
use crate::driver::Dir;
use crate::model::{Ident, b64};
use crate::result::Outcome;
use serde_json::{Value, json};

const SUITE: &str = "interop";

type CaseFn = fn(&mut Ctx, usize, usize) -> Outcome;

pub struct Case {
    pub name: &'static str,
    pub spec: &'static str,
    pub run: CaseFn,
}

fn scs(data: &[u8]) -> Value {
    json!({"type": "scs", "data": b64(data)})
}

/// Pack under `scheme` by P, open by R, compare every field.
fn simple(ctx: &mut Ctx, p: usize, r: usize, scheme: &str, s: &Ident, rcv: &Ident, input: Value) -> Outcome {
    match simple_inner(ctx, p, r, scheme, s, rcv, &input) {
        Ok((o, _)) => o,
        Err(o) => o,
    }
}

#[allow(clippy::too_many_arguments)]
fn simple_inner(
    ctx: &mut Ctx,
    p: usize,
    r: usize,
    scheme: &str,
    s: &Ident,
    rcv: &Ident,
    input: &Value,
) -> Result<(Outcome, Packed), Outcome> {
    if let Some(o) = ctx.gate(p, &payload_caps(scheme, input, Dir::Pack), Dir::Pack) {
        return Err(o);
    }
    if let Some(o) = ctx.gate(r, &payload_caps(scheme, input, Dir::Open), Dir::Open) {
        return Err(o);
    }
    let packed = ctx.pack(p, scheme, s, rcv, input)?;
    let exp = expect_from(scheme, s, rcv, input, &packed)?;
    let what = format!("{scheme}/{}", input["type"].as_str().unwrap_or("?"));
    let pname = ctx.name(p);
    let (_, unreported) = ctx.open_expect(&pname, r, rcv, s, &packed.message, &exp, &what)?;
    let mut o = Outcome::pass();
    o.unreported = unreported;
    Ok((o, packed))
}

fn pair(ctx: &mut Ctx) -> (Ident, Ident) {
    (ctx.keys.ident("alice"), ctx.keys.ident("bob"))
}

fn scs_hpke(ctx: &mut Ctx, p: usize, r: usize, data: Vec<u8>) -> Outcome {
    let (a, b) = pair(ctx);
    simple(ctx, p, r, "hpke-base", &a, &b, scs(&data))
}

fn padded(ctx: &mut Ctx, p: usize, r: usize, n: usize) -> Outcome {
    let (a, b) = pair(ctx);
    let pad: Vec<u8> = (0..n).map(|i| (i % 251) as u8 + 1).collect();
    let mut input = scs(b"padded payload");
    input["padding"] = json!(b64(&pad));
    simple(ctx, p, r, "hpke-base", &a, &b, input)
}

fn rfi_input(ctx: &mut Ctx) -> Value {
    json!({"type": "rfi", "nonce": b64(&ctx.keys.bytes::<16>()), "replyPath": [], "referral": null})
}

fn rfi_with(ctx: &mut Ctx, p: usize, r: usize, scheme: &str, f: impl FnOnce(&mut Ctx, &mut Value)) -> Outcome {
    let (a, b) = pair(ctx);
    let mut input = rfi_input(ctx);
    f(ctx, &mut input);
    simple(ctx, p, r, scheme, &a, &b, input)
}

/// rfa that echoes a digest P itself produced for an rfi.
fn rfa_echo_own(ctx: &mut Ctx, p: usize, r: usize) -> Outcome {
    let (a, b) = pair(ctx);
    let rfi = rfi_input(ctx);
    if let Some(o) = ctx.gate(p, &["hpke-base".into(), "payload.rfi".into(), "payload.rfa".into()], Dir::Pack) {
        return o;
    }
    let invite = match ctx.pack(p, "hpke-base", &a, &b, &rfi) {
        Ok(x) => x,
        Err(o) => return o,
    };
    let Some(d) = invite.digest else {
        return Outcome::fail("packer returned no rfi digest", "packer does not report the rfi digest");
    };
    simple(ctx, p, r, "hpke-base", &b, &a, json!({"type": "rfa", "digest": d}))
}

/// R invites, P opens that invite and answers it; R checks the echo.
fn rfa_answers_receiver(ctx: &mut Ctx, p: usize, r: usize) -> Outcome {
    let (a, b) = pair(ctx);
    let rfi = rfi_input(ctx);
    let hpke = || vec!["hpke-base".to_string(), "payload.rfi".to_string(), "payload.rfa".to_string()];
    if let Some(o) = ctx.gate(r, &hpke(), Dir::Pack).or_else(|| ctx.gate(p, &hpke(), Dir::Open)) {
        return o;
    }
    let invite = match ctx.pack(r, "hpke-base", &b, &a, &rfi) {
        Ok(x) => x,
        Err(o) => return o,
    };
    let exp = match expect_from("hpke-base", &b, &a, &rfi, &invite) {
        Ok(e) => e,
        Err(o) => return o,
    };
    let rname = ctx.name(r);
    let (opened, _) = match ctx.open_expect(&rname, p, &a, &b, &invite.message, &exp, "hpke-base/rfi") {
        Ok(x) => x,
        Err(o) => return o,
    };
    let as_decoded = opened["payload"]["digest"].clone();
    simple(ctx, p, r, "hpke-base", &a, &b, json!({"type": "rfa", "digest": as_decoded}))
}

/// Referral invite, plus an oracle check of `Signature_new` when R reports it:
/// §9.4.1 has it made by VID_new's key over {XRFI, VID_sndr | 4BAA, Digest,
/// Nonce, Reply_Path, VID_new}.
fn rfi_referral(ctx: &mut Ctx, p: usize, r: usize) -> Outcome {
    use crate::resign::{count_code, var_field};
    let (a, b) = pair(ctx);
    let new = ctx.keys.ident("alice-new");
    let mut input = rfi_input(ctx);
    input["referral"] = json!({"vid": new.id, "skS": new.sk_s});
    let (mut o, packed) = match simple_inner(ctx, p, r, "hpke-base", &a, &b, &input) {
        Ok(x) => x,
        Err(o) => return o,
    };
    // Re-open to read the decoded referral (simple_inner compared it already).
    let opened = match ctx.open_raw(r, &b, &a, &packed.message) {
        crate::driver::Reply::Ok(v) => v,
        other => return Outcome::error(format!("re-open failed: {}", other.describe())),
    };
    let pl = &opened["payload"];
    let Some(sig) = pl["referral"]["signature"].as_str() else {
        o.unreported.push("payload.referral.signature".into());
        return o;
    };
    let digest_text = format!("I{}", &crate::cesr::to_text(&[&[0u8][..], &crate::model::unb64(pl["digest"].as_str().unwrap_or(""))].concat())[1..]);
    let nonce_text = format!("0A{}", &crate::cesr::to_text(&[&[0u8, 0][..], &crate::model::unb64(input["nonce"].as_str().unwrap())].concat())[2..]);
    let challenge = |sender_field: &str| -> Vec<u8> {
        let text = format!("XRFI{sender_field}{digest_text}{nonce_text}{}{}", count_code('J', 0), var_field('B', new.id.as_bytes()));
        crate::cesr::from_text(&text).unwrap_or_default()
    };
    let Ok(sig) = <[u8; 64]>::try_from(crate::model::unb64(sig).as_slice()) else {
        return Outcome::fail("referral signature is not 64 bytes", "referral signature has the wrong length");
    };
    let vk_bytes: [u8; 32] = crate::model::unb64(&new.pk_s).try_into().unwrap();
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&vk_bytes).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(&sig);
    let ok = [var_field('B', a.id.as_bytes()), var_field('B', b"")]
        .iter()
        .any(|sf| vk.verify_strict(&challenge(sf), &sig).is_ok());
    if !ok {
        let pn = ctx.name(p);
        return Outcome::fail(
            "Signature_new does not verify under VID_new's key over {XRFI, VID_sndr, Digest, Nonce, Reply_Path, VID_new} (tried VID_sndr present and NULL)",
            format!("{pn}'s referral Signature_new does not match the §9.4.1 challenge"),
        );
    }
    o
}

fn nested_with(ctx: &mut Ctx, inner_packer: usize, p: usize, r: usize) -> Outcome {
    let (a, b) = pair(ctx);
    let a2 = ctx.keys.ident("alice-inner");
    let b2 = ctx.keys.ident("bob-inner");
    let inner_in = scs(b"inner, for bob-inner only");
    if let Some(o) = ctx.gate(inner_packer, &payload_caps("hpke-base", &inner_in, Dir::Pack), Dir::Pack) {
        return o;
    }
    let hop = json!({"type": "hop", "hops": []});
    if let Some(o) = ctx.gate(r, &["payload.scs".into()], Dir::Open) {
        return o;
    }
    let inner = match ctx.pack(inner_packer, "hpke-base", &a2, &b2, &inner_in) {
        Ok(x) => x,
        Err(o) => return o,
    };
    let mut outer_in = hop;
    outer_in["inner"] = json!(inner.message);
    let (mut o, _) = match simple_inner(ctx, p, r, "hpke-base", &a, &b, &outer_in) {
        Ok(x) => x,
        Err(o) => return o,
    };
    let exp = match expect_from("hpke-base", &a2, &b2, &inner_in, &inner) {
        Ok(e) => e,
        Err(o) => return o,
    };
    let iname = ctx.name(inner_packer);
    match ctx.open_expect(&iname, r, &b2, &a2, &inner.message, &exp, "inner hpke-base/scs") {
        Ok((_, u)) => {
            o.unreported.extend(u);
            o
        }
        Err(o) => o,
    }
}

fn routed(ctx: &mut Ctx, p: usize, r: usize) -> Outcome {
    routed_n(ctx, p, r, 2)
}

/// alice -> p (first intermediary) carrying `n` onward hops ending at bob.
fn routed_n(ctx: &mut Ctx, p: usize, r: usize, n: usize) -> Outcome {
    let (a, b) = pair(ctx);
    let hop_p = ctx.keys.ident("intermediary-p");
    let mut hops: Vec<String> = (1..n).map(|k| ctx.keys.ident(&format!("hop-{k}")).id).collect();
    hops.push(b.id.clone());
    let inner_in = scs(b"end to end");
    if let Some(o) = ctx.gate(p, &payload_caps("hpke-base", &inner_in, Dir::Pack), Dir::Pack) {
        return o;
    }
    let inner = match ctx.pack(p, "hpke-base", &a, &b, &inner_in) {
        Ok(x) => x,
        Err(o) => return o,
    };
    let outer_in = json!({"type": "hop", "hops": hops, "inner": inner.message});
    let (mut o, _) = match simple_inner(ctx, p, r, "hpke-base", &a, &hop_p, &outer_in) {
        Ok(x) => x,
        Err(o) => return o,
    };
    if let Some(g) = ctx.gate(r, &["payload.scs".into()], Dir::Open) {
        return g;
    }
    let exp = match expect_from("hpke-base", &a, &b, &inner_in, &inner) {
        Ok(e) => e,
        Err(o) => return o,
    };
    let pname = ctx.name(p);
    match ctx.open_expect(&pname, r, &b, &a, &inner.message, &exp, "routed inner hpke-base/scs") {
        Ok((_, u)) => {
            o.unreported.extend(u);
            o
        }
        Err(o) => o,
    }
}

fn long_vid(ctx: &mut Ctx, p: usize, r: usize) -> Outcome {
    // 13 000 bytes > 4095 quadlets * 3 = 12 285, so the VID needs `7AAB####`.
    let id = format!("did:example:{}", "long".repeat(3247));
    let a = ctx.keys.ident_with_id(&id);
    let b = ctx.keys.ident("bob");
    simple(ctx, p, r, "hpke-base", &a, &b, scs(b"from a long VID"))
}

fn pq(ctx: &mut Ctx, p: usize, r: usize) -> Outcome {
    let a = ctx.fixture_ident("pq_alice");
    let b = ctx.fixture_ident("pq_bob");
    simple(ctx, p, r, "hpke-pq", &a, &b, scs(b"post-quantum interop"))
}

pub fn cases() -> Vec<Case> {
    macro_rules! case {
        ($name:expr, $spec:expr, $f:expr) => {
            Case { name: $name, spec: $spec, run: $f }
        };
    }
    vec![
        case!("scs/hpke-base/small", "§3.5, §8.2, §9.2.3", |c, p, r| scs_hpke(c, p, r, b"hello interop".to_vec())),
        case!("scs/hpke-base/empty", "§9.2.3", |c, p, r| scs_hpke(c, p, r, vec![])),
        case!("scs/hpke-base/2mib", "§9.1 long count codes (--E, --Z, 7AAF)", |c, p, r| {
            scs_hpke(c, p, r, (0..2 * 1024 * 1024).map(|i| (i * 7 % 256) as u8).collect())
        }),
        case!("scs/hpke-base/padding-0", "§9.2.4", |c, p, r| padded(c, p, r, 0)),
        case!("scs/hpke-base/padding-1", "§9.2.4 lead pad 2 (6B)", |c, p, r| padded(c, p, r, 1)),
        case!("scs/hpke-base/padding-2", "§9.2.4 lead pad 1 (5B)", |c, p, r| padded(c, p, r, 2)),
        case!("scs/hpke-base/padding-3", "§9.2.4 lead pad 0 (4B)", |c, p, r| padded(c, p, r, 3)),
        case!("scs/hpke-base/padding-12285", "§9.2.4 short max (4B__)", |c, p, r| padded(c, p, r, 4095 * 3)),
        case!("scs/hpke-base/padding-12286", "§9.2.4 first long (9AAB)", |c, p, r| padded(c, p, r, 4095 * 3 + 1)),
        case!("scs/signed-only", "§3.5, §9.2", |c, p, r| {
            let (a, b) = pair(c);
            simple(c, p, r, "signed-only", &a, &b, scs(b"public announcement"))
        }),
        case!("scs/sealed-box", "§8.3, §9.2.8", |c, p, r| {
            let (a, b) = pair(c);
            simple(c, p, r, "sealed-box", &a, &b, scs(b"legacy sealed box"))
        }),
        case!("scs/hpke-base/payload-sender-null", "§3.2, §8.2.2 (ESSR NULL)", |c, p, r| {
            let (a, b) = pair(c);
            let mut i = scs(b"null essr");
            i["payloadSender"] = Value::Null;
            simple(c, p, r, "hpke-base", &a, &b, i)
        }),
        case!("scs/hpke-base/payload-sender-present", "§3.2, §8.2.2 (ESSR present)", |c, p, r| {
            let (a, b) = pair(c);
            let mut i = scs(b"explicit essr");
            i["payloadSender"] = json!(a.id);
            simple(c, p, r, "hpke-base", &a, &b, i)
        }),
        case!("scs/hpke-base/long-vid", "§9.1 VID 7AAB####", long_vid),
        case!("scs/hpke-pq", "§8.2.3, §9.5.2 (MLKEM768-X25519, ML-DSA-65)", pq),
        case!("ctl/hpke-base", "§9.3 XCTL", |c, p, r| {
            let (a, b) = pair(c);
            simple(c, p, r, "hpke-base", &a, &b, json!({"type": "ctl", "data": b64(b"{\"upper\":\"control\"}")}))
        }),
        case!("pad/hpke-base", "§7.5, §9.4.7 XPAD", |c, p, r| {
            let (a, b) = pair(c);
            simple(c, p, r, "hpke-base", &a, &b, json!({"type": "pad"}))
        }),
        case!("pad/hpke-base/padded", "§7.5, §9.4.7 XPAD with Padding_Field", |c, p, r| {
            let (a, b) = pair(c);
            let mut i = json!({"type": "pad"});
            i["padding"] = json!(b64(&[0u8; 30]));
            simple(c, p, r, "hpke-base", &a, &b, i)
        }),
        case!("rfi/hpke-base/direct", "§7.2.1, §7.2.2, §9.4.1", |c, p, r| rfi_with(c, p, r, "hpke-base", |_, _| {})),
        case!("rfi/hpke-base/reply-path", "§7.2.4, §9.4.1 Reply_Path", |c, p, r| {
            rfi_with(c, p, r, "hpke-base", |c, i| {
                let h1 = c.keys.ident("reply-hop-1");
                let h2 = c.keys.ident("reply-hop-2");
                i["replyPath"] = json!([h1.id, h2.id]);
            })
        }),
        case!("rfi/hpke-base/referral", "§7.2.5, §9.4.1 Referral_Field and Signature_new", rfi_referral),
        case!("rfi/sealed-box", "§7.2.1, §8.3 (Blake2b-256 digest F)", |c, p, r| rfi_with(c, p, r, "sealed-box", |_, _| {})),
        case!("rfa/hpke-base/echo-own-rfi", "§7.2.2, §9.4.2", rfa_echo_own),
        case!("rfa/hpke-base/answers-receiver-rfi", "§7.2.2, §9.4.2 (cross-derived)", rfa_answers_receiver),
        case!("rfd/hpke-base", "§7.3, §9.4.5", |c, p, r| {
            let (a, b) = pair(c);
            let d = c.keys.bytes::<32>();
            simple(c, p, r, "hpke-base", &a, &b, json!({"type": "rfd", "digest": b64(&d), "digestAlg": "sha2-256"}))
        }),
        case!("rfi/hpke-base/padded", "§7.2.1 (Padding_Field excluded from the SAID)", |c, p, r| {
            rfi_with(c, p, r, "hpke-base", |_, i| i["padding"] = json!(b64(&[7u8; 10])))
        }),
        case!("rfi/hpke-pq", "§7.2.1, §8.2.3", |c, p, r| {
            let a = c.fixture_ident("pq_alice");
            let b = c.fixture_ident("pq_bob");
            let i = rfi_input(c);
            simple(c, p, r, "hpke-pq", &a, &b, i)
        }),
        case!("rfa/sealed-box", "§7.2.2, §8.3 (Blake2b-256)", |c, p, r| {
            let (a, b) = pair(c);
            let d = c.keys.bytes::<32>();
            simple(c, p, r, "sealed-box", &a, &b, json!({"type": "rfa", "digest": b64(&d)}))
        }),
        case!("rfd/sealed-box", "§7.3, §8.3 (Blake2b-256)", |c, p, r| {
            let (a, b) = pair(c);
            let d = c.keys.bytes::<32>();
            simple(c, p, r, "sealed-box", &a, &b, json!({"type": "rfd", "digest": b64(&d), "digestAlg": "blake2b-256"}))
        }),
        case!("hop/nested", "§4, §9.4.15", |c, p, r| nested_with(c, p, p, r)),
        case!("hop/nested/padded", "§4, §9.4.15 (Padding_Field before the inner message)", |c, p, r| {
            let (a, b) = pair(c);
            let (a2, b2) = (c.keys.ident("alice-inner"), c.keys.ident("bob-inner"));
            if let Some(o) = c.gate(p, &payload_caps("hpke-base", &scs(b""), Dir::Pack), Dir::Pack) {
                return o;
            }
            let inner = match c.pack(p, "hpke-base", &a2, &b2, &scs(b"inner")) {
                Ok(x) => x,
                Err(o) => return o,
            };
            simple(c, p, r, "hpke-base", &a, &b,
                json!({"type": "hop", "hops": [], "inner": inner.message, "padding": b64(&[9u8; 20])}))
        }),
        case!("hop/nested/signed-only-inner", "§4 (a signed-only inner message is permitted)", |c, p, r| {
            let (a, b) = pair(c);
            let (a2, b2) = (c.keys.ident("alice-inner"), c.keys.ident("bob-inner"));
            let inner_in = scs(b"signed, not sealed, inside a sealed outer");
            if let Some(o) = c.gate(p, &payload_caps("signed-only", &inner_in, Dir::Pack), Dir::Pack)
                .or_else(|| c.gate(r, &payload_caps("signed-only", &inner_in, Dir::Open), Dir::Open))
            {
                return o;
            }
            let inner = match c.pack(p, "signed-only", &a2, &b2, &inner_in) {
                Ok(x) => x,
                Err(o) => return o,
            };
            let (mut o, _) = match simple_inner(c, p, r, "hpke-base", &a, &b,
                &json!({"type": "hop", "hops": [], "inner": inner.message}))
            {
                Ok(x) => x,
                Err(o) => return o,
            };
            let exp = match expect_from("signed-only", &a2, &b2, &inner_in, &inner) {
                Ok(e) => e,
                Err(o) => return o,
            };
            let pn = c.name(p);
            match c.open_expect(&pn, r, &b2, &a2, &inner.message, &exp, "signed-only inner") {
                Ok((_, u)) => {
                    o.unreported.extend(u);
                    o
                }
                Err(o) => o,
            }
        }),
        case!("hop/routed-2-hops", "§5.3, §9.4.16", routed),
        case!("hop/routed-long-hop-list", "§9.4.16 hop list over 4095 quadlets (--J#####)", |c, p, r| {
            let (a, b) = pair(c);
            let hop_p = c.keys.ident("intermediary-p");
            let far = c.keys.ident_with_id(&format!("did:example:far-hop-{}", "x".repeat(13000)));
            if let Some(o) = c.gate(p, &payload_caps("hpke-base", &scs(b""), Dir::Pack), Dir::Pack) {
                return o;
            }
            let inner = match c.pack(p, "hpke-base", &a, &b, &scs(b"far away")) {
                Ok(x) => x,
                Err(o) => return o,
            };
            simple(c, p, r, "hpke-base", &a, &hop_p,
                json!({"type": "hop", "hops": [far.id, b.id], "inner": inner.message}))
        }),
        case!("rfi/hpke-base/long-reply-path", "§9.4.1 Reply_Path over 4095 quadlets (--J#####)", |c, p, r| {
            rfi_with(c, p, r, "hpke-base", |c, i| {
                let h = c.keys.ident_with_id(&format!("did:example:reply-{}", "y".repeat(13000)));
                i["replyPath"] = json!([h.id]);
            })
        }),
        case!("hop/routed-12-hops", "§5.3, §9.4.16 (the spec sets no hop limit)", |c, p, r| routed_n(c, p, r, 12)),
        case!("hop/routed-17-hops", "§5.3, §9.4.16 (the spec sets no hop limit)", |c, p, r| routed_n(c, p, r, 17)),
    ]
}

pub fn run(ctx: &mut Ctx, p: usize, r: usize) {
    let (pn, rn) = (ctx.name(p), ctx.name(r));
    for case in cases() {
        if !ctx.selected(SUITE, case.name) {
            continue;
        }
        let o = (case.run)(ctx, p, r);
        ctx.record(SUITE, case.name, case.spec, &pn, &rn, o);
    }
}

/// Three-party mixing: inner by X, outer by P, both opened by R.
pub fn run_mixed(ctx: &mut Ctx, x: usize, p: usize, r: usize) {
    let case = format!("hop/nested-mixed/inner-by-{}", ctx.name(x));
    if !ctx.selected(SUITE, &case) {
        return;
    }
    let o = nested_with(ctx, x, p, r);
    let (pn, rn) = (ctx.name(p), ctx.name(r));
    ctx.record(SUITE, &case, "§4, §9.4.15 (three implementations)", &pn, &rn, o);
}
