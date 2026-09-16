//! Derive new signed-only messages from a published one by editing the qb64
//! text and re-signing with the sender's (published or generated) Ed25519 key.
//!
//! Only signed-only messages can be derived this way: a confidential payload
//! would also have to be re-encrypted, and the runner implements no HPKE.
//! Re-signing is what isolates the property under test — without it every
//! edit is caught by the signature before the check of interest is reached.

use crate::cesr::{self, MessageMap};
use crate::model::Ident;
use ed25519_dalek::Signer;

/// Replace the trailing Ed25519 signature with one over the (new) content.
pub fn resign(text: &str, signer: &Ident) -> Option<String> {
    let map = cesr::map_message(text).ok()?;
    let mut bytes = cesr::from_text(text).ok()?;
    let content_end = map.content_end / 4 * 3;
    let sig = signer.ed25519_signing_key()?.sign(&bytes[..content_end]).to_bytes();
    let n = bytes.len();
    if n < content_end + 66 {
        return None;
    }
    bytes[n - 64..].copy_from_slice(&sig);
    Some(cesr::to_text(&bytes))
}

pub fn count_code(code: char, quadlets: usize) -> String {
    if quadlets < 4096 {
        format!("-{code}{}", cesr::b64_chars(quadlets as u64, 2))
    } else {
        format!("--{code}{}", cesr::b64_chars(quadlets as u64, 5))
    }
}

/// Encode variable-length data (`4B##`/`5B##`/`6B##` or the long forms).
pub fn var_field(code: char, data: &[u8]) -> String {
    let lead = (3 - data.len() % 3) % 3;
    let mut raw = vec![0u8; lead];
    raw.extend_from_slice(data);
    let quadlets = raw.len() / 3;
    let head = if quadlets < 4096 {
        format!("{}{code}{}", (b'4' + lead as u8) as char, cesr::b64_chars(quadlets as u64, 2))
    } else {
        format!("{}AA{code}{}", (b'7' + lead as u8) as char, cesr::b64_chars(quadlets as u64, 4))
    };
    format!("{head}{}", cesr::to_text(&raw))
}

/// Rebuild a signed-only message around a new cleartext payload (`-Z...`),
/// recomputing the `-E` count and keeping the attachment (to be re-signed).
pub fn with_payload(text: &str, map: &MessageMap, payload: &str) -> String {
    let start = map.cleartext_payload.expect("signed-only");
    let head = &text[map.frame_code.1..start];
    let attachment = &text[map.content_end..];
    let content = format!("{head}{payload}");
    format!("{}{content}{attachment}", count_code('E', content.len() / 4))
}

/// Rebuild a `-Z` payload from its type code and body fields.
pub fn payload(body: &str) -> String {
    format!("{}{body}", count_code('Z', body.len() / 4))
}

/// The cleartext payload text of a signed-only message, without its `-Z` code.
pub fn payload_body<'a>(text: &'a str, map: &MessageMap) -> Option<&'a str> {
    let start = map.cleartext_payload?;
    let mut r = cesr::Reader::new(text);
    r.pos = start;
    r.count('Z').ok()?;
    Some(&text[r.pos..map.content_end])
}
