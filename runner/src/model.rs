//! Identities, keys and the JSON shapes of the driver protocol.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use serde_json::{Map, Value, json};

pub fn b64(b: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(b)
}

pub fn unb64(s: &str) -> Vec<u8> {
    URL_SAFE_NO_PAD
        .decode(s.trim_end_matches('='))
        .unwrap_or_default()
}

/// A protocol `Identity` with its private half.
#[derive(Debug, Clone)]
pub struct Ident {
    pub id: String,
    pub sig_key_type: String,
    pub enc_key_type: String,
    pub pk_s: String,
    pub pk_e: String,
    pub sk_s: String,
    pub sk_e: String,
}

impl Ident {
    pub fn private(&self) -> Value {
        json!({
            "id": self.id, "sigKeyType": self.sig_key_type, "encKeyType": self.enc_key_type,
            "pkS": self.pk_s, "pkE": self.pk_e, "skS": self.sk_s, "skE": self.sk_e,
        })
    }

    pub fn public(&self) -> Value {
        json!({
            "id": self.id, "sigKeyType": self.sig_key_type, "encKeyType": self.enc_key_type,
            "pkS": self.pk_s, "pkE": self.pk_e,
        })
    }

    /// The same keys under another VID.
    pub fn renamed(&self, id: impl Into<String>) -> Ident {
        Ident {
            id: id.into(),
            ..self.clone()
        }
    }

    pub fn from_fixture(v: &Value) -> Ident {
        let s = |k: &str| v[k].as_str().unwrap_or_default().to_string();
        Ident {
            id: s("id"),
            sig_key_type: s("sigKeyType"),
            enc_key_type: s("encKeyType"),
            pk_s: s("pkS"),
            pk_e: s("pkE"),
            sk_s: s("skS"),
            sk_e: s("skE"),
        }
    }

    /// Ed25519 signing key bytes, for the negative suite's re-signing tool.
    pub fn ed25519_signing_key(&self) -> Option<ed25519_dalek::SigningKey> {
        let sk: [u8; 32] = unb64(&self.sk_s).try_into().ok()?;
        Some(ed25519_dalek::SigningKey::from_bytes(&sk))
    }
}

/// Fresh Ed25519/X25519 identities from a seeded generator.
pub struct KeyFactory {
    rng: ChaCha20Rng,
    counter: u64,
}

impl KeyFactory {
    pub fn new(seed: u64) -> Self {
        KeyFactory {
            rng: ChaCha20Rng::seed_from_u64(seed),
            counter: 0,
        }
    }

    pub fn bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut b = [0u8; N];
        self.rng.fill_bytes(&mut b);
        b
    }

    pub fn ident(&mut self, label: &str) -> Ident {
        self.counter += 1;
        let tag = hex::encode(self.bytes::<4>());
        self.ident_with_id(&format!("did:web:{label}-{tag}.conformance.example"))
    }

    pub fn ident_with_id(&mut self, id: &str) -> Ident {
        let sign = ed25519_dalek::SigningKey::from_bytes(&self.bytes::<32>());
        let enc = x25519_dalek::StaticSecret::from(self.bytes::<32>());
        Ident {
            id: id.to_string(),
            sig_key_type: "Ed25519".into(),
            enc_key_type: "X25519".into(),
            pk_s: b64(sign.verifying_key().as_bytes()),
            pk_e: b64(x25519_dalek::PublicKey::from(&enc).as_bytes()),
            sk_s: b64(&sign.to_bytes()),
            sk_e: b64(enc.as_bytes()),
        }
    }
}

/// Build a JSON object from pairs, skipping `None`s.
pub fn obj(pairs: &[(&str, Option<Value>)]) -> Value {
    let mut m = Map::new();
    for (k, v) in pairs {
        if let Some(v) = v {
            m.insert((*k).to_string(), v.clone());
        }
    }
    Value::Object(m)
}

pub fn short(v: &Value) -> String {
    let s = match v {
        Value::String(s) => format!("\"{s}\""),
        other => other.to_string(),
    };
    if s.chars().count() > 96 {
        format!("{}…({} chars)", s.chars().take(80).collect::<String>(), s.chars().count())
    } else {
        s
    }
}
