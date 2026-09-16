//! Conformance driver for the ToIP reference implementation, `tsp_sdk` 0.11.0.
//!
//! Stateless ops use `tsp_sdk::crypto::{seal_*, open, sign, verify}`; padding
//! is reachable only through `SecureStore` (`SendOptions::padding`), so a pack
//! that asks for padding goes through a throwaway store holding just the two
//! VIDs. Endpoint ops use one `SecureStore` per endpoint.

mod proto;

use proto::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use tsp_sdk::cesr::{CryptoType, EnvelopeType};
use tsp_sdk::crypto::CryptoError;
use tsp_sdk::definitions::{Digest, Payload, RelationshipForm};
use tsp_sdk::{Error, OwnedVid, ReceivedRelationshipForm, ReceivedTspMessage, RelationshipStatus, SecureStore, SendOptions, Vid};

const TRANSPORT: &str = "tcp://127.0.0.1:1";

const CAPABILITIES: &[&str] = &[
    "hpke-base",
    "sealed-box",
    "hpke-pq",
    // crypto::sign / crypto::verify carry an application payload only
    "signed-only",
    "payload.scs",
    "payload.ctl",
    "payload.pad",
    "payload.rfi",
    "payload.rfa",
    "payload.rfd",
    "rfi.reply-path",
    // Signature_new is built by crate-private code reachable only through
    // SecureStore::make_parallel_relationship_request
    "rfi.referral:open",
    "payload.hop",
    "padding",
    "peek",
    "endpoint",
];

fn crypto_code(e: &CryptoError) -> &'static str {
    use tsp_sdk::cesr::error::DecodeError as D;
    match e {
        CryptoError::Decode(D::VersionMismatch) => "version",
        CryptoError::Decode(_) | CryptoError::Encode(_) | CryptoError::MissingCiphertext => {
            "malformed"
        }
        CryptoError::CryptographicHpke(_) | CryptoError::CryptographicNacl(_) => "decrypt",
        CryptoError::Verify(..) => "signature",
        CryptoError::UnexpectedRecipient => "receiver",
        CryptoError::UnexpectedSender | CryptoError::MissingSender => "sender",
        CryptoError::DigestMismatch => "digest",
        CryptoError::UnsupportedPayload => "unsupported",
        CryptoError::Key(_) => "invalid-input",
        _ => "internal",
    }
}

fn cerr(e: CryptoError) -> DErr {
    err(crypto_code(&e), e.to_string())
}

fn serr(e: Error) -> DErr {
    let code = match &e {
        Error::Crypto(c) => crypto_code(c),
        Error::Decode(tsp_sdk::cesr::error::DecodeError::VersionMismatch) => "version",
        Error::Decode(_) | Error::Encode(_) => "malformed",
        Error::Relationship(_) | Error::UnestablishedRelationship(..) => "relationship",
        Error::UnverifiedVid(_) | Error::MissingVid(_) | Error::MissingPrivateVid(_) => {
            "invalid-input"
        }
        _ => "internal",
    };
    err(code, e.to_string())
}

// ---------------------------------------------------------------- identities

fn key_types(id: &Identity) -> DResult<(&'static str, &'static str)> {
    let s = match id.sig_key_type.as_str() {
        "Ed25519" => "Ed25519",
        "MlDsa65" => "MlDsa65",
        o => return Err(unsupported(format!("signature key type {o}"))),
    };
    let e = match id.enc_key_type.as_str() {
        "X25519" => "X25519",
        "MLKEM768-X25519" => "MLKEM768-X25519",
        o => return Err(unsupported(format!("encryption key type {o}"))),
    };
    Ok((s, e))
}

/// Build the reference's VID types through their serde form, which is how the
/// crate itself loads identities from files.
fn owned_vid(id: &Identity) -> DResult<OwnedVid> {
    let (s, e) = key_types(id)?;
    let v = json!({
        "id": id.id, "transport": TRANSPORT,
        "sigKeyType": s, "encKeyType": e,
        "publicSigkey": b64e(&id.pk_s), "publicEnckey": b64e(&id.pk_e),
        "sigkey": b64e(id.sk_s()?), "enckey": b64e(id.sk_e()?),
    });
    serde_json::from_str(&v.to_string()).map_err(|e| err("invalid-input", format!("OwnedVid: {e}")))
}

fn public_vid(id: &Identity) -> DResult<Vid> {
    let (s, e) = key_types(id)?;
    let v = json!({
        "id": id.id, "transport": TRANSPORT,
        "sigKeyType": s, "encKeyType": e,
        "publicSigkey": b64e(&id.pk_s), "publicEnckey": b64e(&id.pk_e),
    });
    serde_json::from_str(&v.to_string()).map_err(|e| err("invalid-input", format!("Vid: {e}")))
}

fn random32() -> [u8; 32] {
    let mut b = [0u8; 32];
    getrandom::getrandom(&mut b).expect("OS randomness");
    b
}

// ---------------------------------------------------------------- pack

fn digest_alg(ct: CryptoType) -> &'static str {
    match ct {
        CryptoType::SealedBox => "blake2b-256",
        _ => "sha2-256",
    }
}

fn op_pack(req: &Value) -> DResult<Value> {
    if !matches!(req.get("ephemeral"), None | Some(Value::Null)) {
        return Err(unsupported(
            "tsp_sdk reproduces a message from an RNG seed (seal_reproducibly), not from ikmE/skEm",
        ));
    }
    let scheme = str_field(req, "scheme")?;
    let sender = Identity::field(req, "sender")?;
    let receiver = Identity::field(req, "receiver")?;
    let payload = req
        .get("payload")
        .ok_or_else(|| err("invalid-input", "missing payload"))?;
    let ty = str_field(payload, "type")?;
    let padding = opt_bytes(payload, "padding")?.unwrap_or_default();

    let pq = sender.is_pq() || receiver.is_pq();
    let crypto_type = match scheme {
        "hpke-base" if !pq => Some(CryptoType::HpkeBase),
        "hpke-pq" if pq => Some(CryptoType::HpkeBase),
        "sealed-box" => Some(CryptoType::SealedBox),
        "signed-only" => None,
        other => {
            return Err(unsupported(format!(
                "scheme {other} with these key types"
            )));
        }
    };

    // ESSR sender field: NULL under HPKE-Base and signed-only, the sender under
    // the sealed box. The public API offers no choice (EssrSender is only
    // reachable through a crate-private selection).
    if let Some(ps) = payload.get("payloadSender") {
        let lib_default_null = crypto_type != Some(CryptoType::SealedBox);
        let requested_null = ps.is_null();
        if requested_null != lib_default_null
            || ps.as_str().is_some_and(|s| s != sender.id)
        {
            return Err(unsupported(
                "tsp_sdk chooses the ESSR sender field by scheme (NULL under HPKE-Base and signed-only)",
            ));
        }
    }

    let sender_vid = owned_vid(&sender)?;
    let receiver_vid = public_vid(&receiver)?;

    // --- signed-only: crypto::sign takes an application payload only
    let Some(ct) = crypto_type else {
        if ty != "scs" {
            return Err(unsupported("crypto::sign signs an application payload only"));
        }
        if !padding.is_empty() {
            return Err(unsupported("crypto::sign has no padding option"));
        }
        let data = bytes_field(payload, "data")?;
        let msg = tsp_sdk::crypto::sign(&sender_vid, Some(&receiver_vid), &data).map_err(cerr)?;
        return Ok(json!({"message": b64e(&msg), "digest": null, "digestAlg": null}));
    };

    // --- padding: only SecureStore's SendOptions carries it, for scs/ctl/pad
    if !padding.is_empty() {
        let store = SecureStore::new();
        store.add_private_vid(sender_vid, None).map_err(serr)?;
        store.add_verified_vid(receiver_vid, None).map_err(serr)?;
        let options = SendOptions {
            crypto_type: Some(ct),
            padding: Some(&padding),
            ..Default::default()
        };
        let (s, r) = (sender.id.as_str(), receiver.id.as_str());
        let (_, msg) = match ty {
            "scs" => store.seal_message_with(s, r, &bytes_field(payload, "data")?, options),
            "ctl" => store.seal_control_message(s, r, &bytes_field(payload, "data")?, options),
            "pad" => {
                if opt_bytes(payload, "nonce")?.is_some() {
                    return Err(unsupported("a padding message's nonce is drawn by the library"));
                }
                store.seal_padding_message(s, r, options)
            }
            _ => {
                return Err(unsupported(
                    "padding is reachable only for scs/ctl/pad (SecureStore SendOptions)",
                ));
            }
        }
        .map_err(serr)?;
        return Ok(json!({"message": b64e(&msg), "digest": null, "digestAlg": null}));
    }

    let mut digest: Digest = [0; 32];
    let mut returns_digest = false;
    let data;
    let digest_in;
    let hops_owned: Vec<String>;
    let reply_owned: Vec<String>;
    let mut nonce: Option<[u8; 16]> = None;
    let body: Payload<&[u8]> = match ty {
        "scs" => {
            data = bytes_field(payload, "data")?;
            Payload::Content(&data)
        }
        "ctl" => {
            data = bytes_field(payload, "data")?;
            Payload::ControlMessage(&data)
        }
        "pad" => {
            if opt_bytes(payload, "nonce")?.is_some() {
                return Err(unsupported("a padding message's nonce is drawn by the library"));
            }
            Payload::Padding
        }
        "rfi" => {
            if !matches!(payload.get("referral"), None | Some(Value::Null)) {
                return Err(unsupported(
                    "Signature_new is made by crate-private code (SecureStore::make_parallel_relationship_request only)",
                ));
            }
            if let Some(n) = opt_bytes(payload, "nonce")? {
                nonce = Some(fixed(&n, "nonce")?);
            }
            reply_owned = payload
                .get("replyPath")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            returns_digest = true;
            Payload::RequestRelationship {
                thread_id: [0; 32],
                form: RelationshipForm::Direct,
                reply_path: reply_owned.iter().map(|s| s.as_bytes()).collect(),
            }
        }
        "rfa" => {
            digest_in = fixed::<32>(&bytes_field(payload, "digest")?, "digest")?;
            returns_digest = true;
            Payload::AcceptRelationship {
                thread_id: digest_in,
                reply_thread_id: [0; 32],
                form: RelationshipForm::Direct,
            }
        }
        "rfd" => {
            digest_in = fixed::<32>(&bytes_field(payload, "digest")?, "digest")?;
            if let Some(alg) = payload.get("digestAlg").and_then(Value::as_str)
                && alg != digest_alg(ct)
            {
                return Err(unsupported("tsp_sdk codes the digest by crypto type"));
            }
            Payload::CancelRelationship {
                thread_id: digest_in,
            }
        }
        "hop" => {
            data = bytes_field(payload, "inner")?;
            hops_owned = payload
                .get("hops")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            if hops_owned.is_empty() {
                Payload::NestedMessage(&data)
            } else {
                Payload::RoutedMessage(hops_owned.iter().map(|s| s.as_bytes()).collect(), &data)
            }
        }
        other => return Err(unsupported(format!("payload type {other}"))),
    };

    let msg = if nonce.is_some() {
        // the only public entry that accepts a caller nonce; the seed is fresh
        tsp_sdk::crypto::seal_reproducibly(
            &sender_vid,
            &receiver_vid,
            body,
            Some(&mut digest),
            ct,
            random32(),
            nonce,
        )
    } else {
        tsp_sdk::crypto::seal_and_hash_with_crypto_type(
            &sender_vid,
            &receiver_vid,
            body,
            Some(&mut digest),
            ct,
        )
    }
    .map_err(cerr)?;

    Ok(if returns_digest {
        json!({"message": b64e(&msg), "digest": b64e(&digest), "digestAlg": digest_alg(ct)})
    } else {
        json!({"message": b64e(&msg), "digest": null, "digestAlg": null})
    })
}

// ---------------------------------------------------------------- open

fn envelope_parties(bytes: &[u8]) -> DResult<(String, Option<String>)> {
    let (s, r) = tsp_sdk::cesr::get_sender_receiver(bytes)
        .map_err(|e| err("malformed", e.to_string()))?;
    Ok((
        String::from_utf8_lossy(s).into_owned(),
        r.map(|r| String::from_utf8_lossy(r).into_owned()),
    ))
}

fn hop_strings(hops: &[&[u8]]) -> Vec<String> {
    hops.iter().map(|h| String::from_utf8_lossy(h).into_owned()).collect()
}

fn op_open(req: &Value) -> DResult<Value> {
    let receiver = Identity::field(req, "receiver")?;
    let sender = Identity::field(req, "sender")?;
    let bytes = bytes_field(req, "message")?;
    let mut probe_buf = bytes.clone();
    let signed_only = matches!(
        tsp_sdk::cesr::probe(&mut probe_buf).map_err(|e| err(
            if matches!(e, tsp_sdk::cesr::error::DecodeError::VersionMismatch) { "version" } else { "malformed" },
            e.to_string()
        ))?,
        EnvelopeType::SignedMessage { .. }
    );
    let sender_vid = public_vid(&sender)?;

    if signed_only {
        let mut buf = bytes.clone();
        let (data, _mt) = tsp_sdk::crypto::verify(&sender_vid, &mut buf).map_err(cerr)?;
        let (s, r) = envelope_parties(&bytes)?;
        return Ok(json!({
            "envelopeSender": s,
            "envelopeReceiver": r,
            "scheme": "signed-only",
            "payload": {"type": "scs", "data": b64e(data)},
        }));
    }

    let receiver_vid = owned_vid(&receiver)?;
    let mut buf = bytes.clone();
    let (payload, ct, _st) =
        tsp_sdk::crypto::open(&receiver_vid, &sender_vid, &mut buf).map_err(cerr)?;
    let scheme = match ct {
        CryptoType::SealedBox => "sealed-box",
        CryptoType::HpkeBase if receiver.enc_key_type == "MLKEM768-X25519" => "hpke-pq",
        CryptoType::HpkeBase => "hpke-base",
        CryptoType::Plaintext => "signed-only",
    };
    let alg = digest_alg(ct);
    let p = match payload {
        Payload::Content(d) => json!({"type": "scs", "data": b64e(d)}),
        Payload::ControlMessage(d) => json!({"type": "ctl", "data": b64e(d)}),
        // the nonce is decoded and deliberately not surfaced (crypto/mod.rs)
        Payload::Padding => json!({"type": "pad"}),
        Payload::NestedMessage(inner) => json!({"type": "hop", "hops": [], "inner": b64e(inner)}),
        Payload::RoutedMessage(hops, inner) => {
            json!({"type": "hop", "hops": hop_strings(&hops), "inner": b64e(inner)})
        }
        // the nonce is not surfaced by crypto::open
        Payload::RequestRelationship {
            thread_id,
            form,
            reply_path,
        } => json!({
            "type": "rfi",
            "digest": b64e(&thread_id),
            "digestAlg": alg,
            "replyPath": hop_strings(&reply_path),
            "referral": match form {
                RelationshipForm::Direct => Value::Null,
                RelationshipForm::Parallel { new_vid, sig_new_vid } => json!({
                    "vid": String::from_utf8_lossy(new_vid),
                    "signature": b64e(sig_new_vid),
                }),
            },
        }),
        Payload::AcceptRelationship {
            thread_id,
            reply_thread_id,
            ..
        } => json!({
            "type": "rfa",
            "digest": b64e(&thread_id),
            "replyDigest": b64e(&reply_thread_id),
            "digestAlg": alg,
        }),
        Payload::CancelRelationship { thread_id } => {
            json!({"type": "rfd", "digest": b64e(&thread_id), "digestAlg": alg})
        }
    };
    let (s, r) = envelope_parties(&bytes)?;
    Ok(json!({
        "envelopeSender": s,
        "envelopeReceiver": r,
        "scheme": scheme,
        "payload": p,
    }))
}

fn op_peek(req: &Value) -> DResult<Value> {
    let mut bytes = bytes_field(req, "message")?;
    let confidential = matches!(
        tsp_sdk::cesr::probe(&mut bytes).map_err(|e| err("malformed", e.to_string()))?,
        EnvelopeType::EncryptedMessage { .. }
    );
    let (s, r) = envelope_parties(&bytes)?;
    Ok(json!({"envelopeSender": s, "envelopeReceiver": r, "confidential": confidential}))
}

// ---------------------------------------------------------------- endpoint

struct Endpoint {
    store: SecureStore,
}

#[derive(Default)]
struct State {
    endpoints: HashMap<String, Endpoint>,
    next: u64,
}

fn endpoint<'a>(st: &'a State, req: &Value) -> DResult<&'a Endpoint> {
    let h = str_field(req, "endpoint")?;
    st.endpoints
        .get(h)
        .ok_or_else(|| err("invalid-input", format!("unknown endpoint {h}")))
}

/// Map the reference's relationship status to the protocol's.
///
/// `SecureStore` records each side's own-direction digest as `thread_id` and
/// the other as `remote_thread_id`: the inviter holds (invite, accept) and the
/// accepter (accept, invite). Which side this endpoint was is read from the
/// digest the invite recorded while the relationship was forming.
fn state_json(status: RelationshipStatus, invite: Option<&Digest>) -> Value {
    match status {
        RelationshipStatus::Unrelated => json!({"state": "none"}),
        RelationshipStatus::Unidirectional { thread_id } => {
            json!({"state": "invite-sent", "digest": b64e(&thread_id)})
        }
        RelationshipStatus::ReverseUnidirectional { thread_id } => {
            json!({"state": "invite-received", "digest": b64e(&thread_id)})
        }
        RelationshipStatus::Bidirectional {
            thread_id,
            remote_thread_id,
            ..
        } => {
            let (d, r) = if invite == Some(&remote_thread_id) {
                (remote_thread_id, thread_id)
            } else {
                (thread_id, remote_thread_id)
            };
            json!({"state": "bidirectional", "digest": b64e(&d), "replyDigest": b64e(&r)})
        }
    }
}

fn op_endpoint(st: &mut State, invites: &mut HashMap<(String, String, String), Digest>, op: &str, req: &Value) -> DResult<Value> {
    match op {
        "endpoint.create" => {
            let store = SecureStore::new();
            for v in req.get("identities").and_then(Value::as_array).into_iter().flatten() {
                store
                    .add_private_vid(owned_vid(&Identity::parse(v)?)?, None)
                    .map_err(serr)?;
            }
            for v in req.get("peers").and_then(Value::as_array).into_iter().flatten() {
                store
                    .add_verified_vid(public_vid(&Identity::parse(v)?)?, None)
                    .map_err(serr)?;
            }
            st.next += 1;
            let h = format!("ep-{}", st.next);
            st.endpoints.insert(h.clone(), Endpoint { store });
            Ok(json!({"endpoint": h}))
        }
        "endpoint.invite" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let ep = endpoint(st, req)?;
            let (_, msg) = ep.store.make_relationship_request(from, to, None).map_err(serr)?;
            let status = ep.store.relation_status_for_vid_pair(from, to).map_err(serr)?;
            let RelationshipStatus::Unidirectional { thread_id } = status else {
                return Err(err("internal", format!("after invite, status is {status}")));
            };
            invites.insert((str_field(req, "endpoint")?.into(), from.into(), to.into()), thread_id);
            Ok(json!({"message": b64e(&msg), "digest": b64e(&thread_id)}))
        }
        "endpoint.accept" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let ep = endpoint(st, req)?;
            let status = ep.store.relation_status_for_vid_pair(from, to).map_err(serr)?;
            let RelationshipStatus::ReverseUnidirectional { thread_id } = status else {
                return Err(err("relationship", format!("no invite to accept; status is {status}")));
            };
            let (_, msg) = ep
                .store
                .make_relationship_accept(from, to, thread_id, None)
                .map_err(serr)?;
            let reply = match ep.store.relation_status_for_vid_pair(from, to).map_err(serr)? {
                RelationshipStatus::Bidirectional { thread_id: own, .. } => own,
                other => return Err(err("internal", format!("after accept, status is {other}"))),
            };
            invites.insert((str_field(req, "endpoint")?.into(), from.into(), to.into()), thread_id);
            Ok(json!({"message": b64e(&msg), "digest": b64e(&thread_id), "replyDigest": b64e(&reply)}))
        }
        "endpoint.cancel" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let ep = endpoint(st, req)?;
            let (_, msg) = ep.store.make_relationship_cancel(from, to).map_err(serr)?;
            Ok(json!({"message": b64e(&msg)}))
        }
        "endpoint.send" => {
            let (from, to) = (str_field(req, "from")?, str_field(req, "to")?);
            let data = bytes_field(req, "data")?;
            let ep = endpoint(st, req)?;
            let (_, msg) = ep.store.seal_message(from, to, &data).map_err(serr)?;
            Ok(json!({"message": b64e(&msg)}))
        }
        "endpoint.receive" => {
            let handle = str_field(req, "endpoint")?.to_string();
            let mut bytes = bytes_field(req, "message")?;
            let ep = endpoint(st, req)?;
            let received = ep.store.open_message(&mut bytes).map_err(serr)?;
            Ok(match received {
                ReceivedTspMessage::GenericMessage {
                    sender,
                    receiver,
                    message,
                    ..
                } => json!({"event": "message", "from": sender, "to": receiver, "data": b64e(message)}),
                ReceivedTspMessage::RequestRelationship {
                    sender,
                    receiver,
                    thread_id,
                    form,
                    ..
                } => {
                    if !matches!(form, ReceivedRelationshipForm::Direct) {
                        return Err(unsupported("parallel relationship request"));
                    }
                    json!({"event": "invite", "from": sender, "to": receiver, "digest": b64e(&thread_id)})
                }
                ReceivedTspMessage::AcceptRelationship {
                    sender,
                    receiver,
                    thread_id,
                    reply_thread_id,
                    ..
                } => {
                    invites.insert((handle, receiver.clone(), sender.clone()), thread_id);
                    json!({"event": "accept", "from": sender, "to": receiver,
                           "digest": b64e(&thread_id), "replyDigest": b64e(&reply_thread_id)})
                }
                ReceivedTspMessage::CancelRelationship {
                    sender,
                    receiver,
                    thread_id,
                    ..
                } => json!({"event": "cancel", "from": sender, "to": receiver, "digest": b64e(&thread_id)}),
                other => {
                    return Err(unsupported(format!(
                        "received message kind not mapped: {:?}",
                        std::mem::discriminant(&other)
                    )));
                }
            })
        }
        "endpoint.state" => {
            let (local, remote) = (str_field(req, "local")?, str_field(req, "remote")?);
            let ep = endpoint(st, req)?;
            let status = ep.store.relation_status_for_vid_pair(local, remote).map_err(serr)?;
            let invite = invites.get(&(str_field(req, "endpoint")?.into(), local.into(), remote.into()));
            Ok(state_json(status, invite))
        }
        other => Err(unsupported(format!("op {other}"))),
    }
}

fn main() {
    let mut st = State::default();
    let mut invites = HashMap::new();
    proto::run(|op, req| match op {
        "hello" => Ok(json!({
            "name": "tsp_sdk",
            "version": "0.11.0",
            "language": "rust",
            "protocol": 1,
            "capabilities": CAPABILITIES,
        })),
        "pack" => op_pack(req),
        "open" => op_open(req),
        "peek" => op_peek(req),
        op if op.starts_with("endpoint.") => op_endpoint(&mut st, &mut invites, op, req),
        other => Err(unsupported(format!("op {other}"))),
    });
}
