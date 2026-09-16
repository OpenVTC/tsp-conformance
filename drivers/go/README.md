# Go driver — affinidi-tsp-go

Wraps [`affinidi-tsp-go`](../../../affinidi-tsp-go) (TSP Rev 3, `YTSP-AAC`) in
the driver protocol of [`docs/driver-protocol.md`](../../docs/driver-protocol.md).
The library is taken from the sibling checkout through a `replace` directive
in `go.mod`. Requires Go 1.27.

## Build and run

```
go build -o tsp-driver .     # build (cwd: drivers/go)
./tsp-driver                 # run: JSON lines on stdin/stdout
```

`drivers.toml` entry:

```toml
[[driver]]
name = "go"
description = "affinidi-tsp-go (sibling checkout via replace)"
cwd = "drivers/go"
build = "go build -o tsp-driver ."
run = "./tsp-driver"
enabled = true
```

## Self-check

```
go test ./...     # hello, every Appendix A vector through open (and pack where
                  # reproducible), an endpoint relationship flow, error codes
./selfcheck.sh    # builds the binary and pipes hello + direct-hpke-base open
```

## Capabilities

All of: `hpke-base`, `signed-only`, `sealed-box`, `hpke-pq`, `payload.scs`,
`payload.ctl`, `payload.pad`, `payload.rfi`, `payload.rfa`, `payload.rfd`,
`rfi.reply-path`, `rfi.referral`, `payload.hop`, `padding`, `payload-sender`,
`deterministic`, `peek`, `endpoint`.

Qualifications:

- `deterministic` covers `hpke-base` (`ikmE`) and `sealed-box` (`skEm`). A
  `pack` under `hpke-pq` with a non-null `ephemeral` answers `unsupported`: the
  hybrid KEM is the standard library's, which draws its own encapsulation
  randomness. For `signed-only` the ephemeral is ignored (nothing is encrypted).
- `payloadSender` may be `null` or the envelope sender; any other VID answers
  `unsupported`. `null` under `sealed-box` is refused (`invalid-input`): the
  scheme requires the ESSR sender.
- `rfi.referral` on `pack`: `Signature_new` covers the invite's own digest, so a
  caller cannot compute it in advance. The driver accepts either
  `{"vid", "signature"}` (carried verbatim) or the extension
  `{"vid", "skS", "sigKeyType"?}`, in which case the library signs. `open`
  reports the referral unverified (the introduced VID's key is not part of the
  request).
- `scs`/`ctl` `data` is the single Bytes primitive of the `-A##` stream. The
  library also accepts other stream contents; `open` of such a stream answers
  `unsupported` because the protocol has no field for it.
- `endpoint.receive` of `pad`, `ctl` or `hop` answers `unsupported` (the
  protocol defines only invite / accept / cancel / message events).

## Endpoint semantics

- `invite` is allowed from `none` or `invite-sent` (replacing the outstanding
  invite); `accept` requires `invite-received`; `send` requires `bidirectional`;
  `cancel` requires any relationship and returns the local side to `none`.
- A crossing invite is kept only if its digest is lower (binary comparison of
  the 32 bytes); the losing incoming invite is refused with `relationship`.
- An RFD names the relationship with the digest this side *received*
  (§7.3): the Reply_Digest for the inviter, the Digest for the accepter; a
  pending invite is named by its Digest. On receive either digest is recognised.
- Application messages are accepted only over a `bidirectional` relationship.
