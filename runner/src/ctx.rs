//! Shared run state and the request helpers every suite uses.

use crate::compare::{Expect, SenderExpect, compare};
use crate::driver::{Dir, Driver, Reply};
use crate::model::{Ident, KeyFactory, obj};
use crate::result::{CaseResult, Outcome, Status, normalize};
use serde_json::{Value, json};

pub struct Ctx {
    pub drivers: Vec<Driver>,
    pub fixture: Value,
    pub keys: KeyFactory,
    pub filters: Vec<String>,
    pub results: Vec<CaseResult>,
    pub verbose: bool,
}

pub struct Packed {
    pub message: String,
    pub digest: Option<String>,
    pub digest_alg: Option<String>,
}

impl Ctx {
    pub fn name(&self, i: usize) -> String {
        self.drivers[i].name().to_string()
    }

    pub fn selected(&self, suite: &str, case: &str) -> bool {
        let full = format!("{suite}/{case}");
        self.filters.is_empty() || self.filters.iter().any(|f| full.contains(f.as_str()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        suite: &str,
        case: &str,
        spec: &str,
        sender: &str,
        receiver: &str,
        o: Outcome,
    ) {
        let mark = match o.status {
            Status::Pass => "pass",
            Status::Fail => "FAIL",
            Status::Skip => "skip",
            Status::Error => "ERROR",
        };
        if self.verbose || matches!(o.status, Status::Fail | Status::Error) {
            eprintln!(
                "  {mark:5} {suite}/{case} [{sender} -> {receiver}] {}",
                o.detail.lines().next().unwrap_or("")
            );
        }
        self.results.push(CaseResult {
            suite: suite.into(),
            case: case.into(),
            spec: spec.into(),
            sender: sender.into(),
            receiver: receiver.into(),
            status: o.status,
            detail: o.detail,
            warnings: o.warnings,
            unreported: o.unreported,
            cause: o.cause,
            finding: None,
        });
    }

    pub fn fixture_ident(&self, name: &str) -> Ident {
        Ident::from_fixture(&self.fixture["identifiers"][name])
    }

    /// Skip unless driver `i` has every capability in `caps` for `dir`.
    pub fn gate(&self, i: usize, caps: &[String], dir: Dir) -> Option<Outcome> {
        let d = &self.drivers[i];
        let missing: Vec<_> = caps.iter().filter(|c| !d.has(c, dir)).cloned().collect();
        if missing.is_empty() {
            None
        } else {
            let dir = match dir {
                Dir::Pack => "pack",
                Dir::Open => "open",
            };
            Some(Outcome::skip(format!(
                "{} lacks {}",
                d.name(),
                missing
                    .iter()
                    .map(|m| format!("{m}:{dir}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )))
        }
    }

    pub fn call(&mut self, i: usize, op: &str, params: Value) -> Reply {
        self.drivers[i].call(op, params)
    }

    /// `pack`, mapping every non-success to the outcome the case should take.
    pub fn pack(
        &mut self,
        i: usize,
        scheme: &str,
        sender: &Ident,
        receiver: &Ident,
        payload: &Value,
    ) -> Result<Packed, Outcome> {
        let name = self.name(i);
        let reply = self.call(
            i,
            "pack",
            json!({
                "scheme": scheme,
                "sender": sender.private(),
                "receiver": receiver.public(),
                "payload": payload,
                "ephemeral": null,
            }),
        );
        match reply {
            Reply::Ok(v) => Ok(Packed {
                message: v["message"].as_str().unwrap_or_default().to_string(),
                digest: v["digest"].as_str().map(str::to_string),
                digest_alg: v["digestAlg"].as_str().map(str::to_string),
            }),
            Reply::Err { code, message } if code == "unsupported" => {
                Err(Outcome::skip(format!("{name} pack: unsupported — {message}")))
            }
            Reply::Err { code, message } => Err(Outcome::fail(
                format!("{name} refused to pack: [{code}] {message}"),
                format!(
                    "{name} refuses to pack {scheme}/{}: [{code}] {}",
                    payload["type"].as_str().unwrap_or("?"),
                    normalize(&message)
                ),
            )),
            Reply::Broken(m) => Err(Outcome::error(format!("{name} pack: {m}"))),
        }
    }

    /// `open`, returning the raw reply.
    pub fn open_raw(&mut self, i: usize, receiver: &Ident, sender: &Ident, message: &str) -> Reply {
        self.call(
            i,
            "open",
            json!({"receiver": receiver.private(), "sender": sender.public(), "message": message}),
        )
    }

    /// Open and compare with an expectation.
    #[allow(clippy::too_many_arguments)]
    pub fn open_expect(
        &mut self,
        packer: &str,
        r: usize,
        receiver: &Ident,
        sender: &Ident,
        message: &str,
        exp: &Expect,
        what: &str,
    ) -> Result<(Value, Vec<String>), Outcome> {
        let rname = self.name(r);
        match self.open_raw(r, receiver, sender, message) {
            Reply::Ok(v) => {
                let c = compare(exp, &v);
                if c.diffs.is_empty() {
                    Ok((v, c.unreported))
                } else {
                    let fields: Vec<String> = c
                        .diffs
                        .iter()
                        .map(|d| d.split(':').next().unwrap_or("").to_string())
                        .collect();
                    let mut o = Outcome::fail(
                        c.diffs.join("\n"),
                        format!(
                            "{rname} decodes {packer}'s {what} with different {}",
                            fields.join(", ")
                        ),
                    );
                    o.unreported = c.unreported;
                    Err(o)
                }
            }
            Reply::Err { code, message } if code == "unsupported" => {
                Err(Outcome::skip(format!("{rname} open: unsupported — {message}")))
            }
            Reply::Err { code, message } => Err(Outcome::fail(
                format!("{rname} refused to open {packer}'s message: [{code}] {message}"),
                format!("{rname} refuses {packer}'s {what}: [{code}] {}", normalize(&message)),
            )),
            Reply::Broken(m) => Err(Outcome::error(format!("{rname} open: {m}"))),
        }
    }

}

/// Capabilities a payload needs, beyond the scheme.
pub fn payload_caps(scheme: &str, payload: &Value, dir: Dir) -> Vec<String> {
    let mut caps = vec![
        scheme.to_string(),
        format!("payload.{}", payload["type"].as_str().unwrap_or("?")),
    ];
    if payload["replyPath"].as_array().is_some_and(|a| !a.is_empty()) {
        caps.push("rfi.reply-path".into());
    }
    if !payload["referral"].is_null() && payload.get("referral").is_some() {
        caps.push("rfi.referral".into());
    }
    if dir == Dir::Pack
        && payload["padding"].as_str().is_some_and(|p| !p.is_empty())
    {
        caps.push("padding".into());
    }
    caps
}

pub fn digest_alg_for(scheme: &str) -> &'static str {
    if scheme == "sealed-box" {
        "blake2b-256"
    } else {
        "sha2-256"
    }
}

/// The expectation for opening what `sender` packed from `input`.
pub fn expect_from(
    scheme: &str,
    sender: &Ident,
    receiver: &Ident,
    input: &Value,
    packed: &Packed,
) -> Result<Expect, Outcome> {
    let ty = input["type"].as_str().unwrap_or("?");
    let alg = json!(digest_alg_for(scheme));
    let own_digest = || -> Result<Value, Outcome> {
        packed.digest.clone().map(Value::String).ok_or_else(|| {
            Outcome::fail(
                format!("packer returned no digest for {ty}"),
                format!("packer does not report the {ty} digest"),
            )
        })
    };
    let payload = match ty {
        "scs" | "ctl" => obj(&[("type", Some(json!(ty))), ("data", Some(input["data"].clone()))]),
        "pad" => obj(&[("type", Some(json!(ty))), ("nonce", input.get("nonce").cloned())]),
        "rfi" => obj(&[
            ("type", Some(json!(ty))),
            ("digest", Some(own_digest()?)),
            ("digestAlg", Some(alg)),
            ("nonce", input.get("nonce").cloned()),
            ("replyPath", Some(input.get("replyPath").cloned().unwrap_or(json!([])))),
            (
                "referral",
                Some(match input.get("referral") {
                    Some(r) if !r.is_null() => json!({"vid": r["vid"]}),
                    _ => Value::Null,
                }),
            ),
        ]),
        "rfa" => obj(&[
            ("type", Some(json!(ty))),
            ("digest", Some(input["digest"].clone())),
            ("replyDigest", Some(own_digest()?)),
            ("digestAlg", Some(alg)),
        ]),
        "rfd" => obj(&[
            ("type", Some(json!(ty))),
            ("digest", Some(input["digest"].clone())),
            ("digestAlg", Some(alg)),
        ]),
        "hop" => obj(&[
            ("type", Some(json!(ty))),
            ("hops", Some(input.get("hops").cloned().unwrap_or(json!([])))),
            ("inner", Some(input["inner"].clone())),
        ]),
        other => return Err(Outcome::error(format!("harness: unknown payload type {other}"))),
    };
    let mut payload = payload;
    payload["padding"] = json!(input["padding"].as_str().unwrap_or(""));
    if let (Some(pd), Some(pa)) = (&packed.digest, &packed.digest_alg)
        && matches!(ty, "rfi" | "rfa")
        && pa != digest_alg_for(scheme)
    {
        let _ = pd;
        return Err(Outcome::fail(
            format!("packer reported digestAlg {pa} under {scheme}"),
            "packer reports the wrong digest algorithm for the scheme",
        ));
    }
    let payload_sender = match input.get("payloadSender") {
        Some(Value::Null) => SenderExpect::Exactly(None),
        Some(Value::String(s)) => SenderExpect::Exactly(Some(s.clone())),
        _ if scheme == "sealed-box" => SenderExpect::Exactly(Some(sender.id.clone())),
        _ => SenderExpect::NullOr(sender.id.clone()),
    };
    Ok(Expect {
        version: (0, 2),
        sender: sender.id.clone(),
        receiver: Some(receiver.id.clone()),
        scheme: scheme.into(),
        payload,
        payload_sender,
    })
}
