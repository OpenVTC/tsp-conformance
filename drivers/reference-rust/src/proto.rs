//! Driver-protocol plumbing shared (by copy) between the Rust drivers.
//!
//! Nothing in here knows anything about TSP: it frames JSON lines, decodes
//! base64url and parses the protocol's `Identity` object. Kept as a copied file
//! rather than a shared crate so each driver stays a standalone cargo workspace
//! whose dependency graph is never resolved together with another library's.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

#[derive(Debug)]
pub struct DErr {
    pub code: &'static str,
    pub message: String,
}

pub fn err(code: &'static str, message: impl Into<String>) -> DErr {
    DErr {
        code,
        message: message.into(),
    }
}

pub fn unsupported(message: impl Into<String>) -> DErr {
    err("unsupported", message)
}

pub type DResult<T> = Result<T, DErr>;

pub fn b64d(s: &str) -> DResult<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(s.trim_end_matches('='))
        .map_err(|e| err("invalid-input", format!("bad base64url: {e}")))
}

pub fn b64e(b: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(b)
}

pub fn str_field<'a>(v: &'a Value, k: &str) -> DResult<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| err("invalid-input", format!("missing string field `{k}`")))
}

pub fn opt_bytes(v: &Value, k: &str) -> DResult<Option<Vec<u8>>> {
    match v.get(k) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => b64d(s).map(Some),
        Some(_) => Err(err("invalid-input", format!("`{k}` must be base64url"))),
    }
}

pub fn bytes_field(v: &Value, k: &str) -> DResult<Vec<u8>> {
    opt_bytes(v, k)?.ok_or_else(|| err("invalid-input", format!("missing bytes field `{k}`")))
}

pub fn fixed<const N: usize>(b: &[u8], what: &str) -> DResult<[u8; N]> {
    b.try_into()
        .map_err(|_| err("invalid-input", format!("{what}: expected {N} bytes, got {}", b.len())))
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub sig_key_type: String,
    pub enc_key_type: String,
    pub pk_s: Vec<u8>,
    pub pk_e: Vec<u8>,
    pub sk_s: Option<Vec<u8>>,
    pub sk_e: Option<Vec<u8>>,
}

impl Identity {
    pub fn parse(v: &Value) -> DResult<Identity> {
        Ok(Identity {
            id: str_field(v, "id")?.to_string(),
            sig_key_type: v
                .get("sigKeyType")
                .and_then(Value::as_str)
                .unwrap_or("Ed25519")
                .to_string(),
            enc_key_type: v
                .get("encKeyType")
                .and_then(Value::as_str)
                .unwrap_or("X25519")
                .to_string(),
            pk_s: bytes_field(v, "pkS")?,
            pk_e: bytes_field(v, "pkE")?,
            sk_s: opt_bytes(v, "skS")?,
            sk_e: opt_bytes(v, "skE")?,
        })
    }

    pub fn field(v: &Value, k: &str) -> DResult<Identity> {
        Identity::parse(
            v.get(k)
                .ok_or_else(|| err("invalid-input", format!("missing identity `{k}`")))?,
        )
    }

    pub fn is_pq(&self) -> bool {
        self.sig_key_type == "MlDsa65" || self.enc_key_type == "MLKEM768-X25519"
    }

    pub fn sk_s(&self) -> DResult<&[u8]> {
        self.sk_s
            .as_deref()
            .ok_or_else(|| err("invalid-input", format!("identity {} has no skS", self.id)))
    }

    pub fn sk_e(&self) -> DResult<&[u8]> {
        self.sk_e
            .as_deref()
            .ok_or_else(|| err("invalid-input", format!("identity {} has no skE", self.id)))
    }
}

/// Run the request loop until stdin closes.
pub fn run(mut handle: impl FnMut(&str, &Value) -> DResult<Value>) {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Err(e) => json!({"id": null, "ok": false, "error": {"code": "invalid-input", "message": format!("request is not JSON: {e}")}}),
            Ok(req) => {
                let id = req.get("id").cloned().unwrap_or(Value::Null);
                let op = req.get("op").and_then(Value::as_str).unwrap_or("").to_string();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handle(&op, &req)
                }));
                match outcome {
                    Ok(Ok(result)) => json!({"id": id, "ok": true, "result": result}),
                    Ok(Err(e)) => {
                        json!({"id": id, "ok": false, "error": {"code": e.code, "message": e.message}})
                    }
                    Err(p) => {
                        let msg = p
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "panic".into());
                        json!({"id": id, "ok": false, "error": {"code": "internal", "message": format!("library panicked: {msg}")}})
                    }
                }
            }
        };
        let _ = writeln!(stdout, "{response}");
        let _ = stdout.flush();
    }
}
