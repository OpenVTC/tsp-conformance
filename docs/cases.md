# Case catalogue

Every case the runner executes, what it asserts, and the section of TSP Rev 3
(`trustoverip/tswg-tsp-specification` `f5b8668`) it traces to. Section numbers
follow the specification's own numbering (§9.4.1 is *TSP_RFI* encoding, and so
on); "A" is Appendix A.

Case ids are `suite/case`; `--case <substring>` selects by them.

## Conventions

- **Field equality.** An `open` result is compared field by field with what the
  packer was asked to produce: `version` (0.2), `envelopeSender`,
  `envelopeReceiver`, `scheme`, `payload.type` and every type-specific field
  (`data`, `digest`, `replyDigest`, `digestAlg`, `nonce`, `replyPath`,
  `referral.vid`, `hops`, `inner`), plus `padding` and `payloadSender` when the
  opener reports them (driver protocol §7.1).
- **ESSR sender.** When the packer took its default, `payloadSender` may be
  NULL or the envelope sender under HPKE-Base and signed-only (§3.2, §8.2.2);
  under the sealed box it must be the sender (§8.3.2).
- **Digests.** For `rfi`/`rfa` the digest the packer reported must equal the
  digest the opener decoded; the opener is required to have verified it
  (driver protocol §4 `open`), so a pass means both derived the same SAID
  (§7.2.1).
- **Keys.** Fresh Ed25519/X25519 identities per case from a seeded ChaCha20
  generator (`--seed`); post-quantum cases use Appendix A's `pq_alice`/`pq_bob`.
- **Gating.** A case needs the scheme and payload capabilities on the packing
  side (`:pack`) and the opening side (`:open`), plus `padding`,
  `rfi.reply-path`, `rfi.referral` where used. Missing capability or an
  `unsupported` answer is a **skip**.

## 1. `vectors` — per implementation

For each of the ten Appendix A vectors (`direct-sealed-box`, `direct-hpke-base`,
`direct-signed-only`, `control-rfi-direct`, `control-rfa-direct`, `control-rfd`,
`control-rfi-sealed-box`, `nested-direct`, `routed`, `direct-hpke-base-pq`):

| Case | Asserts | Spec |
|---|---|---|
| `<vector>/open` | Opens with the published keys; every field equals the value decoded (by the runner's text-domain CESR reader) from the vector's printed `payload` — including digests, nonce, reply path, hop list and the raw inner message | A; per vector (§8.2, §8.3, §3.5, §7.2.1–7.2.2, §7.3, §9.4.1–9.4.2, §9.4.5, §4/§9.4.15, §5.3/§9.4.16) |
| `nested-direct/inner-open`, `routed/inner-open` | The inner message opens with `nested_alice`/`nested_bob` and matches `innerPayload` | A; §4, §5.3 |
| `<vector>/pack-exact` | `pack` with the vector's `ikmE`/`skEm`, nonce and padding reproduces `message` byte for byte. Needs `deterministic`, except `direct-signed-only`, which has no randomness at all (Ed25519) and is attempted for every signed-only packer. `direct-hpke-base-pq` publishes no ephemeral value and always skips | A (preamble) |
| `<vector>/peek` | Keyless envelope view: sender, receiver, `confidential`, version if reported | §3.1, §5.1 |
| `direct-signed-only/resigned/minor-ABA-accepted` | Version count rewritten to `YTSP-ABA` (what tsp_sdk 0.10 emitted) and re-signed with alice's published key: must open, reporting 0.64 | §9.1 (MINOR does not gate) |
| `direct-signed-only/resigned/minor-AAD-accepted` | Same with a later MINOR `AAD` | §9.1 |
| `direct-signed-only/resigned/essr-sender-present-accepted` | ESSR field changed from NULL to alice's VID and re-signed: must open | §3.7 step 7 |

## 2. `interop` — every ordered pair, including self-pairs

P packs, R opens. All HPKE-Base unless named.

| Case | What P packs | Spec |
|---|---|---|
| `scs/hpke-base/small` | 13-byte application payload | §3.5, §8.2, §9.2.3 |
| `scs/hpke-base/empty` | empty payload | §9.2.3 |
| `scs/hpke-base/2mib` | 2 MiB payload: long `--E`, `--Z`, `--A`, `7AAF`/`8AAF`/`9AAF` codes | §9.1 |
| `scs/hpke-base/padding-{0,1,2,3}` | Padding_Field of 0–3 bytes: lead pad 0/2/1/0 (`4B`/`6B`/`5B`/`4B`) | §9.2.4 |
| `scs/hpke-base/padding-12285` | largest short padding (`4B__`, 4095 quadlets) | §9.2.4 |
| `scs/hpke-base/padding-12286` | smallest long padding (`9AAB####`) | §9.2.4 |
| `scs/signed-only` | non-confidential application message | §3.5 |
| `scs/sealed-box` | libsodium sealed box | §8.3, §9.2.8 |
| `scs/hpke-base/payload-sender-null` | ESSR field requested NULL | §3.2, §8.2.2 |
| `scs/hpke-base/payload-sender-present` | ESSR field requested = sender | §3.2, §8.2.2 |
| `scs/hpke-base/long-vid` | 13 000-byte sender VID (`7AAB####` in the envelope) | §9.1 |
| `scs/hpke-pq` | MLKEM768-X25519 + ML-DSA-65 (`pq_alice` → `pq_bob`) | §8.2.3, §9.5.2 |
| `ctl/hpke-base` | `XCTL` generic control payload | §9.3 |
| `pad/hpke-base`, `pad/hpke-base/padded` | `XPAD`, without and with a Padding_Field | §7.5, §9.4.7 |
| `rfi/hpke-base/direct` | invite with a caller nonce; R's decoded digest = P's | §7.2.1, §7.2.2, §9.4.1 |
| `rfi/hpke-base/reply-path` | invite with a two-hop Reply_Path | §7.2.4, §9.4.1 |
| `rfi/hpke-base/long-reply-path` | Reply_Path over 4095 quadlets (`--J#####`) | §9.4.1 |
| `rfi/hpke-base/referral` | invite with Referral_Field; additionally, when R reports `Signature_new`, the runner verifies it under VID_new's key over {XRFI, VID_sndr\|4BAA, Digest, Nonce, Reply_Path, VID_new} | §7.2.5, §9.4.1 |
| `rfi/hpke-base/padded` | invite with padding — the SAID must exclude it | §7.2.1 |
| `rfi/sealed-box` | invite under the sealed box: Blake2b-256 digest (`F`) | §7.2.1, §8.3 |
| `rfi/hpke-pq` | invite between post-quantum identities | §7.2.1, §8.2.3 |
| `rfa/hpke-base/echo-own-rfi` | P packs an rfi, then an rfa echoing P's own digest | §7.2.2, §9.4.2 |
| `rfa/hpke-base/answers-receiver-rfi` | R packs an rfi; P opens it and answers with an rfa echoing the digest P decoded; R must see its own digest echoed | §7.2.2, §9.4.2 |
| `rfa/sealed-box`, `rfd/sealed-box` | accept/cancel under the sealed box (Blake2b-256) | §7.2.2, §7.3, §8.3 |
| `rfd/hpke-base` | cancel naming a digest | §7.3, §9.4.5 |
| `hop/nested` | inner packed by P (alice-inner → bob-inner), outer by P; R opens both | §4, §9.4.15 |
| `hop/nested/padded` | nested outer with a Padding_Field | §9.4.15 |
| `hop/nested/signed-only-inner` | signed-only inner inside an HPKE outer | §4 |
| `hop/nested-mixed/inner-by-X` | three implementations: inner by X, outer by P, both opened by R (every X) | §4, §9.4.15 |
| `hop/routed-2-hops` | alice → p carrying `[q, bob]` and the alice → bob message; R opens the outer as p and the inner as bob | §5.3, §9.4.16 |
| `hop/routed-long-hop-list` | hop list over 4095 quadlets (`--J#####`) | §9.4.16 |
| `hop/routed-12-hops`, `hop/routed-17-hops` | long routes (the spec sets no maximum) | §5.3 |

## 3. `negative` — per implementation R

A corrupted message must be refused (`ok:false`). Any code passes; a code other
than the expected one is a warning. Each mutation is applied to every source
whose unmodified message R can open first (otherwise skip): R's own HPKE-Base
output (`self/hpke-base`) and the vectors `direct-hpke-base`,
`direct-signed-only`, `direct-sealed-box`, `control-rfi-direct`,
`direct-hpke-base-pq`.

| Mutation | Expected code | Spec |
|---|---|---|
| `flip-ciphertext` | decrypt | §3.7 step 6, §8.2.2 |
| `flip-signature` | signature | §3.4, §3.7 step 5, §9.5 |
| `flip-envelope-sender` | signature / sender | §3.7, §8.2.2 (aad) |
| `flip-version-minor` | signature / decrypt | §9.1 (MINOR is signed and in the aad) |
| `unknown-major-version` (`-AAC` → `-BAC`) | version | §9.1 |
| `truncate-one-triplet`, `truncate-half` | malformed | §9.1, §9.5 |
| `trailing-bytes` (one extra zero triplet) | malformed | §9.1, §9.5 |
| `frame-count-too-large`, `frame-count-too-small` (`-E` ±1) | malformed | §9.1 |
| `signature-index-nonzero` (`B0` → `B1`; the index is not signed) | signature / malformed | §9.5.1 |
| `attachment-count-too-large` (`-C` +1) | malformed | §9.5 |
| `wrong-receiver-key` (same VID, other X25519 key) | decrypt | §3.7 step 6 |
| `wrong-sender-key` (same VID, other Ed25519 key) | signature | §3.7 step 5 |

Re-signed cases — the edit is followed by a fresh signature with the sender's
key, so only the check under test can catch it (signed-only messages only; the
runner implements no HPKE):

| Case | Edit | Expected | Spec |
|---|---|---|---|
| `resigned/signed-only-rfi/digest-tampered` | a signed-only rfi (packed by the first driver able to) with one digest character changed | digest | §7.2.1 |
| `resigned/direct-signed-only/non-canonical-lead-byte` | the `5BAH` data field's lead byte made non-zero | malformed | §3.7 (MUST reject) |
| `resigned/direct-signed-only/essr-sender-mismatch` | ESSR field set to bob while the envelope says alice | sender | §3.7 step 7, §8.2.2 |
| `resigned/direct-signed-only/payload-count-too-large` | `-Z` count +1 | malformed | §9.2 |
| `resigned/direct-signed-only/data-after-payload-fields` | an extra field after the `-A` stream, counts adjusted | malformed | §9.2.3 |

## 4. `relationship` — every ordered pair of endpoints (A, B)

Needs `endpoint` on both. The runner carries messages between the drivers.

| Case | Flow and assertions | Spec |
|---|---|---|
| `invite-accept-bidirectional` | A invites; B's decoded digest = A's; states invite-sent / invite-received (with digest if reported); B accepts echoing it; A decodes both digests; both bidirectional with agreeing digest/replyDigest; application data both ways | §7.2.1, §7.2.2 |
| `message-before-relationship-refused` | A's library packs an application message with no relationship; B refuses it and stays `none` | §7.2.2 |
| `cancel-returns-to-none` | after forming, A cancels; B sees `cancel` naming one of the two digests; both `none` | §7.3 |
| `cancel-by-{inviter,accepter}-naming-{invite,accept}-digest` | after forming, one side's *library* packs an RFD naming a chosen digest (the endpoint API always picks its own); the other endpoint must report `cancel` and reach `none`. Covers all four combinations, since §7.2.2 has both endpoints record both digests | §7.2.2, §7.3 |
| `rfi-race-lower-digest-wins` | both invite at once and each receives the other's; the side with the lexicographically lower digest stays invite-sent, the other becomes invite-received on that digest; the accept then completes with the lower digest on both sides | §7.2.3 |
| `accept-unknown-digest-refused` | A invites; B's library packs an rfa echoing a random digest; A refuses it and stays invite-sent | §7.2.2 |
