//! Suite 4 — the relationship protocol between two stateful endpoints.

use crate::ctx::Ctx;
use crate::driver::{Dir, Reply};
use crate::model::{Ident, b64};
use crate::result::{Outcome, normalize};
use serde_json::{Value, json};

const SUITE: &str = "relationship";

type R<T> = Result<T, Outcome>;

struct Pair {
    a: usize,
    b: usize,
    ea: String,
    eb: String,
    va: Ident,
    vb: Ident,
}

fn ep(ctx: &mut Ctx, i: usize, op: &str, params: Value) -> R<Value> {
    let name = ctx.name(i);
    match ctx.call(i, op, params) {
        Reply::Ok(v) => Ok(v),
        Reply::Err { code, message } if code == "unsupported" => {
            Err(Outcome::skip(format!("{name} {op}: unsupported — {message}")))
        }
        Reply::Err { code, message } => Err(Outcome::fail(
            format!("{name} {op} failed: [{code}] {message}"),
            format!("{name} {op} fails: [{code}] {}", normalize(&message)),
        )),
        Reply::Broken(m) => Err(Outcome::error(format!("{name} {op}: {m}"))),
    }
}

/// Deliver a message that must be *refused*; returns the refusal.
fn refused(ctx: &mut Ctx, i: usize, handle: &str, message: &str, what: &str) -> R<String> {
    let name = ctx.name(i);
    match ctx.call(i, "endpoint.receive", json!({"endpoint": handle, "message": message})) {
        Reply::Ok(v) => Err(Outcome::fail(
            format!("{name} accepted {what}: event {}", v["event"]),
            format!("{name} accepts {what}"),
        )),
        Reply::Err { code, message } => Ok(format!("{name} refused: [{code}] {message}")),
        Reply::Broken(m) => Err(Outcome::error(m)),
    }
}

fn finish_with(diffs: Vec<String>, cause: &str, note: String) -> Outcome {
    let mut o = finish(diffs, cause);
    if o.detail.is_empty() {
        o.detail = note;
    }
    o
}

fn setup(ctx: &mut Ctx, a: usize, b: usize) -> R<Pair> {
    let va = ctx.keys.ident("endpoint-a");
    let vb = ctx.keys.ident("endpoint-b");
    let ea = ep(ctx, a, "endpoint.create", json!({"identities": [va.private()], "peers": [vb.public()]}))?;
    let eb = ep(ctx, b, "endpoint.create", json!({"identities": [vb.private()], "peers": [va.public()]}))?;
    Ok(Pair {
        a,
        b,
        ea: ea["endpoint"].as_str().unwrap_or_default().into(),
        eb: eb["endpoint"].as_str().unwrap_or_default().into(),
        va,
        vb,
    })
}

fn state(ctx: &mut Ctx, i: usize, handle: &str, local: &Ident, remote: &Ident) -> R<Value> {
    ep(ctx, i, "endpoint.state", json!({"endpoint": handle, "local": local.id, "remote": remote.id}))
}

fn expect_state(ctx: &mut Ctx, i: usize, handle: &str, local: &Ident, remote: &Ident, want: &str, diffs: &mut Vec<String>) -> R<Value> {
    let s = state(ctx, i, handle, local, remote)?;
    if s["state"].as_str() != Some(want) {
        diffs.push(format!("{} state: expected {want}, got {}", ctx.name(i), s["state"]));
    }
    Ok(s)
}

fn check_digest(label: &str, got: &Value, want: &str, diffs: &mut Vec<String>) {
    if let Some(g) = got.as_str()
        && g != want
    {
        diffs.push(format!("{label}: expected {want}, got {g}"));
    }
}

/// invite → accept; returns (invite digest, accept digest).
fn form(ctx: &mut Ctx, p: &Pair, diffs: &mut Vec<String>) -> R<(String, String)> {
    let inv = ep(ctx, p.a, "endpoint.invite", json!({"endpoint": p.ea, "from": p.va.id, "to": p.vb.id}))?;
    let d1 = inv["digest"].as_str().unwrap_or_default().to_string();
    let got = ep(ctx, p.b, "endpoint.receive", json!({"endpoint": p.eb, "message": inv["message"]}))?;
    if got["event"] != "invite" {
        diffs.push(format!("B receive: expected event invite, got {}", got["event"]));
    }
    check_digest("B's decoded invite digest", &got["digest"], &d1, diffs);
    let s = expect_state(ctx, p.a, &p.ea.clone(), &p.va.clone(), &p.vb.clone(), "invite-sent", diffs)?;
    check_digest("A state digest", &s["digest"], &d1, diffs);
    let s = expect_state(ctx, p.b, &p.eb.clone(), &p.vb.clone(), &p.va.clone(), "invite-received", diffs)?;
    check_digest("B state digest", &s["digest"], &d1, diffs);

    let acc = ep(ctx, p.b, "endpoint.accept", json!({"endpoint": p.eb, "from": p.vb.id, "to": p.va.id}))?;
    check_digest("B accept echoes", &acc["digest"], &d1, diffs);
    let d2 = acc["replyDigest"].as_str().unwrap_or_default().to_string();
    let got = ep(ctx, p.a, "endpoint.receive", json!({"endpoint": p.ea, "message": acc["message"]}))?;
    if got["event"] != "accept" {
        diffs.push(format!("A receive: expected event accept, got {}", got["event"]));
    }
    check_digest("A's decoded accept digest", &got["digest"], &d1, diffs);
    check_digest("A's decoded accept replyDigest", &got["replyDigest"], &d2, diffs);
    Ok((d1, d2))
}

fn bidirectional(ctx: &mut Ctx, p: &Pair, d1: &str, d2: &str, diffs: &mut Vec<String>) -> R<()> {
    for (i, h, l, r) in [(p.a, p.ea.clone(), p.va.clone(), p.vb.clone()), (p.b, p.eb.clone(), p.vb.clone(), p.va.clone())] {
        let s = expect_state(ctx, i, &h, &l, &r, "bidirectional", diffs)?;
        check_digest(&format!("{} state digest", ctx.name(i)), &s["digest"], d1, diffs);
        check_digest(&format!("{} state replyDigest", ctx.name(i)), &s["replyDigest"], d2, diffs);
    }
    Ok(())
}

fn finish(diffs: Vec<String>, cause: &str) -> Outcome {
    if diffs.is_empty() {
        Outcome::pass()
    } else {
        Outcome::fail(diffs.join("\n"), cause)
    }
}

fn case_invite_accept(ctx: &mut Ctx, a: usize, b: usize) -> R<Outcome> {
    let p = setup(ctx, a, b)?;
    let mut diffs = vec![];
    let (d1, d2) = form(ctx, &p, &mut diffs)?;
    bidirectional(ctx, &p, &d1, &d2, &mut diffs)?;
    // application data both ways
    for (s, sh, r, rh, from, to) in [
        (p.a, p.ea.clone(), p.b, p.eb.clone(), p.va.clone(), p.vb.clone()),
        (p.b, p.eb.clone(), p.a, p.ea.clone(), p.vb.clone(), p.va.clone()),
    ] {
        let data = b64(format!("hello from {}", ctx.name(s)).as_bytes());
        let m = ep(ctx, s, "endpoint.send", json!({"endpoint": sh, "from": from.id, "to": to.id, "data": data}))?;
        let got = ep(ctx, r, "endpoint.receive", json!({"endpoint": rh, "message": m["message"]}))?;
        if got["event"] != "message" || got["data"] != json!(data) {
            diffs.push(format!("{} receive of application data: got {}", ctx.name(r), got));
        }
    }
    let cause = format!("{} and {} disagree forming a relationship", ctx.name(a), ctx.name(b));
    Ok(finish(diffs, &cause))
}

fn case_message_first(ctx: &mut Ctx, a: usize, b: usize) -> R<Outcome> {
    if let Some(o) = ctx.gate(a, &["hpke-base".into(), "payload.scs".into()], Dir::Pack) {
        return Ok(o);
    }
    let p = setup(ctx, a, b)?;
    let packed = ctx.pack(a, "hpke-base", &p.va, &p.vb, &json!({"type": "scs", "data": b64(b"no relationship yet")}))?;
    let note = refused(ctx, b, &p.eb, &packed.message, "an application message with no relationship (§7.2.2)")?;
    let mut diffs = vec![];
    expect_state(ctx, b, &p.eb.clone(), &p.vb.clone(), &p.va.clone(), "none", &mut diffs)?;
    Ok(finish_with(diffs, "application message changes relationship state", note))
}

fn case_cancel(ctx: &mut Ctx, a: usize, b: usize) -> R<Outcome> {
    let p = setup(ctx, a, b)?;
    let mut diffs = vec![];
    let (d1, d2) = form(ctx, &p, &mut diffs)?;
    let c = ep(ctx, a, "endpoint.cancel", json!({"endpoint": p.ea, "from": p.va.id, "to": p.vb.id}))?;
    let got = ep(ctx, b, "endpoint.receive", json!({"endpoint": p.eb, "message": c["message"]}))?;
    if got["event"] != "cancel" {
        diffs.push(format!("B receive: expected cancel, got {}", got["event"]));
    }
    if let Some(named) = got["digest"].as_str()
        && named != d1
        && named != d2
    {
        diffs.push(format!("cancel names {named}, which is neither relationship digest"));
    }
    expect_state(ctx, a, &p.ea.clone(), &p.va.clone(), &p.vb.clone(), "none", &mut diffs)?;
    expect_state(ctx, b, &p.eb.clone(), &p.vb.clone(), &p.va.clone(), "none", &mut diffs)?;
    let cause = format!("{} and {} disagree cancelling a relationship", ctx.name(a), ctx.name(b));
    Ok(finish(diffs, &cause))
}

/// After forming, one side's *library* packs an RFD naming a chosen digest (the
/// endpoint API picks its own), and the other endpoint must recognise it.
/// §7.2.2 records the Digest for <a, b> and the Reply_Digest for <b, a> at
/// both endpoints, and §7.3 lets a cancellation name either.
fn cancel_naming(ctx: &mut Ctx, a: usize, b: usize, from_inviter: bool, name_accept: bool) -> R<Outcome> {
    let (packer, _) = if from_inviter { (a, b) } else { (b, a) };
    if let Some(o) = ctx.gate(packer, &["hpke-base".into(), "payload.rfd".into()], Dir::Pack) {
        return Ok(o);
    }
    let p = setup(ctx, a, b)?;
    let mut diffs = vec![];
    let (d1, d2) = form(ctx, &p, &mut diffs)?;
    if !diffs.is_empty() {
        return Ok(Outcome::skip(format!("relationship did not form cleanly: {}", diffs.join("; "))));
    }
    let named = if name_accept { &d2 } else { &d1 };
    let (s, rcv, ri, rh) = if from_inviter {
        (p.va.clone(), p.vb.clone(), b, p.eb.clone())
    } else {
        (p.vb.clone(), p.va.clone(), a, p.ea.clone())
    };
    let packed = ctx.pack(packer, "hpke-base", &s, &rcv, &json!({"type": "rfd", "digest": named, "digestAlg": "sha2-256"}))?;
    let got = ep(ctx, ri, "endpoint.receive", json!({"endpoint": rh, "message": packed.message}))?;
    if got["event"] != "cancel" {
        diffs.push(format!("{} receive: expected cancel, got {}", ctx.name(ri), got["event"]));
    }
    expect_state(ctx, ri, &rh, &rcv, &s, "none", &mut diffs)?;
    let which = if name_accept { "the accept's digest (Reply_Digest)" } else { "the invite's digest" };
    let role = if from_inviter { "accepter" } else { "inviter" };
    let cause = format!("{} (as {role}) does not recognise a cancellation naming {which}", ctx.name(ri));
    Ok(finish(diffs, &cause))
}

fn case_race(ctx: &mut Ctx, a: usize, b: usize) -> R<Outcome> {
    let p = setup(ctx, a, b)?;
    let mut diffs = vec![];
    let ia = ep(ctx, a, "endpoint.invite", json!({"endpoint": p.ea, "from": p.va.id, "to": p.vb.id}))?;
    let ib = ep(ctx, b, "endpoint.invite", json!({"endpoint": p.eb, "from": p.vb.id, "to": p.va.id}))?;
    let (da, db) = (
        ia["digest"].as_str().unwrap_or_default().to_string(),
        ib["digest"].as_str().unwrap_or_default().to_string(),
    );
    // "lexicographical comparison" of the digest: compare the raw bytes
    let a_wins = crate::model::unb64(&da) < crate::model::unb64(&db);
    let (win, lose) = if a_wins { (&da, &db) } else { (&db, &da) };
    // cross-deliver; the side whose own invite is lower discards the other
    let ra = ctx.call(a, "endpoint.receive", json!({"endpoint": p.ea, "message": ib["message"]}));
    let rb = ctx.call(b, "endpoint.receive", json!({"endpoint": p.eb, "message": ia["message"]}));
    for (reply, i) in [(&ra, a), (&rb, b)] {
        if let Reply::Broken(m) = reply {
            return Err(Outcome::error(format!("{}: {m}", ctx.name(i))));
        }
    }
    let _ = lose;
    // the loser (higher digest) adopts the winner's invite
    let (wi, wh, wl, wr, li, lh, ll, lr) = if a_wins {
        (a, p.ea.clone(), p.va.clone(), p.vb.clone(), b, p.eb.clone(), p.vb.clone(), p.va.clone())
    } else {
        (b, p.eb.clone(), p.vb.clone(), p.va.clone(), a, p.ea.clone(), p.va.clone(), p.vb.clone())
    };
    let s = expect_state(ctx, wi, &wh, &wl, &wr, "invite-sent", &mut diffs)?;
    check_digest(&format!("{} (lower digest) state digest", ctx.name(wi)), &s["digest"], win, &mut diffs);
    let s = expect_state(ctx, li, &lh, &ll, &lr, "invite-received", &mut diffs)?;
    check_digest(&format!("{} (higher digest) state digest", ctx.name(li)), &s["digest"], win, &mut diffs);
    if !diffs.is_empty() {
        let cause = format!("{} and {} resolve the §7.2.3 invite race differently", ctx.name(a), ctx.name(b));
        return Ok(Outcome::fail(diffs.join("\n"), cause));
    }
    // and the loser now accepts the winning invite
    let acc = ep(ctx, li, "endpoint.accept", json!({"endpoint": lh, "from": ll.id, "to": lr.id}))?;
    check_digest("accept after the race echoes the lower digest", &acc["digest"], win, &mut diffs);
    let got = ep(ctx, wi, "endpoint.receive", json!({"endpoint": wh, "message": acc["message"]}))?;
    check_digest("winner decodes the echo", &got["digest"], win, &mut diffs);
    let d2 = acc["replyDigest"].as_str().unwrap_or_default().to_string();
    for (i, h, l, r) in [(wi, wh, wl, wr), (li, lh, ll, lr)] {
        let s = expect_state(ctx, i, &h, &l, &r, "bidirectional", &mut diffs)?;
        check_digest(&format!("{} state digest", ctx.name(i)), &s["digest"], win, &mut diffs);
        check_digest(&format!("{} state replyDigest", ctx.name(i)), &s["replyDigest"], &d2, &mut diffs);
    }
    let cause = format!("{} and {} resolve the §7.2.3 invite race differently", ctx.name(a), ctx.name(b));
    Ok(finish(diffs, &cause))
}

fn case_unknown_accept(ctx: &mut Ctx, a: usize, b: usize) -> R<Outcome> {
    if let Some(o) = ctx.gate(b, &["hpke-base".into(), "payload.rfa".into()], Dir::Pack) {
        return Ok(o);
    }
    let p = setup(ctx, a, b)?;
    let mut diffs = vec![];
    let _inv = ep(ctx, a, "endpoint.invite", json!({"endpoint": p.ea, "from": p.va.id, "to": p.vb.id}))?;
    // B's library packs an accept that echoes a digest A never issued
    let bogus = b64(&ctx.keys.bytes::<32>());
    let packed = ctx.pack(b, "hpke-base", &p.vb, &p.va, &json!({"type": "rfa", "digest": bogus}))?;
    let note = refused(ctx, a, &p.ea, &packed.message, "an accept naming an invite it never sent (§7.2.2)")?;
    expect_state(ctx, a, &p.ea.clone(), &p.va.clone(), &p.vb.clone(), "invite-sent", &mut diffs)?;
    Ok(finish_with(diffs, "an unmatched accept changes relationship state", note))
}

pub fn run(ctx: &mut Ctx, a: usize, b: usize) {
    let (an, bn) = (ctx.name(a), ctx.name(b));
    type F = fn(&mut Ctx, usize, usize) -> R<Outcome>;
    let cases: [(&str, &str, F); 9] = [
        ("invite-accept-bidirectional", "§7.2.1, §7.2.2", case_invite_accept),
        ("message-before-relationship-refused", "§7.2.2", case_message_first),
        ("cancel-returns-to-none", "§7.3", case_cancel),
        ("cancel-by-inviter-naming-invite-digest", "§7.2.2, §7.3", |c, a, b| cancel_naming(c, a, b, true, false)),
        ("cancel-by-inviter-naming-accept-digest", "§7.2.2 (Reply_Digest recorded by both), §7.3", |c, a, b| cancel_naming(c, a, b, true, true)),
        ("cancel-by-accepter-naming-invite-digest", "§7.2.2, §7.3", |c, a, b| cancel_naming(c, a, b, false, false)),
        ("cancel-by-accepter-naming-accept-digest", "§7.2.2 (Reply_Digest recorded by both), §7.3", |c, a, b| cancel_naming(c, a, b, false, true)),
        ("rfi-race-lower-digest-wins", "§7.2.3", case_race),
        ("accept-unknown-digest-refused", "§7.2.2", case_unknown_accept),
    ];
    for (name, spec, f) in cases {
        if !ctx.selected(SUITE, name) {
            continue;
        }
        let o = if let Some(o) = ctx
            .gate(a, &["endpoint".into()], Dir::Open)
            .or_else(|| ctx.gate(b, &["endpoint".into()], Dir::Open))
        {
            o
        } else {
            f(ctx, a, b).unwrap_or_else(|o| o)
        };
        ctx.record(SUITE, name, spec, &an, &bn, o);
    }
}
