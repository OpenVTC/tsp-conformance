# Dart driver — `affinidi_tsp`

Conformance driver (protocol v1, `docs/driver-protocol.md`) for the Dart TSP
Rev 3 library at `../../../affinidi-tsp-dart` (package `affinidi_tsp`) and its
post-quantum key types (`affinidi_tsp_pq`). It is a thin adapter: every byte
it returns is produced by the library.

Requires the Dart SDK ≥ 3.10 (the post-quantum package depends on
`pqcrypto`, which does).

## Build

```sh
dart pub get && mkdir -p build && dart compile exe bin/tsp_driver.dart -o build/tsp_driver
```

## Run

```sh
build/tsp_driver
```

Without compiling (slower start-up, same behaviour):

```sh
dart run bin/tsp_driver.dart
```

`drivers.toml` entry:

```toml
[[driver]]
name = "affinidi-dart"
description = "affinidi_tsp (affinidi-tsp-dart), path dependency, with affinidi_tsp_pq"
cwd = "drivers/dart"
build = "dart pub get && mkdir -p build && dart compile exe bin/tsp_driver.dart -o build/tsp_driver"
run = "build/tsp_driver"
enabled = true
```

## Self-check

```sh
dart test
```

Pipes `hello` and the `direct-hpke-base` vector `open` through the real
stdin/stdout loop, opens every Appendix A vector (including
`direct-hpke-base-pq`), re-packs every classical vector with its published
ephemeral material and checks the bytes are identical, and runs an endpoint
invite/accept round trip.

## Capabilities

All of: `hpke-base`, `signed-only`, `sealed-box`, `hpke-pq`, `payload.scs`,
`payload.ctl`, `payload.pad`, `payload.rfi`, `payload.rfa`, `payload.rfd`,
`rfi.reply-path`, `rfi.referral`, `payload.hop`, `padding`, `payload-sender`,
`deterministic`, `peek`, `endpoint`.

## Interpretation notes

- `hpke-pq` + `ephemeral.ikmE`: the 64-byte value is used directly as the
  MLKEM768-X25519 encapsulation randomness (the draft-ietf-hpke-pq test-vector
  convention: 32 bytes for ML-KEM, 32 for X25519), not fed to DeriveKeyPair.
- `payloadSender` other than `null` or the envelope sender is answered
  `unsupported`; `null` under `sealed-box` is `invalid-input` (§8.3 makes the
  sender mandatory there).
- `pack` with scheme `hpke-base` to a receiver whose `encKeyType` is
  `MLKEM768-X25519` (or `hpke-pq` to an X25519 receiver) is `invalid-input`.
- `rfa` input: the echoed digest is written with the scheme's digest code
  (`F` under the sealed box, `I` otherwise) unless `digestAlg` is given.
- `endpoint.receive` of a crossing invite that loses to our own lower-digest
  invite (§7.2.3) answers `ok:false` code `relationship` and keeps state
  `invite-sent`; the other side adopts the winning invite.
- `endpoint.send` requires a `bidirectional` relationship; `endpoint.receive`
  admits application data in `invite-received` or `bidirectional`.
- `endpoint.receive` of `ctl`/`pad`/`hop` payloads is `unsupported`.
