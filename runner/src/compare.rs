//! Field-by-field comparison of an `open` result against an expectation.

use crate::model::short;
use serde_json::Value;

/// What the ESSR sender field inside the payload should be.
#[derive(Debug, Clone)]
pub enum SenderExpect {
    /// The packer was told (or the scheme requires) exactly this.
    Exactly(Option<String>),
    /// The packer took its library default: NULL or the envelope sender are
    /// both conformant under HPKE-Base and signed-only.
    NullOr(String),
}

#[derive(Debug, Clone)]
pub struct Expect {
    pub version: (u64, u64),
    pub sender: String,
    pub receiver: Option<String>,
    pub scheme: String,
    /// Expected payload; keys present are compared. `padding` is base64url.
    pub payload: Value,
    pub payload_sender: SenderExpect,
}

/// Output fields a library may legitimately not expose (driver-protocol §7).
const MAY_BE_UNREPORTED: &[&str] = &["padding", "payloadSender", "nonce"];

#[derive(Default, Debug)]
pub struct Comparison {
    pub diffs: Vec<String>,
    pub unreported: Vec<String>,
    pub notes: Vec<String>,
}

pub fn compare(exp: &Expect, got: &Value) -> Comparison {
    let mut c = Comparison::default();

    match got.get("version") {
        None => c.unreported.push("version".into()),
        Some(v) => {
            let (maj, min) = (v["major"].as_u64(), v["minor"].as_u64());
            if (maj, min) != (Some(exp.version.0), Some(exp.version.1)) {
                c.diffs.push(format!(
                    "version: expected {}.{} (YTSP-AAC), got {}",
                    exp.version.0,
                    exp.version.1,
                    short(v)
                ));
            }
        }
    }
    if got["envelopeSender"].as_str() != Some(exp.sender.as_str()) {
        c.diffs.push(format!(
            "envelopeSender: expected {}, got {}",
            short(&Value::String(exp.sender.clone())),
            short(&got["envelopeSender"])
        ));
    }
    let want_r = exp.receiver.clone().map(Value::String).unwrap_or(Value::Null);
    let got_r = got.get("envelopeReceiver").cloned().unwrap_or(Value::Null);
    if want_r != got_r {
        c.diffs.push(format!(
            "envelopeReceiver: expected {}, got {}",
            short(&want_r),
            short(&got_r)
        ));
    }
    if got["scheme"].as_str() != Some(exp.scheme.as_str()) {
        c.diffs.push(format!(
            "scheme: expected {}, got {}",
            exp.scheme,
            short(&got["scheme"])
        ));
    }

    let gp = &got["payload"];
    let ep = &exp.payload;
    if gp["type"] != ep["type"] {
        c.diffs.push(format!(
            "payload.type: expected {}, got {}",
            short(&ep["type"]),
            short(&gp["type"])
        ));
        return c;
    }
    if let Some(obj) = ep.as_object() {
        for (k, want) in obj {
            if k == "type" || k == "payloadSender" {
                continue;
            }
            let Some(have) = gp.get(k) else {
                if MAY_BE_UNREPORTED.contains(&k.as_str()) {
                    c.unreported.push(format!("payload.{k}"));
                } else {
                    c.diffs.push(format!("payload.{k}: expected {}, not reported", short(want)));
                }
                continue;
            };
            if k == "referral" {
                compare_referral(want, have, &mut c);
                continue;
            }
            if k == "padding" {
                let (w, h) = (want.as_str().unwrap_or(""), have.as_str().unwrap_or(""));
                if w != h {
                    c.diffs.push(format!(
                        "payload.padding: expected {} bytes, got {} bytes",
                        crate::model::unb64(w).len(),
                        crate::model::unb64(h).len()
                    ));
                }
                continue;
            }
            if want != have {
                c.diffs.push(format!(
                    "payload.{k}: expected {}, got {}",
                    short(want),
                    short(have)
                ));
            }
        }
    }

    match gp.get("payloadSender") {
        None => c.unreported.push("payload.payloadSender".into()),
        Some(have) => {
            let have = have.as_str().map(str::to_string);
            match &exp.payload_sender {
                SenderExpect::Exactly(want) => {
                    if &have != want {
                        c.diffs.push(format!(
                            "payload.payloadSender: expected {want:?}, got {have:?}"
                        ));
                    }
                }
                SenderExpect::NullOr(vid) => match have {
                    None => c.notes.push("payloadSender: NULL".into()),
                    Some(h) if &h == vid => c.notes.push("payloadSender: present".into()),
                    Some(h) => c.diffs.push(format!(
                        "payload.payloadSender: expected NULL or {vid}, got {h}"
                    )),
                },
            }
        }
    }
    c
}

fn compare_referral(want: &Value, have: &Value, c: &mut Comparison) {
    match (want, have) {
        (Value::Null, Value::Null) => {}
        (Value::Object(w), Value::Object(h)) => {
            if w.get("vid") != h.get("vid") {
                c.diffs.push(format!(
                    "payload.referral.vid: expected {}, got {}",
                    short(&w["vid"]),
                    short(&h["vid"])
                ));
            }
            match (w.get("signature"), h.get("signature")) {
                (Some(ws), Some(hs)) if ws != hs => c.diffs.push(format!(
                    "payload.referral.signature: expected {}, got {}",
                    short(ws),
                    short(hs)
                )),
                (_, None) => c.unreported.push("payload.referral.signature".into()),
                _ => {}
            }
        }
        _ => c.diffs.push(format!(
            "payload.referral: expected {}, got {}",
            short(want),
            short(have)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn exp() -> Expect {
        Expect {
            version: (0, 2),
            sender: "did:a".into(),
            receiver: Some("did:b".into()),
            scheme: "hpke-base".into(),
            payload: json!({"type": "rfi", "digest": "AAAA", "digestAlg": "sha2-256", "nonce": "BBBB", "replyPath": [], "referral": null, "padding": ""}),
            payload_sender: SenderExpect::NullOr("did:a".into()),
        }
    }

    fn got() -> Value {
        json!({"version": {"major": 0, "minor": 2}, "envelopeSender": "did:a", "envelopeReceiver": "did:b",
               "scheme": "hpke-base",
               "payload": {"type": "rfi", "digest": "AAAA", "digestAlg": "sha2-256", "nonce": "BBBB", "replyPath": [], "referral": null}})
    }

    #[test]
    fn identical_passes_and_lists_unreported() {
        let c = compare(&exp(), &got());
        assert!(c.diffs.is_empty(), "{:?}", c.diffs);
        assert!(c.unreported.contains(&"payload.padding".to_string()));
        assert!(c.unreported.contains(&"payload.payloadSender".to_string()));
    }

    #[test]
    fn every_field_is_compared() {
        for (path, bad) in [
            ("/version/minor", json!(64)),
            ("/envelopeSender", json!("did:x")),
            ("/envelopeReceiver", Value::Null),
            ("/scheme", json!("sealed-box")),
            ("/payload/digest", json!("CCCC")),
            ("/payload/digestAlg", json!("blake2b-256")),
            ("/payload/nonce", json!("DDDD")),
            ("/payload/replyPath", json!(["did:h"])),
            ("/payload/referral", json!({"vid": "did:n"})),
        ] {
            let mut g = got();
            *g.pointer_mut(path).unwrap() = bad;
            assert!(!compare(&exp(), &g).diffs.is_empty(), "{path} not compared");
        }
        let mut g = got();
        g["payload"].as_object_mut().unwrap().remove("digest");
        assert!(!compare(&exp(), &g).diffs.is_empty(), "missing digest not a diff");
        let mut g = got();
        g["payload"]["payloadSender"] = json!("did:x");
        assert!(!compare(&exp(), &g).diffs.is_empty(), "wrong ESSR sender not a diff");
    }
}
