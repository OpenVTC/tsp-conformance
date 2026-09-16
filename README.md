# TSP Rev 3 conformance suite

A cross-implementation conformance suite for the **Trust Spanning Protocol, Rev 3**
(`YTSP-AAC`, `trustoverip/tswg-tsp-specification` commit `f5b8668`).

It answers three questions about every implementation it is given:

1. **Does it read the specification's own test vectors?** Every Appendix A
   vector is opened and every decoded field compared with the value printed in
   the specification — not with another implementation.
2. **Does it interoperate?** Every ordered pair of implementations (including
   each with itself) packs and opens ~35 message shapes with fresh keys, down to
   the digests each side derives.
3. **Does it refuse what it must, and run the relationship protocol the same
   way?** Corrupted and re-signed messages must be rejected; invite, accept,
   cancel and the invite race must end in the same state on both endpoints.

The latest run is in [`reports/report.md`](reports/report.md) (machine-readable:
[`reports/report.json`](reports/report.json)).

## Layout

```
drivers.toml              which drivers exist and how to build/start them
findings.toml             investigated root causes, used to group failures
fixtures/spec-vectors.json  Appendix A, extracted by tools/extract_spec_vectors.py
docs/driver-protocol.md   the driver contract (JSON lines over stdin/stdout)
docs/cases.md             every case, what it asserts, the spec section it traces to
runner/                   `tsp-conformance`, the Rust runner (links no TSP implementation)
drivers/affinidi-rust/    driver for affinidi-tsp (path dependency, feature pq)
drivers/reference-rust/   driver for the ToIP reference tsp_sdk 0.11.0 (crates.io)
drivers/tsp-js/           driver for @openvtc/vti-tsp-js (Node, imports its built dist)
drivers/go/, drivers/dart/  maintained with the Go and Dart libraries
reports/                  report.json, report.md, logs/ (driver stderr, build output)
```

The runner never links an implementation. It starts each driver as a process
and speaks [the driver protocol](docs/driver-protocol.md) to it, which is what
lets a Rust packer be paired with a TypeScript opener. A driver is a thin
adapter: every byte it returns comes from its library, and anything the
library cannot do is answered `unsupported` — which the report shows as a skip
and in the capability matrix, never as a failure.

## Running it

Prerequisites: a stable Rust toolchain (`cargo +stable`; tsp_sdk needs newer
than the affinidi-tdk-rs pin), Node ≥ 20 with `npm`, and Go ≥ 1.27 and Dart ≥ 3.10
for their drivers. The drivers build the implementations from sibling checkouts
next to this directory:

```sh
git clone https://github.com/affinidi/affinidi-tdk-rs
git clone -b feat/tsp-rev2-rev3-dual-handler https://github.com/OpenVTC/vta-browser-plugin pnm-browser-plugin   # tsp-js (Rev 3 is not on main yet)
git clone https://github.com/affinidi/affinidi-tsp-go
git clone https://github.com/affinidi/affinidi-tsp-dart
```

The reference implementation comes from crates.io. To test other checkouts or
branches, copy `drivers.local.toml.example` to `drivers.local.toml`.

```sh
make build        # runner + every driver's build command
make run          # all suites, all enabled drivers
make report       # re-render reports/report.md from reports/report.json
```

`make run` takes `DRIVERS=`, `SUITES=`, `CASE=`, `SEED=`, `TIMEOUT=`. Or call the
runner directly:

```sh
runner/target/release/tsp-conformance \
  --drivers affinidi-rust,reference-rust,tsp-js \
  --suites vectors,interop,negative,relationship \
  --case rfi/ --seed 20260916 \
  --json reports/report.json --markdown reports/report.md
```

| Flag | Meaning |
|---|---|
| `--drivers a,b` | run these drivers (default: every `enabled` one) |
| `--suites …` | `vectors`, `interop`, `negative`, `relationship` (default: all) |
| `--case s` | only cases whose `suite/case` contains `s` (repeatable) |
| `--build` / `--build-only` | run each driver's build command first / and stop |
| `--seed n` | key-generation seed; the seed used is always recorded in the report |
| `--timeout s` | per-request timeout (default 60); a timed-out or crashed driver fails that case and is restarted |
| `--json`, `--markdown` | report paths |
| `--from-json` | re-render Markdown from an existing JSON report |
| `-v` | print every case, not only failures |

To build against a worktree or branch without editing committed files, copy
`drivers.local.toml.example` to `drivers.local.toml` (git-ignored). It overrides
entries of `drivers.toml` by name and adds `env`: `AFFINIDI_TSP_PATH` makes
`drivers/affinidi-rust/build.sh` build against another copy of the crate, and
`TSP_JS_DIR` points the tsp-js build and driver at another package directory.
Overrides in effect are listed in the report; `--note "…"` adds a line under
its title.

The exit status is non-zero when any case failed or errored. A driver whose
directory is missing, whose build fails or that does not answer `hello` is
skipped, and the report says why.

### Reading the report

- **Summary** and a **capability matrix** (what each driver declares, and which
  output fields its library does not report).
- **Matrices** — vectors and negative per implementation; interop and
  relationship as sender × receiver, with a per-case grid.
- **Findings** — failures attributed to a root cause in
  [`findings.toml`](findings.toml), each with its spec reference, evidence and
  affected cases. Failures that match no entry are listed under *Not yet
  investigated*, grouped by the runner's own cause string; that is where a new
  disagreement appears. When you have investigated one, add an entry.
- **Warnings** (a correct rejection under an unexpected error code) and **skip
  reasons**.

## CI and the gate

`.github/workflows/conformance.yml` checks out all four implementation repos
next to the suite, builds every driver from source and runs the full matrix on
every push and pull request, weekly, and on demand (with a ref per repo). The
seed is the run id, so each run uses fresh keys and the report says which.

CI runs with `--gate known`. `make run` keeps the default `--gate all`, which is
non-zero on any failure. `known` is non-zero only when:

- a failure matches no finding in `findings.toml` — a new disagreement, which
  needs investigating and either a fix or a new finding;
- a failure matches a finding marked `fixed` — a regression;
- a driver did not build or start.

Findings that matched nothing are printed as a note, since the cause may have
been fixed upstream; mark them `fixed = "<where>"` so that a recurrence fails.
The report and driver logs are uploaded as the `conformance-report` artifact.

## Adding an implementation

1. Create `drivers/<name>/` with a program that reads one JSON request per line
   on stdin and writes one response per line on stdout, as specified in
   [`docs/driver-protocol.md`](docs/driver-protocol.md) (read §7, the
   clarifications, too). Logs go to stderr.
2. Answer `hello` with honest capabilities. Declare one-sided support as
   `cap:pack` / `cap:open`. When a request needs something the library cannot
   express, answer `unsupported` — do not fill the gap in the driver.
3. Add an entry to [`drivers.toml`](drivers.toml):

   ```toml
   [[driver]]
   name = "my-impl"
   description = "what library, which version, how it is linked"
   cwd = "drivers/my-impl"
   build = "…"      # optional; run by --build
   run = "…"        # started with `sh -c` in cwd
   enabled = true
   ```

4. `make build DRIVERS=my-impl && make run DRIVERS=my-impl,reference-rust`.
   Start with `--suites vectors`: if the Appendix A vectors do not open, nothing
   else will.

## The three bundled drivers

**affinidi-rust** — `affinidi-tsp` by path, `default-features = false,
features = ["pq"]`. Stateless ops use `message::direct` (`pack_padded`,
`pack_sealed_box`, `pack_signed_only`, `pack_pq`, `pack_referral_invite`,
`unpack_with`) and `message::routed`; endpoint ops use `TspAgent`. Not
expressible: ephemeral-key injection, a NULL ESSR field, caller nonce on `XPAD`,
padding with the sealed box / signed-only / PQ / routed / referral paths.
`open` does not surface the padding or the ESSR field.

**reference-rust** — `tsp_sdk = "=0.11.0"`, `default-features = false,
features = ["serialize", "nacl", "pq"]` (0.11.0 is the first release to emit
`YTSP-AAC`; 0.10.0 emitted `ABA`). Stateless ops use `crypto::seal_*`,
`crypto::open`, `crypto::sign`/`verify`; `seal_reproducibly` when a caller nonce
is given; `SecureStore` `SendOptions` when padding is (scs/ctl/pad only).
Endpoint ops use one `SecureStore` per endpoint. Not expressible:
`ikmE`/`skEm` injection (it reproduces from an RNG seed), referral packing
(only via the stateful parallel-relationship flow), signed-only payloads other
than application data, choosing the ESSR field. `open` does not surface the
version, nonce, padding or ESSR field.

**tsp-js** — imports `pnm-browser-plugin/packages/tsp-js/dist/index.js`
(override with `TSP_JS_DIR`). HPKE-Base only. The package deliberately ships its
relationship *rules* as pure functions and leaves storage to the wallet, so the
driver's endpoint holds a `Map` and applies only those functions
(`transition`, `resolveInviteRace`, `resolveCancel`, `admitsApplicationMessage`,
`canSend`); it adds no rule of its own.

## Scope and limits

- Byte-exact reproduction of the HPKE and sealed-box vectors needs
  `deterministic` (ephemeral injection), which no bundled library exposes; the
  signed-only vector has no randomness and is reproduced where signed-only
  packing is available.
- The runner contains a small text-domain CESR reader for two jobs only:
  reading expected values out of the printed vectors, and locating the regions
  of a message to corrupt. For signed-only messages it can also re-sign an
  edited message with the sender's key (Ed25519), which is what lets the
  negative suite reach checks that sit behind the signature. It implements no
  encryption, decryption or digest derivation.
- Relationship coverage is the direct form. Routed, nested and parallel
  relationship forming are not exercised through `endpoint.*`.
