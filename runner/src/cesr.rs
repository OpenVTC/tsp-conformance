//! A deliberately small, text-domain CESR walker.
//!
//! This is test-oracle code, not a TSP implementation. It exists for two jobs:
//!
//! 1. Read the *expected* field values out of Appendix A's printed `payload`
//!    and `innerPayload` strings, so the vectors suite compares what an
//!    implementation decodes against what the specification printed, rather
//!    than against another implementation.
//! 2. Locate the regions of a message (envelope VIDs, ciphertext, signature,
//!    count codes) so the negative suite can corrupt exactly one of them.
//!
//! It never decrypts, verifies or derives anything. The text domain is used
//! because every TSP frame is 24-bit aligned, so qb64 characters map onto
//! binary bytes 4:3 and "flip one character" is "corrupt one 6-bit group".

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn b64_index(c: u8) -> Option<u32> {
    B64.iter().position(|&x| x == c).map(|p| p as u32)
}

pub fn b64_value(s: &str) -> Option<u64> {
    s.bytes()
        .try_fold(0u64, |acc, c| Some(acc * 64 + b64_index(c)? as u64))
}

pub fn to_text(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn from_text(text: &str) -> Result<Vec<u8>, String> {
    URL_SAFE_NO_PAD
        .decode(text)
        .map_err(|e| format!("not qb64: {e}"))
}

/// A cursor over qb64 text.
pub struct Reader<'a> {
    pub t: &'a str,
    pub pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub start: usize,
    /// Start of the data characters (after the code and count).
    pub data_start: usize,
    pub end: usize,
    pub data: Vec<u8>,
}

impl<'a> Reader<'a> {
    pub fn new(t: &'a str) -> Self {
        Reader { t, pos: 0 }
    }

    pub fn rest(&self) -> &'a str {
        &self.t[self.pos..]
    }

    fn take(&mut self, n: usize) -> Result<&'a str, String> {
        let s = self
            .t
            .get(self.pos..self.pos + n)
            .ok_or_else(|| format!("truncated at {}", self.pos))?;
        self.pos += n;
        Ok(s)
    }

    /// A group count code for `code` (e.g. `Z`, `E`, `J`, `A`): short `-X##`
    /// or long `--X#####`. Returns the quadlet count.
    pub fn count(&mut self, code: char) -> Result<usize, String> {
        let r = self.rest();
        if r.starts_with(&format!("--{code}")) {
            self.take(3)?;
            Ok(b64_value(self.take(5)?).unwrap() as usize)
        } else if r.starts_with(&format!("-{code}")) {
            self.take(2)?;
            Ok(b64_value(self.take(2)?).ok_or("bad count")? as usize)
        } else {
            Err(format!(
                "expected -{code} count at {}, found {:?}",
                self.pos,
                &r[..r.len().min(8)]
            ))
        }
    }

    /// Variable-length data with a one-letter code (`B` bytes/VID, `F` HPKE
    /// ciphertext, `C` sealed box): `4X##`/`5X##`/`6X##` or `7AAX####`...
    pub fn var(&mut self, code: char) -> Result<Field, String> {
        let start = self.pos;
        let r = self.rest();
        let sel = r.as_bytes().first().copied().ok_or("truncated")?;
        let (lead, quadlets) = match sel {
            b'4' | b'5' | b'6' if r.as_bytes().get(1) == Some(&(code as u8)) => {
                self.take(2)?;
                ((sel - b'4') as usize, b64_value(self.take(2)?).unwrap() as usize)
            }
            b'7' | b'8' | b'9' if r.get(1..4) == Some(&format!("AA{code}")) => {
                self.take(4)?;
                ((sel - b'7') as usize, b64_value(self.take(4)?).unwrap() as usize)
            }
            _ => {
                return Err(format!(
                    "expected {code} variable data at {start}, found {:?}",
                    &r[..r.len().min(8)]
                ));
            }
        };
        let data_start = self.pos;
        let text = self.take(quadlets * 4)?;
        let raw = from_text(text)?;
        let data = raw.get(lead..).ok_or("lead exceeds data")?.to_vec();
        Ok(Field {
            start,
            data_start,
            end: self.pos,
            data,
        })
    }

    /// 32-byte digest: `I` (SHA2-256) or `F` (Blake2b-256) + 43 characters.
    pub fn digest(&mut self) -> Result<(Vec<u8>, &'static str), String> {
        let s = self.take(44)?;
        let alg = match &s[..1] {
            "I" => "sha2-256",
            "F" => "blake2b-256",
            other => return Err(format!("unknown digest code {other}")),
        };
        let raw = from_text(&format!("A{}", &s[1..]))?;
        Ok((raw[1..].to_vec(), alg))
    }

    /// 16-byte nonce: `0A` + 22 characters.
    pub fn nonce(&mut self) -> Result<Vec<u8>, String> {
        let s = self.take(24)?;
        if !s.starts_with("0A") {
            return Err(format!("expected 0A nonce, found {s}"));
        }
        let raw = from_text(&format!("AA{}", &s[2..]))?;
        Ok(raw[2..].to_vec())
    }

    pub fn code4(&mut self) -> Result<&'a str, String> {
        self.take(4)
    }

    pub fn vid_list(&mut self) -> Result<Vec<String>, String> {
        let q = self.count('J')?;
        let end = self.pos + q * 4;
        let mut out = vec![];
        while self.pos < end {
            out.push(String::from_utf8_lossy(&self.var('B')?.data).into_owned());
        }
        Ok(out)
    }
}

/// Everything the vectors suite needs from a printed payload.
#[derive(Debug, Clone, Default)]
pub struct DecodedPayload {
    pub kind: String,
    pub payload_sender: Option<String>,
    pub padding: Vec<u8>,
    pub data: Option<Vec<u8>>,
    pub digest: Option<Vec<u8>>,
    pub reply_digest: Option<Vec<u8>>,
    pub digest_alg: Option<&'static str>,
    pub nonce: Option<Vec<u8>>,
    pub reply_path: Option<Vec<String>>,
    pub referral_vid: Option<Option<String>>,
    pub hops: Option<Vec<String>>,
    pub inner_text: Option<String>,
}

/// Decode a payload printed in qb64 (`-Z## XXXX ...`).
pub fn decode_payload(text: &str) -> Result<DecodedPayload, String> {
    let mut r = Reader::new(text);
    let q = r.count('Z')?;
    let end = r.pos + q * 4;
    if end != text.len() {
        return Err(format!("-Z count {q} does not cover the payload"));
    }
    let ty = r.code4()?;
    let mut p = DecodedPayload::default();
    let sender = r.var('B')?.data;
    p.payload_sender = (!sender.is_empty()).then(|| String::from_utf8_lossy(&sender).into_owned());
    let stream = |r: &mut Reader| -> Result<Vec<u8>, String> {
        r.count('A')?;
        Ok(r.var('B')?.data)
    };
    match ty {
        "XSCS" | "XCTL" => {
            p.kind = if ty == "XSCS" { "scs" } else { "ctl" }.into();
            p.padding = r.var('B')?.data;
            p.data = Some(stream(&mut r)?);
        }
        "XPAD" => {
            p.kind = "pad".into();
            p.nonce = Some(r.nonce()?);
            p.padding = r.var('B')?.data;
        }
        "XHOP" => {
            p.kind = "hop".into();
            p.hops = Some(r.vid_list()?);
            p.padding = r.var('B')?.data;
            p.inner_text = Some(r.rest().to_string());
            r.pos = text.len();
        }
        "XRFI" => {
            p.kind = "rfi".into();
            let (d, alg) = r.digest()?;
            p.digest = Some(d);
            p.digest_alg = Some(alg);
            p.nonce = Some(r.nonce()?);
            p.reply_path = Some(r.vid_list()?);
            let q = r.count('J')?;
            if q == 0 {
                p.referral_vid = Some(None);
            } else {
                let group_end = r.pos + q * 4;
                let vid = r.var('B')?.data;
                p.referral_vid = Some(Some(String::from_utf8_lossy(&vid).into_owned()));
                r.pos = group_end;
            }
            p.padding = r.var('B')?.data;
        }
        "XRFA" => {
            p.kind = "rfa".into();
            let (d, alg) = r.digest()?;
            let (rd, _) = r.digest()?;
            p.digest = Some(d);
            p.reply_digest = Some(rd);
            p.digest_alg = Some(alg);
            p.padding = r.var('B')?.data;
        }
        "XRFD" => {
            p.kind = "rfd".into();
            let (d, alg) = r.digest()?;
            p.digest = Some(d);
            p.digest_alg = Some(alg);
            p.padding = r.var('B')?.data;
        }
        other => return Err(format!("unknown payload type {other}")),
    }
    if r.pos != text.len() {
        return Err(format!("{} characters left over in payload", text.len() - r.pos));
    }
    Ok(p)
}

/// The regions of a whole message, in qb64 character offsets.
#[derive(Debug, Clone)]
pub struct MessageMap {
    /// The `-E` count code, `(start, end)`.
    pub frame_code: (usize, usize),
    pub frame_quadlets: usize,
    /// `YTSP-AAC`: the `-AAC` part is `(version_start, version_start + 4)`.
    pub version_start: usize,
    pub sender: Field,
    #[allow(dead_code)]
    pub receiver: Field,
    /// Ciphertext field (HPKE `F` or sealed box `C`), if confidential.
    pub ciphertext: Option<Field>,
    /// Start of a cleartext `-Z` payload, if signed-only.
    pub cleartext_payload: Option<usize>,
    /// Where the signable content ends / the signature attachment begins.
    pub content_end: usize,
    pub sender_vid: String,
    pub receiver_vid: String,
}

pub fn map_message(text: &str) -> Result<MessageMap, String> {
    let mut r = Reader::new(text);
    let q = r.count('E')?;
    let frame_code = (0, r.pos);
    let content_end = r.pos + q * 4;
    if !r.rest().starts_with("YTSP") {
        return Err("no YTSP marker".into());
    }
    r.pos += 4;
    let version_start = r.pos;
    r.pos += 4;
    let sender = r.var('B')?;
    let receiver = r.var('B')?;
    let (ciphertext, cleartext_payload) = if r.rest().starts_with("-Z") || r.rest().starts_with("--Z") {
        (None, Some(r.pos))
    } else if let Ok(f) = r.var('F') {
        (Some(f), None)
    } else {
        (Some(r.var('C')?), None)
    };
    Ok(MessageMap {
        frame_code,
        frame_quadlets: q,
        version_start,
        sender_vid: String::from_utf8_lossy(&sender.data).into_owned(),
        receiver_vid: String::from_utf8_lossy(&receiver.data).into_owned(),
        sender,
        receiver,
        ciphertext,
        cleartext_payload,
        content_end,
    })
}

/// Replace the character at `at` with a different base64url character.
pub fn flip_char(text: &str, at: usize) -> String {
    let mut b = text.as_bytes().to_vec();
    let c = b[at];
    let i = b64_index(c).unwrap_or(0);
    b[at] = B64[((i + 1) % 64) as usize];
    String::from_utf8(b).unwrap()
}

/// Encode a count as `n` base64 characters.
pub fn b64_chars(mut v: u64, n: usize) -> String {
    let mut out = vec![b'A'; n];
    for i in (0..n).rev() {
        out[i] = B64[(v % 64) as usize];
        v /= 64;
    }
    String::from_utf8(out).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_rfi_vector_payload() {
        let p = decode_payload(
            "-ZAWXRFI4BAAIG6HKhYGieW7r7cADGj6gJ0aMB0rNFf6IyDgK_u9jFE60AARERERERERERERERERERER-JAA-JAA4BAA",
        )
        .unwrap();
        assert_eq!(p.kind, "rfi");
        assert_eq!(p.nonce.unwrap(), vec![0x11; 16]);
        assert_eq!(p.digest_alg, Some("sha2-256"));
        assert_eq!(p.reply_path.unwrap().len(), 0);
    }

    #[test]
    fn decodes_the_scs_vector_payload() {
        let p = decode_payload("-ZAJXSCS4BAA4BAA-AAF5BAEAGhlbGxvIHdvcmxk").unwrap();
        assert_eq!(p.data.unwrap(), b"hello world");
        assert!(p.payload_sender.is_none());
    }
}
