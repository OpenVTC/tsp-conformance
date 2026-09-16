# TSP Rev 3 conformance report

Target **trustoverip/tswg-tsp-specification** at commit `f5b8668` (`YTSP-AAC`). Generated 2026-09-16T11:11:51Z with key seed `20260916`.

Each case is **pass**, **fail** (with the diff), **skip** (a capability the implementation does not offer) or **error** (the harness or a driver broke). Cells read `✓ passed/ran` when nothing failed, `✗ passed/ran` when something did, `–` when every case skipped.

## Implementations

| Driver | Library | Version | Language | Status |
|---|---|---|---|---|
| affinidi-rust | affinidi-tsp | 0.2.0 | rust | ran |
| reference-rust | tsp_sdk | 0.11.0 | rust | ran |
| tsp-js | @openvtc/vti-tsp-js | 0.3.0 | typescript | ran |

## Summary

| Suite | Pass | Fail | Error | Skip |
|---|---:|---:|---:|---:|
| interop | 256 | 13 | 0 | 91 |
| negative | 200 | 9 | 0 | 58 |
| relationship | 75 | 6 | 0 | 0 |
| vectors | 69 | 0 | 0 | 36 |

## Capability matrix

As each driver declares in `hello`. `pack only`/`open only` mark one-sided support.

| Capability | affinidi-rust | reference-rust | tsp-js |
|---|---|---|---|
| `hpke-base` | ✓ | ✓ | ✓ |
| `signed-only` | ✓ | ✓ | — |
| `sealed-box` | ✓ | ✓ | — |
| `hpke-pq` | ✓ | ✓ | — |
| `payload.scs` | ✓ | ✓ | ✓ |
| `payload.ctl` | ✓ | ✓ | open only |
| `payload.pad` | ✓ | ✓ | open only |
| `payload.rfi` | ✓ | ✓ | ✓ |
| `payload.rfa` | ✓ | ✓ | ✓ |
| `payload.rfd` | ✓ | ✓ | ✓ |
| `rfi.reply-path` | ✓ | ✓ | ✓ |
| `rfi.referral` | ✓ | open only | open only |
| `payload.hop` | ✓ | ✓ | ✓ |
| `padding` | ✓ | ✓ | — |
| `payload-sender` | — | — | — |
| `deterministic` | — | — | — |
| `peek` | ✓ | ✓ | ✓ |
| `endpoint` | ✓ | ✓ | ✓ |
| *not reported by `open`* | `payload.padding`, `payload.payloadSender` | `payload.nonce`, `payload.padding`, `payload.payloadSender`, `peek.version`, `version` | `payload.padding`, `payload.payloadSender` |

## Vectors matrix

| Case | affinidi-rust | reference-rust | tsp-js |
|---|:-:|:-:|:-:|
| `control-rfa-direct/open` | ✓ | ✓ | ✓ |
| `control-rfa-direct/pack-exact` | – | – | – |
| `control-rfa-direct/peek` | ✓ | ✓ | ✓ |
| `control-rfd/open` | ✓ | ✓ | ✓ |
| `control-rfd/pack-exact` | – | – | – |
| `control-rfd/peek` | ✓ | ✓ | ✓ |
| `control-rfi-direct/open` | ✓ | ✓ | ✓ |
| `control-rfi-direct/pack-exact` | – | – | – |
| `control-rfi-direct/peek` | ✓ | ✓ | ✓ |
| `control-rfi-sealed-box/open` | ✓ | ✓ | – |
| `control-rfi-sealed-box/pack-exact` | – | – | – |
| `control-rfi-sealed-box/peek` | ✓ | ✓ | ✓ |
| `direct-hpke-base/open` | ✓ | ✓ | ✓ |
| `direct-hpke-base/pack-exact` | – | – | – |
| `direct-hpke-base/peek` | ✓ | ✓ | ✓ |
| `direct-hpke-base-pq/open` | ✓ | ✓ | – |
| `direct-hpke-base-pq/pack-exact` | – | – | – |
| `direct-hpke-base-pq/peek` | ✓ | ✓ | ✓ |
| `direct-sealed-box/open` | ✓ | ✓ | – |
| `direct-sealed-box/pack-exact` | – | – | – |
| `direct-sealed-box/peek` | ✓ | ✓ | ✓ |
| `direct-signed-only/open` | ✓ | ✓ | – |
| `direct-signed-only/pack-exact` | – | ✓ | – |
| `direct-signed-only/peek` | ✓ | ✓ | ✓ |
| `nested-direct/open` | ✓ | ✓ | ✓ |
| `nested-direct/inner-open` | ✓ | ✓ | ✓ |
| `nested-direct/pack-exact` | – | – | – |
| `nested-direct/peek` | ✓ | ✓ | ✓ |
| `routed/open` | ✓ | ✓ | ✓ |
| `routed/inner-open` | ✓ | ✓ | ✓ |
| `routed/pack-exact` | – | – | – |
| `routed/peek` | ✓ | ✓ | ✓ |
| `direct-signed-only/resigned/minor-ABA-accepted` | ✓ | ✓ | – |
| `direct-signed-only/resigned/minor-AAD-accepted` | ✓ | ✓ | – |
| `direct-signed-only/resigned/essr-sender-present-accepted` | ✓ | ✓ | – |

## Interop matrix

| packs ↓ / opens → | affinidi-rust | reference-rust | tsp-js |
|---|---|---|---|
| **affinidi-rust** | ✗ 38/39 | ✗ 38/39 | ✗ 29/31 |
| **reference-rust** | ✗ 35/36 | ✓ 36/36 | ✗ 26/28 |
| **tsp-js** | ✗ 18/20 | ✗ 18/20 | ✗ 18/20 |

<details><summary>Per case (40 cases × 9 pairs)</summary>

| Case | affin→affin | affin→refer | affin→tsp | refer→affin | refer→refer | refer→tsp | tsp→affin | tsp→refer | tsp→tsp |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `scs/hpke-base/small` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/empty` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/2mib` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-0` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-1` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `scs/hpke-base/padding-2` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `scs/hpke-base/padding-3` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `scs/hpke-base/padding-12285` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `scs/hpke-base/padding-12286` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `scs/signed-only` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `scs/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `scs/hpke-base/payload-sender-null` | – | – | – | ✓ | ✓ | ✓ | – | – | – |
| `scs/hpke-base/payload-sender-present` | ✓ | ✓ | ✓ | – | – | – | ✓ | ✓ | ✓ |
| `scs/hpke-base/long-vid` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-pq` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `ctl/hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `pad/hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `pad/hpke-base/padded` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – |
| `rfi/hpke-base/direct` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/reply-path` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/referral` | ✓ | ✓ | ✓ | – | – | – | – | – | – |
| `rfi/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `rfa/hpke-base/echo-own-rfi` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfa/hpke-base/answers-receiver-rfi` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfd/hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/padded` | ✓ | ✓ | ✓ | – | – | – | – | – | – |
| `rfi/hpke-pq` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `rfa/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `rfd/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `hop/nested` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested/padded` | ✓ | ✓ | ✓ | – | – | – | – | – | – |
| `hop/nested/signed-only-inner` | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – |
| `hop/routed-2-hops` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/routed-long-hop-list` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/long-reply-path` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/routed-12-hops` | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `hop/routed-17-hops` | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested-mixed/inner-by-affinidi-rust` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-reference-rust` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-tsp-js` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

</details>

## Negative matrix

Aggregated over the message sources (the implementation's own output and the Appendix A vectors); see the JSON report for each source.

| Case | affinidi-rust | reference-rust | tsp-js |
|---|:-:|:-:|:-:|
| `flip-ciphertext` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 |
| `flip-signature` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `flip-envelope-sender` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `flip-version-minor` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `unknown-major-version` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `truncate-one-triplet` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `truncate-half` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `trailing-bytes` | ✓ 6/6 | ✗ 0/6 | ✓ 3/3 |
| `frame-count-too-large` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `frame-count-too-small` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `signature-index-nonzero` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 |
| `attachment-count-too-large` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 |
| `wrong-receiver-key` | ✓ 4/4 | ✓ 4/4 | ✓ 3/3 |
| `wrong-sender-key` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 |
| `digest-tampered` | ✓ 1/1 | – | – |
| `non-canonical-lead-byte` | ✓ 1/1 | ✗ 0/1 | – |
| `essr-sender-mismatch` | ✓ 1/1 | ✗ 0/1 | – |
| `payload-count-too-large` | ✓ 1/1 | ✓ 1/1 | – |
| `data-after-payload-fields` | ✗ 0/1 | ✓ 1/1 | – |

## Relationship matrix

| A ↓ / B → | affinidi-rust | reference-rust | tsp-js |
|---|---|---|---|
| **affinidi-rust** | ✗ 8/9 | ✓ 9/9 | ✓ 9/9 |
| **reference-rust** | ✗ 8/9 | ✓ 9/9 | ✓ 9/9 |
| **tsp-js** | ✗ 7/9 | ✗ 8/9 | ✗ 8/9 |

<details><summary>Per case (9 cases × 9 pairs)</summary>

| Case | affin→affin | affin→refer | affin→tsp | refer→affin | refer→refer | refer→tsp | tsp→affin | tsp→refer | tsp→tsp |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `invite-accept-bidirectional` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `message-before-relationship-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-returns-to-none` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-inviter-naming-invite-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-inviter-naming-accept-digest` | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ |
| `cancel-by-accepter-naming-invite-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-accepter-naming-accept-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi-race-lower-digest-wins` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `accept-unknown-digest-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |

</details>

## Findings

Failures attributed to an investigated root cause (`findings.toml`). *Kind*: `spec-violation` contradicts a MUST; `disagreement` — implementations differ where the specification is silent or ambiguous; `limit` — a bound the specification does not set; `api-gap` — the library cannot express what the protocol needs.

| # | Finding | Kind | Cases |
|---|---|---|---:|
| 1 | tsp_sdk accepts a primitive whose lead bytes are non-zero | `spec-violation` | 1 |
| 2 | tsp_sdk ignores bytes after the signature attachment; affinidi-tsp and tsp-js reject them | `disagreement` | 6 |
| 3 | tsp_sdk does not check the ESSR sender field of a signed-only message | `disagreement` | 1 |
| 4 | affinidi-tsp ignores data after the last field of an XSCS payload; tsp_sdk rejects it | `disagreement` | 1 |
| 5 | affinidi-tsp, as the accepting endpoint, discards a cancellation that names its own accept's digest | `spec-violation` | 3 |
| 6 | Route length limits differ: tsp-js 10 hops, affinidi-tsp 16, tsp_sdk none | `limit` | 13 |
| 7 | tsp-js's relationship rules accept an RFA that names an invite never sent | `api-gap` | 3 |

### 1. tsp_sdk accepts a primitive whose lead bytes are non-zero

**Kind:** `spec-violation` · **Spec:** §3.7: "A receiver MUST reject a message containing a primitive whose pad bits are non-zero."

The `direct-signed-only` vector's application-data field is `5BAH…`: one lead byte, which must be zero. The harness sets that byte non-zero and re-signs the message with alice's published key, so only a canonicality check can catch it. affinidi-tsp refuses it (`wire.rs` `decode_variable_data_range` rejects non-zero lead bytes, citing §3.7). tsp_sdk 0.11.0 returns the payload: `cesr/decode.rs` `decode_variable_data_index` computes `data_begin = offset + 3` and skips the lead bytes without inspecting them. Because TSP signatures and digests are taken over exact bytes, this admits two byte sequences for one value. Static reading: tsp-js `cesr/wire.ts` `decodeVariableDataRange` has no lead-byte check either, but tsp-js has no signed-only reader, so the case cannot reach it.

Observed in 1 case(s); for example `negative/resigned/direct-signed-only/non-canonical-lead-byte` (vector (re-signed) → reference-rust):

```text
accepted a message with non-canonical-lead-byte (source direct-signed-only (re-signed)); decoded payload type "scs"
```

<details><summary>Affected cases</summary>

- `negative/resigned/direct-signed-only/non-canonical-lead-byte` vector (re-signed) → reference-rust

</details>

### 2. tsp_sdk ignores bytes after the signature attachment; affinidi-tsp and tsp-js reject them

**Kind:** `disagreement` · **Spec:** §9.1 (-E count delimits the signable content), §9.5 (-C count delimits the attachment); no explicit rule for data after the attachment

Appending one zero triplet (`AAAA` in qb64) after a valid message: affinidi-tsp refuses ("trailing bytes after signature"), tsp-js refuses ("tsp: trailing bytes after signature"), tsp_sdk 0.11.0 opens it — deliberately: `cesr/packet.rs` `decode_envelope`: "any data after the signature attachment is not part of this message (a transport may deliver several messages back-to-back) and is ignored". Every message source fails the same way (its own output and five Appendix A vectors). The specification does not say whether a TSP message is a whole transport unit; if it is, two distinct wire byte strings verify as the same message, which matters to anything keyed on a hash of the wire bytes (affinidi's mediator deduplicates on exactly that).

Observed in 6 case(s); for example `negative/self/hpke-base/trailing-bytes` (self/hpke-base → reference-rust):

```text
accepted a message with trailing-bytes (source self/hpke-base); decoded payload type "scs"
```

<details><summary>Affected cases</summary>

- `negative/self/hpke-base/trailing-bytes` self/hpke-base → reference-rust
- `negative/vector/direct-hpke-base/trailing-bytes` vector/direct-hpke-base → reference-rust
- `negative/vector/direct-signed-only/trailing-bytes` vector/direct-signed-only → reference-rust
- `negative/vector/direct-sealed-box/trailing-bytes` vector/direct-sealed-box → reference-rust
- `negative/vector/control-rfi-direct/trailing-bytes` vector/control-rfi-direct → reference-rust
- `negative/vector/direct-hpke-base-pq/trailing-bytes` vector/direct-hpke-base-pq → reference-rust

</details>

### 3. tsp_sdk does not check the ESSR sender field of a signed-only message

**Kind:** `disagreement` · **Spec:** §3.7 step 7 and §8.2.2 require the check per PKAE variant; §9.4 says every payload layout carries VID_sndr; signed-only is not addressed explicitly

The `direct-signed-only` vector's ESSR field (`4BAA`, NULL) is replaced with bob's VID and the message re-signed with alice's key, so the envelope says alice and the payload says bob. affinidi-tsp refuses ("ESSR sender VID does not match the envelope sender") because it checks the field for every layout. tsp_sdk 0.11.0 accepts: `crypto/nonconfidential.rs` `verify_payload` decodes `sender_identity` and `crypto::verify` discards it; the check exists only in `tsp_hpke::open` and `tsp_nacl::open`. The spec states the check only for the confidential variants, so this is a gap to close in the spec as much as a behaviour to align.

Observed in 1 case(s); for example `negative/resigned/direct-signed-only/essr-sender-mismatch` (vector (re-signed) → reference-rust):

```text
accepted a message with essr-sender-mismatch (source direct-signed-only (re-signed)); decoded payload type "scs"
```

<details><summary>Affected cases</summary>

- `negative/resigned/direct-signed-only/essr-sender-mismatch` vector (re-signed) → reference-rust

</details>

### 4. affinidi-tsp ignores data after the last field of an XSCS payload; tsp_sdk rejects it

**Kind:** `disagreement` · **Spec:** §9.2.3: the XSCS layout ends with the -A## stream

An extra empty Bytes field is appended inside the `-Z` frame after the `-A` stream (both counts adjusted) and the `direct-signed-only` vector re-signed. tsp_sdk refuses ("frame count does not match the framed content": `cesr/packet.rs` `decode_opaque_data` requires the `-A` stream to fill the rest of the payload exactly). affinidi-tsp returns the payload: `message/direct.rs` `decode_payload_frame` checks that the body does not overrun the `-A` stream but never that the stream ends the frame. The extra bytes are covered by the sender's signature, so this is a canonicality/malleability point rather than a forgery. Static reading: tsp-js `rev3/payload.ts` `decodePayloadFrame` has the same shape (no end-of-frame check), untestable here without a signed-only reader.

Observed in 1 case(s); for example `negative/resigned/direct-signed-only/data-after-payload-fields` (vector (re-signed) → affinidi-rust):

```text
accepted a message with data-after-payload-fields (source direct-signed-only (re-signed)); decoded payload type "scs"
```

<details><summary>Affected cases</summary>

- `negative/resigned/direct-signed-only/data-after-payload-fields` vector (re-signed) → affinidi-rust

</details>

### 5. affinidi-tsp, as the accepting endpoint, discards a cancellation that names its own accept's digest

**Kind:** `spec-violation` · **Spec:** §7.2.2: "The Digest is recorded by both endpoints … and similarly Reply_Digest"; §7.3: the RFD Digest is "the previously received Digest or Reply_Digest"

After invite → accept, the inviter's library packs an RFD naming the accept's own digest (Reply_Digest — the digest the inviter *received*) and it is delivered to affinidi-tsp's `TspAgent`, which refuses it ("message discarded: cancellation from … names an unrecognised relationship") and stays bidirectional. The same RFD naming the invite's digest is accepted, and tsp_sdk and tsp-js accept both. Cause: `TspAgent::send_relationship_accept` (lib.rs) never records the accept it sends — `set_reply_thread_digest` is called only when an accept is *received* — so `TspStore::recognizes_digest` on the accepting side holds the invite digest alone, although its own documentation says "a cancellation may name either, so both must be recognizable". The bundled endpoint APIs all happen to name the invite digest, so endpoint-to-endpoint flows hide it; the Go implementation names the Reply_Digest and hit it first, which is what prompted the `cancel-by-*-naming-*-digest` cases that now show it with any peer.

Observed in 3 case(s); for example `relationship/cancel-by-inviter-naming-accept-digest` (affinidi-rust → affinidi-rust):

```text
affinidi-rust endpoint.receive failed: [relationship] message discarded: cancellation from did:web:endpoint-a-2161e121.conformance.example names an unrecognised relationship
```

<details><summary>Affected cases</summary>

- `relationship/cancel-by-inviter-naming-accept-digest` affinidi-rust → affinidi-rust
- `relationship/cancel-by-inviter-naming-accept-digest` reference-rust → affinidi-rust
- `relationship/cancel-by-inviter-naming-accept-digest` tsp-js → affinidi-rust

</details>

### 6. Route length limits differ: tsp-js 10 hops, affinidi-tsp 16, tsp_sdk none

**Kind:** `limit` · **Spec:** §5.3: "the number of intermediaries in the route path may not be limited to 2"; no maximum is set

tsp-js `cesr/wire.ts` `MAX_HOPS = 10`, enforced when packing (`packRouted`) and when decoding any VID list (`rev3/fields.ts` `decodeVidList`, which also reads Reply_Path). affinidi-tsp `message/routed.rs` `MAX_HOPS = 16`, enforced when packing and in `wire.rs` `decode_hops`. tsp_sdk sets none. So a 12-hop route packed by affinidi-tsp or tsp_sdk cannot be opened by tsp-js, and a 17-hop route packed by tsp_sdk cannot be opened by affinidi-tsp. A bound is reasonable; three different ones are not interoperable. The specification could state a minimum every receiver must accept.

Observed in 13 case(s); for example `interop/hop/routed-17-hops` (affinidi-rust → affinidi-rust):

```text
affinidi-rust refused to pack: [malformed] invalid message: route has 17 hops, exceeds maximum of 16
```

<details><summary>Affected cases</summary>

- `interop/hop/routed-17-hops` affinidi-rust → affinidi-rust
- `interop/hop/routed-17-hops` affinidi-rust → reference-rust
- `interop/hop/routed-12-hops` affinidi-rust → tsp-js
- `interop/hop/routed-17-hops` affinidi-rust → tsp-js
- `interop/hop/routed-17-hops` reference-rust → affinidi-rust
- `interop/hop/routed-12-hops` reference-rust → tsp-js
- `interop/hop/routed-17-hops` reference-rust → tsp-js
- `interop/hop/routed-12-hops` tsp-js → affinidi-rust
- `interop/hop/routed-17-hops` tsp-js → affinidi-rust
- `interop/hop/routed-12-hops` tsp-js → reference-rust
- `interop/hop/routed-17-hops` tsp-js → reference-rust
- `interop/hop/routed-12-hops` tsp-js → tsp-js
- `interop/hop/routed-17-hops` tsp-js → tsp-js

</details>

### 7. tsp-js's relationship rules accept an RFA that names an invite never sent

**Kind:** `api-gap` · **Spec:** §7.2.2: the RFA's Digest is the digest of the corresponding TSP_RFI

The inviter (tsp-js) is sent an RFA packed by the other implementation's library with a random Digest. affinidi-tsp (`TspAgent::handle_control`: "accept … answers an invite we did not send") and tsp_sdk (`SecureStore::upgrade_relation`: "thread_id does not match digest") refuse it and stay in invite-sent. tsp-js ships its relationship rules as pure functions (`relationship.ts`) and leaves storage to the wallet; `transition("pending", "receiveAccept")` returns `bidirectional` without seeing a digest, and unlike the invite race (`resolveInviteRace`) and cancellation (`resolveCancel`) there is no helper for accept correlation. The driver applies only library rules, so the gap is visible here; every integrator of tsp-js must add the check themselves.

Observed in 3 case(s); for example `relationship/accept-unknown-digest-refused` (tsp-js → affinidi-rust):

```text
tsp-js accepted an accept naming an invite it never sent (§7.2.2): event "accept"
```

<details><summary>Affected cases</summary>

- `relationship/accept-unknown-digest-refused` tsp-js → affinidi-rust
- `relationship/accept-unknown-digest-refused` tsp-js → reference-rust
- `relationship/accept-unknown-digest-refused` tsp-js → tsp-js

</details>

## Warnings

Correct rejections under an error code other than the one expected (the specification mandates no taxonomy).

| Implementation | Case → code | Count |
|---|---|---:|
| affinidi-rust | `flip-envelope-sender → [malformed]` | 1 |

## Skip reasons

| Reason | Cases |
|---|---:|
| tsp-js lacks signed-only:open | 27 |
| tsp-js lacks sealed-box:open | 24 |
| tsp-js lacks padding:pack | 21 |
| tsp-js lacks hpke-pq:open | 19 |
| tsp-js lacks sealed-box:pack | 12 |
| mutation not applicable to this source | 10 |
| affinidi-rust lacks deterministic | 8 |
| reference-rust lacks deterministic | 8 |
| tsp-js lacks deterministic | 8 |
| tsp-js lacks signed-only:pack | 7 |
| reference-rust pack: unsupported — padding is reachable only for scs/ctl/pad (SecureStore SendOptions) | 6 |
| tsp-js lacks hpke-pq:pack | 6 |
| affinidi-rust pack: unsupported — affinidi-tsp always carries the sender VID in the payload; NULL cannot be requested | 4 |
| reference-rust lacks rfi.referral:pack | 3 |
| reference-rust pack: unsupported — tsp_sdk chooses the ESSR sender field by scheme (NULL under HPKE-Base and signed-only) | 3 |
| the vector publishes no ephemeral material (spec: the hybrid KEM draws encapsulation randomness) | 3 |
| tsp-js lacks payload.ctl:pack | 3 |
| tsp-js lacks payload.pad:pack | 3 |
| tsp-js lacks payload.pad:pack, padding:pack | 3 |
| tsp-js lacks rfi.referral:pack | 3 |
| tsp-js pack: unsupported — tsp-js always writes the sender VID in the ESSR field | 3 |
| a re-signed but untouched signed-only rfi does not open here ([unsupported] payload type is not supported by this implementation) | 1 |
