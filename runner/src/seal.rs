//! Build a confidential (HPKE-Base) `XSCS` message from scratch.
//!
//! The negative suite needs one probe no driver can produce: a 1.0 message
//! whose confidential payload carries the NULL VID in its ESSR sender field.
//! 1.0 forbids it (§3.2, §3.7 step 7), so every conformant packer refuses to
//! write one. The runner therefore seals it itself, with RFC 9180 HPKE-Base —
//! a crypto primitive, not a TSP implementation: the framing below is the same
//! hand-built qb64 the re-signing tool already uses, and `aad`/`info` are
//! §8.2.2's.
//!
//! Every use pairs the probe with a control sealed the same way but carrying
//! the sender's VID. A receiver that cannot open the control says nothing
//! about the probe, and one that can has also confirmed the runner seals
//! correctly.

use crate::cesr;
use crate::model::{Ident, SPEC_MARKER, unb64};
use crate::resign::{count_code, payload, resign, var_field};
use hpke::aead::ChaCha20Poly1305;
use hpke::kdf::HkdfSha256;
use hpke::kem::X25519HkdfSha256;
use hpke::{Deserializable, Kem, OpModeS, Serializable};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

/// The HPKE `info` (§8.2.2): the five ASCII characters `YTSP-`.
const INFO: &[u8] = b"YTSP-";

/// Seal `data` from `sender` to `receiver` as a direct `XSCS` message whose
/// ESSR sender field is `payload_sender` (`""` for the NULL VID), and sign it
/// with `sender`'s key. `seed` fixes the HPKE ephemeral.
pub fn seal_scs(sender: &Ident, receiver: &Ident, payload_sender: &str, data: &[u8], seed: [u8; 32]) -> Option<String> {
    let fields = format!(
        "{SPEC_MARKER}{}{}",
        var_field('B', sender.id.as_bytes()),
        var_field('B', receiver.id.as_bytes())
    );
    let stream = var_field('B', data);
    let body = format!(
        "XSCS{}{}{}{stream}",
        var_field('B', payload_sender.as_bytes()),
        var_field('B', b""),
        count_code('A', stream.len() / 4)
    );
    let plaintext = cesr::from_text(&payload(&body)).ok()?;
    let aad = cesr::from_text(&fields).ok()?;

    let pk_bytes = unb64(&receiver.pk_e);
    let pk = <X25519HkdfSha256 as Kem>::PublicKey::from_bytes(&pk_bytes).ok()?;
    let mut rng = ChaCha20Rng::from_seed(seed);
    let (enc, ct) = hpke::single_shot_seal::<ChaCha20Poly1305, HkdfSha256, X25519HkdfSha256, _>(
        &OpModeS::Base,
        &pk,
        INFO,
        &plaintext,
        &aad,
        &mut rng,
    )
    .ok()?;
    let mut sealed = enc.to_bytes().to_vec();
    sealed.extend_from_slice(&ct);

    let content = format!("{fields}{}", var_field('F', &sealed));
    // `-C23 -K22` then the indexed Ed25519 code `B` (index 0) and 64 bytes;
    // the signature itself is filled in by `resign`.
    let mut sig_field = vec![0x04, 0x00];
    sig_field.extend_from_slice(&[0u8; 64]);
    let text = format!(
        "{}{content}{}{}{}",
        count_code('E', content.len() / 4),
        count_code('C', 23),
        count_code('K', 22),
        cesr::to_text(&sig_field)
    );
    resign(&text, sender)
}
