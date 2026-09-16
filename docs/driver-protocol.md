# TSP conformance driver protocol — v1

Every implementation under test is wrapped in a small **driver**: an executable
that speaks this protocol on stdin/stdout. The runner never links an
implementation; it only talks to drivers. That is what lets one runner pair a
Rust sender with a Dart receiver, or a Go sender with the ToIP reference.

A driver is a thin adapter. It must not implement TSP itself — every byte it
returns must come from the library under test. If the library cannot do
something, the driver answers `unsupported`; it never fills the gap.

Target: **TSP Rev 3**, `YTSP-AAC` (spec `trustoverip/tswg-tsp-specification`
commit `f5b8668`). Fixture: `fixtures/spec-vectors.json`.

## 1. Framing

- The runner starts the driver once (from `drivers.toml`) and keeps it alive.
- Requests are **one JSON object per line** on stdin. The driver writes **exactly
  one JSON object per line** on stdout for each request, in order.
- stdout carries protocol lines only. Logs go to stderr.
- On stdin EOF the driver exits 0.

Request: `{"id": <int>, "op": "<name>", ...params}`

Success: `{"id": <int>, "ok": true, "result": {...}}`

Failure: `{"id": <int>, "ok": false, "error": {"code": "<code>", "message": "<free text>"}}`

A driver that panics or writes a non-JSON line fails the current case and is
restarted by the runner.

## 2. Encodings

- **All byte strings are base64url without padding** (RFC 4648 §5), including
  messages, keys, digests and nonces.
- Messages are in the **binary (qb2) domain** — the bytes that go on the wire.
  (Spec vectors are printed in qb64; the runner base64url-decodes them before
  sending, which is the CESR text→binary transcode.)
- VIDs are plain strings.
- A **digest** is the raw 32-byte hash value, *not* its CESR encoding. Its
  algorithm is reported alongside it where it can vary: `"sha2-256"` (code `I`)
  or `"blake2b-256"` (code `F`).

## 3. Shared types

### Identity

```json
{
  "id": "did:peer:4zQm...",
  "sigKeyType": "Ed25519" | "MlDsa65",
  "encKeyType": "X25519" | "MLKEM768-X25519",
  "pkS": "<b64u>", "pkE": "<b64u>",
  "skS": "<b64u>", "skE": "<b64u>"
}
```

`skS`/`skE` are present only where the driver acts as that identity. Key forms
match Appendix A: Ed25519/X25519 are 32 raw bytes; `MlDsa65` `skS` is the
4032-byte expanded key and `pkS` 1952 bytes; `MLKEM768-X25519` `skE` is the
32-byte seed (DeriveKeyPair input) and `pkE` 1216 bytes.

### Scheme

| value | meaning |
|---|---|
| `hpke-base` | HPKE-Base, DHKEM(X25519)/HKDF-SHA256/ChaCha20Poly1305 (§8.2) |
| `hpke-pq` | HPKE-Base with KEM `MLKEM768-X25519` (0x647a); signature ML-DSA-65 |
| `sealed-box` | libsodium anonymous sealed box (§8.3) |
| `signed-only` | non-confidential: payload in the clear, signed |

### Payload

Tagged by `type`. The same shape is used as **input** to `pack` and **output**
of `open`. Fields marked *(out)* are ignored on input and must be reported on
output; fields marked *(in)* are only meaningful on input.

| `type` | code | fields |
|---|---|---|
| `scs` | `XSCS` | `data` — the application bytes (the content of the single Bytes primitive inside the `-A##` stream; Appendix A `direct-hpke-base` carries `"hello world"`) |
| `ctl` | `XCTL` | `data` — as for `scs` |
| `pad` | `XPAD` | `nonce` (16 bytes) |
| `rfi` | `XRFI` | `nonce` (16 bytes), `replyPath` (array of VIDs, `[]` for direct), `referral` (`null` or `{"vid", "signature"}`), `digest` *(out)* — the RFI's own SAID, `digestAlg` *(out)* |
| `rfa` | `XRFA` | `digest` — the invite's digest, echoed; `replyDigest` *(out)* — this message's own SAID; `digestAlg` *(out)* |
| `rfd` | `XRFD` | `digest` — the relationship being declined/cancelled; `digestAlg` |
| `hop` | `XHOP` | `hops` (array of VIDs; `[]` = nested, non-empty = routed), `inner` — a complete encoded TSP message |

Every payload additionally carries:

- `padding` — bytes of the Padding_Field; `""` when none. *(in and out)*
- `payloadSender` — the ESSR `VID_sndr` inside the payload, or `null` for the
  NULL VID `4BAA`. On input to `pack`: omit to take the library's default for
  the scheme; `null` to request NULL; a string to request that VID. A library
  that does not let the caller choose returns `unsupported` only if the
  requested value differs from what it would do.

`rfi`/`rfa` digests on input: `rfi.digest` and `rfa.replyDigest` are always
computed by the library. `rfa.digest` and `rfd.digest` are supplied by the
caller. `digestAlg` on input to `rfd` is optional (default `sha2-256`).

## 4. Operations

### `hello`

→ `{}`

← `{"name": "affinidi-tsp", "version": "0.2.0", "language": "rust", "protocol": 1, "capabilities": [...]}`

Capabilities (strings):

| capability | covers |
|---|---|
| `hpke-base` | pack+open under `hpke-base` |
| `signed-only` | pack+open non-confidential |
| `sealed-box` | pack+open under `sealed-box` |
| `hpke-pq` | pack+open under `hpke-pq` |
| `payload.scs` `payload.ctl` `payload.pad` | those payload types |
| `payload.rfi` `payload.rfa` `payload.rfd` | control messages (direct) |
| `rfi.reply-path` `rfi.referral` | non-empty `replyPath` / `referral` |
| `payload.hop` | nested and routed |
| `padding` | caller-chosen non-empty padding |
| `payload-sender` | caller choice of NULL vs present ESSR sender |
| `deterministic` | `pack` honours `ephemeral` (and caller nonces), making output byte-reproducible |
| `peek` | the `peek` op |
| `endpoint` | the stateful `endpoint.*` ops (§5) |

A pack-only or open-only gap is expressed with a suffix: `hpke-pq:open`
means it can open but not pack. A capability without a suffix means both.

### `pack`

→
```json
{
  "scheme": "hpke-base",
  "sender": Identity,     // with skS (and skE)
  "receiver": Identity,   // public only
  "payload": Payload,
  "ephemeral": {"ikmE": "<b64u>"} | {"skEm": "<b64u>"} | null
}
```

`ephemeral` is the Appendix A key material: `ikmE` for HPKE (DeriveKeyPair
input), `skEm` for the sealed box. When non-null and the driver lacks
`deterministic`, answer `unsupported`.

← `{"message": "<b64u>", "digest": "<b64u>"|null, "digestAlg": "..."|null}`

`digest` is the SAID the library computed for an `rfi` (its `digest`) or `rfa`
(its `replyDigest`); `null` otherwise.

### `open`

→ `{"receiver": Identity /* with skE, skS */, "sender": Identity /* public */, "message": "<b64u>"}`

← 
```json
{
  "version": {"major": 0, "minor": 2},
  "envelopeSender": "did:...",
  "envelopeReceiver": "did:...",
  "scheme": "hpke-base",
  "payload": Payload
}
```

`open` MUST verify the signature, decrypt, and check any SAID it carries. For
`hop` it returns the inner message **unopened**.

### `peek`

→ `{"message": "<b64u>"}` ← `{"version": {...}, "envelopeSender", "envelopeReceiver", "confidential": bool}`

No keys. What an intermediary can see.

## 5. Stateful endpoint ops (capability `endpoint`)

For relationship-protocol scenarios the runner drives two endpoints, each
possibly in a different driver, and carries messages between them. Each
endpoint holds its own identities and relationship store.

| op | request | result |
|---|---|---|
| `endpoint.create` | `{"identities": [Identity private], "peers": [Identity public]}` | `{"endpoint": "<handle>"}` |
| `endpoint.invite` | `{"endpoint", "from", "to"}` | `{"message", "digest"}` |
| `endpoint.accept` | `{"endpoint", "from", "to"}` — accept the invite received from `to` | `{"message", "digest", "replyDigest"}` |
| `endpoint.cancel` | `{"endpoint", "from", "to"}` | `{"message"}` |
| `endpoint.send` | `{"endpoint", "from", "to", "data"}` | `{"message"}` |
| `endpoint.receive` | `{"endpoint", "message"}` | `{"event": "invite"\|"accept"\|"cancel"\|"message", "from", "to", "data"?, "digest"?, "replyDigest"?}` |
| `endpoint.state` | `{"endpoint", "local", "remote"}` | `{"state": "none"\|"invite-sent"\|"invite-received"\|"bidirectional", "digest"?, "replyDigest"?}` |

A message the endpoint refuses (e.g. application data with no relationship,
an accept whose digest matches no outstanding invite) is an `ok:false` with
code `relationship`.

## 6. Error codes

| code | when |
|---|---|
| `unsupported` | op, scheme, payload type or option not supported |
| `malformed` | CESR/framing cannot be parsed |
| `version` | unknown/unsupported TSP version |
| `signature` | signature verification failed |
| `decrypt` | decryption/authentication failed |
| `sender` | envelope/ESSR sender mismatch, or wrong sender key |
| `receiver` | message not addressed to the receiver identity |
| `digest` | a SAID does not verify |
| `relationship` | relationship state forbids the message |
| `invalid-input` | the request itself is bad |
| `internal` | anything else |

The runner treats **any** `ok:false` as a correct rejection in negative cases;
a code that differs from the expected one is reported as a warning, not a
failure — libraries classify errors differently and the spec does not mandate
a taxonomy.

## 7. Clarifications (additive, still protocol 1)

Written while building the first three drivers and the runner. None changes
the meaning of anything above; each resolves a case the text left open, and a
driver written against §1–§6 alone remains conformant.

1. **Unreported output fields.** `open` MAY omit a field its library does not
   expose: `version`, `payload.padding`, `payload.payloadSender`, `nonce` on
   `rfi`/`pad`, and `referral.signature`. An *absent* key means "not reported"
   and is not compared; it is listed per implementation in the report. It is
   distinct from `payloadSender: null`, which asserts the NULL VID `4BAA`.
   Every other output field is required. `peek` MAY omit `version` likewise.
2. **Caller nonces are optional.** On `pack`, `nonce` for `rfi` and `pad` MAY
   be omitted, in which case the library draws it. A driver whose library
   cannot take a caller nonce answers `unsupported` only when one is given.
3. **Referral input carries a key, not a signature.** `Signature_new` covers
   the invite's own digest (§9.4.1), which exists only inside `pack`, so no
   caller can supply it. On `pack` input, `referral` is
   `{"vid": "<VID_new>", "skS": "<b64u>"}` (plus `sigKeyType`, default
   `Ed25519`); the library makes the signature and `signature` is ignored on
   input. On `open` output `referral` is `{"vid", "signature"}` with the raw
   signature bytes. `open` does not verify `Signature_new`: the request names
   no key for `VID_new`.
4. **`hop` shape.** `hops` is an array of VID strings; `inner` is the complete
   inner message bytes (b64u). `[]` is nested, anything else routed. A driver
   may refuse a `hop` payload under `signed-only` (§4 requires the outer of a
   nesting to be confidential).
5. **`digestAlg` follows the scheme.** `pack` results carry `digestAlg`
   whenever `digest` is non-null. The library chooses the digest code by
   scheme (`blake2b-256` under `sealed-box`, else `sha2-256`); a `digestAlg`
   on `rfd` input that disagrees with the scheme MAY be answered `unsupported`.
6. **Signed-only open.** `open` of a non-confidential message reports
   `scheme: "signed-only"`. The request still carries a `receiver` identity
   (its keys are unused).
7. **Scheme vs. key types.** `hpke-pq` is used exactly when the identities are
   `MlDsa65`/`MLKEM768-X25519`; `hpke-base` with post-quantum identities (or
   the reverse) MAY be `unsupported`.
8. **`unsupported` always skips.** Any `unsupported` answer makes the runner
   skip the case, even when `hello` advertised the capability (a capability is
   coarse — e.g. `signed-only` may cover application payloads only).
9. **Endpoint details.** `endpoint.state` MAY omit `digest`/`replyDigest` when
   the library does not expose them. `endpoint.accept` accepts the invite most
   recently received from `to`. `endpoint.receive` of an invite that loses the
   §7.2.3 race MAY answer `ok:false` (`relationship`) or report the `invite`
   event; the runner judges by the subsequent `endpoint.state`. An endpoint
   handle is valid only for the lifetime of the driver process.
10. **Lifecycle.** The runner sends `hello` first after every (re)start, uses
    strictly increasing `id`s, and treats a response whose `id` differs from
    the request's, a non-JSON line, EOF or a per-request timeout as a broken
    driver: the case is an **error** and the driver is restarted for the next
    request. Driver logs (stderr) are kept under `reports/logs/`.
