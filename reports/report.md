# TSP 1.0 conformance report

Target **trustoverip/tswg-tsp-specification** at commit `ccda6b3` (`YTSP-BAA`). Generated 2026-10-22T18:22:13Z with key seed `20261022`.

> **Note:** TSP 1.0. Built from affinidi-tdk-rs main 7609754e (affinidi-tsp 0.3.0, test-vectors), vta-browser-plugin main a24e498f (@openvtc/vti-tsp-js 0.4.0), tsp_sdk 0.16.0 from crates.io, and affinidi-tsp-go / affinidi-tsp-dart main, which are still Rev 3 (YTSP-AAC). The Go and Dart drivers ran from the main checkout of this repository (cwd override); the Dart binary is a previous build, as its driver no longer builds from a fresh checkout (see finding dart-rev3).

Each case is **pass**, **fail** (with the diff), **skip** (a capability the implementation does not offer) or **error** (the harness or a driver broke). Cells read `✓ passed/ran` when nothing failed, `✗ passed/ran` when something did, `–` when every case skipped.

## Implementations

| Driver | Library | Version | Language | Status |
|---|---|---|---|---|
| affinidi-rust | affinidi-tsp | 0.3.0 | rust | ran |
| reference-rust | tsp_sdk | 0.16.0 | rust | ran |
| tsp-js | @openvtc/vti-tsp-js | 0.4.0 | typescript | ran |
| go | affinidi-tsp-go | 0.1.0 | go | ran |
| dart | affinidi-tsp-dart | 0.1.0 | dart | ran |

Local overrides in effect (`drivers.local.toml`, not committed):

- **affinidi-rust**: `AFFINIDI_TSP_PATH=~/devel/affinidi-tdk-rs-worktrees/main-tsp1/crates/messaging/affinidi-tsp`
- **tsp-js**: `TSP_JS_DIR=~/devel/pnm-browser-plugin-worktrees/main-tsp1/packages/tsp-js`
- **go**: `cwd`
- **dart**: `cwd`

## Summary

| Suite | Pass | Fail | Error | Skip |
|---|---:|---:|---:|---:|
| interop | 284 | 624 | 0 | 142 |
| negative | 258 | 3 | 0 | 229 |
| relationship | 154 | 96 | 0 | 0 |
| vectors | 91 | 74 | 0 | 25 |

## Capability matrix

As each driver declares in `hello`. `pack only`/`open only` mark one-sided support.

| Capability | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|---|---|---|---|---|
| `hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `signed-only` | ✓ | ✓ | — | ✓ | ✓ |
| `sealed-box` | ✓ | ✓ | — | ✓ | ✓ |
| `hpke-pq` | ✓ | ✓ | — | ✓ | ✓ |
| `payload.scs` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `payload.ctl` | ✓ | ✓ | open only | ✓ | ✓ |
| `payload.pad` | ✓ | ✓ | open only | ✓ | ✓ |
| `payload.rfi` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `payload.rfa` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `payload.rfd` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi.reply-path` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi.referral` | ✓ | open only | open only | ✓ | ✓ |
| `payload.hop` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `padding` | ✓ | ✓ | — | ✓ | ✓ |
| `payload-sender` | — | — | — | ✓ | ✓ |
| `deterministic` | ✓ | — | ✓ | ✓ | ✓ |
| `peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `endpoint` | ✓ | ✓ | ✓ | ✓ | ✓ |
| *not reported by `open`* | `payload.padding`, `payload.payloadSender` | `payload.nonce`, `payload.padding`, `payload.payloadSender`, `peek.version`, `version` | `payload.padding`, `payload.payloadSender` | — | — |

## Vectors matrix

| Case | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|:-:|:-:|:-:|:-:|:-:|
| `control-rfa-direct/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfa-direct/pack-exact` | ✓ | – | ✓ | ✗ | ✗ |
| `control-rfa-direct/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfd/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfd/pack-exact` | ✓ | – | ✓ | ✗ | ✗ |
| `control-rfd/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfi-direct/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfi-direct/pack-exact` | ✓ | – | ✓ | ✗ | ✗ |
| `control-rfi-direct/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfi-referral/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfi-referral/pack-exact` | ✓ | – | – | ✗ | ✗ |
| `control-rfi-referral/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `control-rfi-sealed-box/open` | ✓ | ✓ | – | ✗ | ✗ |
| `control-rfi-sealed-box/pack-exact` | ✓ | – | – | ✗ | ✗ |
| `control-rfi-sealed-box/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `direct-hpke-base/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `direct-hpke-base/pack-exact` | ✓ | – | ✓ | ✗ | ✗ |
| `direct-hpke-base/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `direct-hpke-base-pq/open` | ✓ | ✓ | – | ✗ | ✗ |
| `direct-hpke-base-pq/pack-exact` | – | – | – | – | – |
| `direct-hpke-base-pq/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `direct-sealed-box/open` | ✓ | ✓ | – | ✗ | ✗ |
| `direct-sealed-box/pack-exact` | ✓ | – | – | ✗ | ✗ |
| `direct-sealed-box/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `direct-signed-only/open` | ✓ | ✓ | – | ✗ | ✗ |
| `direct-signed-only/pack-exact` | ✓ | ✓ | – | ✗ | ✗ |
| `direct-signed-only/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `nested-direct/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `nested-direct/inner-open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `nested-direct/pack-exact` | ✓ | – | ✓ | ✗ | ✗ |
| `nested-direct/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `routed/open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `routed/inner-open` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `routed/pack-exact` | ✓ | – | ✓ | ✗ | ✗ |
| `routed/peek` | ✓ | ✓ | ✓ | ✗ | ✗ |
| `direct-signed-only/resigned/minor-BAB-accepted` | ✓ | ✓ | – | ✗ | ✗ |
| `direct-signed-only/resigned/minor-BBA-accepted` | ✓ | ✓ | – | ✗ | ✗ |
| `direct-signed-only/resigned/essr-sender-present-accepted` | ✓ | ✓ | – | ✗ | ✗ |

## Interop matrix

| packs ↓ / opens → | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|---|---|---|---|---|
| **affinidi-rust** | ✗ 40/42 | ✗ 40/42 | ✗ 32/34 | ✗ 1/42 | ✗ 1/42 |
| **reference-rust** | ✗ 37/39 | ✗ 37/39 | ✗ 29/31 | ✗ 1/39 | ✗ 1/39 |
| **tsp-js** | ✗ 21/23 | ✗ 21/23 | ✗ 21/23 | ✗ 1/23 | ✗ 1/23 |
| **go** | ✗ 0/42 | ✗ 0/42 | ✗ 0/34 | ✗ 0/42 | ✗ 0/42 |
| **dart** | ✗ 0/42 | ✗ 0/42 | ✗ 0/34 | ✗ 0/42 | ✗ 0/42 |

<details><summary>Per case (42 cases × 25 pairs)</summary>

| Case | affin→affin | affin→refer | affin→tsp | affin→go | affin→dart | refer→affin | refer→refer | refer→tsp | refer→go | refer→dart | tsp→affin | tsp→refer | tsp→tsp | tsp→go | tsp→dart | go→affin | go→refer | go→tsp | go→go | go→dart | dart→affin | dart→refer | dart→tsp | dart→go | dart→dart |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `scs/hpke-base/small` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/empty` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/2mib` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/padding-0` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/padding-1` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/padding-2` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/padding-3` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/padding-12285` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/padding-12286` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/signed-only` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `scs/sealed-box` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `scs/hpke-base/payload-sender-null-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/payload-sender-present` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-base/long-vid` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `scs/hpke-pq` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `ctl/hpke-base` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `pad/hpke-base` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `pad/hpke-base/padded` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/hpke-base/direct` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/hpke-base/reply-path` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/hpke-base/referral` | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/sealed-box` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `rfa/hpke-base/echo-own-rfi` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfa/hpke-base/answers-receiver-rfi` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfd/hpke-base` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/hpke-base/padded` | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/hpke-pq` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `rfa/sealed-box` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `rfd/sealed-box` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `hop/nested` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested/padded` | ✓ | ✓ | ✓ | ✗ | ✗ | – | – | – | – | – | – | – | – | – | – | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested/signed-only-inner` | ✓ | ✓ | – | ✗ | ✗ | ✓ | ✓ | – | ✗ | ✗ | – | – | – | – | – | ✗ | ✗ | – | ✗ | ✗ | ✗ | ✗ | – | ✗ | ✗ |
| `hop/routed-2-hops` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/routed-long-hop-list` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `rfi/hpke-base/long-reply-path` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/routed-12-hops` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/routed-17-hops` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested-mixed/inner-by-affinidi-rust` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested-mixed/inner-by-reference-rust` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested-mixed/inner-by-tsp-js` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested-mixed/inner-by-go` | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `hop/nested-mixed/inner-by-dart` | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

</details>

## Negative matrix

Aggregated over the message sources (the implementation's own output and the Appendix A vectors); see the JSON report for each source.

| Case | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|:-:|:-:|:-:|:-:|:-:|
| `flip-ciphertext` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `flip-signature` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `flip-envelope-sender` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `flip-version-minor` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `unknown-major-version` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `draft-version-rev3` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✗ 0/1 | ✗ 0/1 |
| `truncate-one-triplet` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `truncate-half` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `trailing-bytes` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `frame-count-too-large` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `frame-count-too-small` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `signature-index-nonzero` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `attachment-count-too-large` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `wrong-receiver-key` | ✓ 4/4 | ✓ 4/4 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `wrong-sender-key` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 | ✓ 1/1 | ✓ 1/1 |
| `digest-tampered` | ✓ 1/1 | – | – | – | – |
| `non-canonical-lead-byte` | ✓ 1/1 | ✓ 1/1 | – | – | – |
| `essr-sender-mismatch` | ✓ 1/1 | ✓ 1/1 | – | – | – |
| `payload-count-too-large` | ✓ 1/1 | ✓ 1/1 | – | – | – |
| `xscs-body-h-group-json` | ✓ 1/1 | ✓ 1/1 | – | – | – |
| `xscs-body-two-bytes-primitives` | ✓ 1/1 | ✓ 1/1 | – | – | – |
| `xscs-body-data-after-stream` | ✓ 1/1 | ✓ 1/1 | – | – | – |
| `essr-sender-null` | ✓ 1/1 | ✗ 0/1 | ✓ 1/1 | – | – |

## Relationship matrix

| A ↓ / B → | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|---|---|---|---|---|
| **affinidi-rust** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✗ 2/10 | ✗ 2/10 |
| **reference-rust** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✗ 2/10 | ✗ 2/10 |
| **tsp-js** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✗ 2/10 | ✗ 2/10 |
| **go** | ✗ 2/10 | ✗ 2/10 | ✗ 2/10 | ✓ 10/10 | ✓ 10/10 |
| **dart** | ✗ 2/10 | ✗ 2/10 | ✗ 2/10 | ✓ 10/10 | ✓ 10/10 |

<details><summary>Per case (10 cases × 25 pairs)</summary>

| Case | affin→affin | affin→refer | affin→tsp | affin→go | affin→dart | refer→affin | refer→refer | refer→tsp | refer→go | refer→dart | tsp→affin | tsp→refer | tsp→tsp | tsp→go | tsp→dart | go→affin | go→refer | go→tsp | go→go | go→dart | dart→affin | dart→refer | dart→tsp | dart→go | dart→dart |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `invite-accept-bidirectional` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `message-before-relationship-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-returns-to-none` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `cancel-by-inviter-naming-invite-digest` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `cancel-by-inviter-naming-accept-digest` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `cancel-by-accepter-naming-invite-digest` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `cancel-by-accepter-naming-accept-digest` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `rfi-race-lower-digest-wins` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `invite-race-raw-byte-order` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ |
| `accept-unknown-digest-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

</details>

## Findings

Failures attributed to an investigated root cause (`findings.toml`). *Kind*: `spec-violation` contradicts a MUST; `disagreement` — implementations differ where the specification is silent or ambiguous; `limit` — a bound the specification does not set; `api-gap` — the library cannot express what the protocol needs.

| # | Finding | Kind | Cases |
|---|---|---|---:|
| 1 | tsp_sdk 0.16.0 accepts a confidential payload whose ESSR sender is NULL | `spec-violation` | 1 |
| 2 | affinidi-tsp-go still speaks Rev 3 (YTSP-AAC, 0.2), not 1.0 | `spec-violation` | 431 |
| 3 | affinidi-tsp-dart still speaks Rev 3 (YTSP-AAC, 0.2), not 1.0 | `spec-violation` | 347 |
| 4 | a nested message whose inner message affinidi-tsp-go packed (Rev 3) is refused | `spec-violation` | 9 |
| 5 | a nested message whose inner message affinidi-tsp-dart packed (Rev 3) is refused | `spec-violation` | 9 |

### 1. tsp_sdk 0.16.0 accepts a confidential payload whose ESSR sender is NULL

**Kind:** `spec-violation` · **Spec:** §3.7 step 7 (1.0): "For both PKAE variants, retrieve the sender VID from the decrypted payload plaintext and verify that it matches VID_sndr." §3.2: "For all PKAE schemes, the VID_sndr MUST appear as a confidential payload field."

1.0 (tswg-tsp-specification#86) made the ESSR sender field mandatory in every confidential payload; Rev 3 let HPKE-Base write the NULL VID `4BAA`. tsp_sdk 0.16.0 moved its *send* side (`EssrSender` defaults to `Always`) but not its receive side: `crypto/tsp_hpke.rs` `open_payload` still reads the field as optional (`if let Some(id) = sender_identity && id != envelope.sender`), under a comment that still says "In HPKE-Base the sender-VID confidential field is optional", and `cesr/packet.rs` `decode_sender_identity` maps an empty field to `None`. The runner seals a 1.0 HPKE-Base XSCS with a NULL sender (`runner/src/seal.rs`), after a control naming the sender opens; tsp_sdk returns the payload. affinidi-tsp 0.3.0 and tsp-js 0.4.0 refuse it.

Observed in 1 case(s); for example `negative/sealed/hpke-base-scs/essr-sender-null` (runner-sealed → reference-rust):

```text
accepted a message with a NULL ESSR sender in a confidential payload (source runner-sealed); decoded payload type "scs"
```

<details><summary>Affected cases</summary>

- `negative/sealed/hpke-base-scs/essr-sender-null` runner-sealed → reference-rust

</details>

### 2. affinidi-tsp-go still speaks Rev 3 (YTSP-AAC, 0.2), not 1.0

**Kind:** `spec-violation` · **Spec:** §9.1: the current version is YTSP-BAA (1.0); MAJOR gates processability

affinidi-tsp-go `main` packs `YTSP-AAC` (`payload.go` `CurrentVersion`) and refuses MAJOR 1, so every case pairing it with a 1.0 implementation, and every 1.0 vector, fails on the version; within Rev 3 it may also write a NULL ESSR sender under HPKE-Base, which 1.0 forbids. Until the library moves to 1.0 every failure involving it is attributed here — remove this entry when it does, so its remaining failures are seen.

Observed in 431 case(s); for example `vectors/control-rfa-direct/open` (spec → go):

```text
go refused to open the spec's message: [version] tsp: version: unsupported TSP major version 1
```

<details><summary>Affected cases</summary>

- `vectors/control-rfa-direct/open` spec → go
- `vectors/control-rfa-direct/pack-exact` spec → go
- `vectors/control-rfa-direct/peek` spec → go
- `vectors/control-rfd/open` spec → go
- `vectors/control-rfd/pack-exact` spec → go
- `vectors/control-rfd/peek` spec → go
- `vectors/control-rfi-direct/open` spec → go
- `vectors/control-rfi-direct/pack-exact` spec → go
- `vectors/control-rfi-direct/peek` spec → go
- `vectors/control-rfi-referral/open` spec → go
- `vectors/control-rfi-referral/pack-exact` spec → go
- `vectors/control-rfi-referral/peek` spec → go
- `vectors/control-rfi-sealed-box/open` spec → go
- `vectors/control-rfi-sealed-box/pack-exact` spec → go
- `vectors/control-rfi-sealed-box/peek` spec → go
- `vectors/direct-hpke-base/open` spec → go
- `vectors/direct-hpke-base/pack-exact` spec → go
- `vectors/direct-hpke-base/peek` spec → go
- `vectors/direct-hpke-base-pq/open` spec → go
- `vectors/direct-hpke-base-pq/peek` spec → go
- `vectors/direct-sealed-box/open` spec → go
- `vectors/direct-sealed-box/pack-exact` spec → go
- `vectors/direct-sealed-box/peek` spec → go
- `vectors/direct-signed-only/open` spec → go
- `vectors/direct-signed-only/pack-exact` spec → go
- `vectors/direct-signed-only/peek` spec → go
- `vectors/nested-direct/open` spec → go
- `vectors/nested-direct/inner-open` spec → go
- `vectors/nested-direct/pack-exact` spec → go
- `vectors/nested-direct/peek` spec → go
- `vectors/routed/open` spec → go
- `vectors/routed/inner-open` spec → go
- `vectors/routed/pack-exact` spec → go
- `vectors/routed/peek` spec → go
- `vectors/direct-signed-only/resigned/minor-BAB-accepted` spec (re-signed) → go
- `vectors/direct-signed-only/resigned/minor-BBA-accepted` spec (re-signed) → go
- `vectors/direct-signed-only/resigned/essr-sender-present-accepted` spec (re-signed) → go
- `interop/scs/hpke-base/small` affinidi-rust → go
- `interop/scs/hpke-base/empty` affinidi-rust → go
- `interop/scs/hpke-base/2mib` affinidi-rust → go
- `interop/scs/hpke-base/padding-0` affinidi-rust → go
- `interop/scs/hpke-base/padding-1` affinidi-rust → go
- `interop/scs/hpke-base/padding-2` affinidi-rust → go
- `interop/scs/hpke-base/padding-3` affinidi-rust → go
- `interop/scs/hpke-base/padding-12285` affinidi-rust → go
- `interop/scs/hpke-base/padding-12286` affinidi-rust → go
- `interop/scs/signed-only` affinidi-rust → go
- `interop/scs/sealed-box` affinidi-rust → go
- `interop/scs/hpke-base/payload-sender-present` affinidi-rust → go
- `interop/scs/hpke-base/long-vid` affinidi-rust → go
- `interop/scs/hpke-pq` affinidi-rust → go
- `interop/ctl/hpke-base` affinidi-rust → go
- `interop/pad/hpke-base` affinidi-rust → go
- `interop/pad/hpke-base/padded` affinidi-rust → go
- `interop/rfi/hpke-base/direct` affinidi-rust → go
- `interop/rfi/hpke-base/reply-path` affinidi-rust → go
- `interop/rfi/hpke-base/referral` affinidi-rust → go
- `interop/rfi/sealed-box` affinidi-rust → go
- `interop/rfa/hpke-base/echo-own-rfi` affinidi-rust → go
- `interop/rfa/hpke-base/answers-receiver-rfi` affinidi-rust → go
- `interop/rfd/hpke-base` affinidi-rust → go
- `interop/rfi/hpke-base/padded` affinidi-rust → go
- `interop/rfi/hpke-pq` affinidi-rust → go
- `interop/rfa/sealed-box` affinidi-rust → go
- `interop/rfd/sealed-box` affinidi-rust → go
- `interop/hop/nested` affinidi-rust → go
- `interop/hop/nested/padded` affinidi-rust → go
- `interop/hop/nested/signed-only-inner` affinidi-rust → go
- `interop/hop/routed-2-hops` affinidi-rust → go
- `interop/hop/routed-long-hop-list` affinidi-rust → go
- `interop/rfi/hpke-base/long-reply-path` affinidi-rust → go
- `interop/hop/routed-12-hops` affinidi-rust → go
- `interop/hop/routed-17-hops` affinidi-rust → go
- `interop/scs/hpke-base/small` reference-rust → go
- `interop/scs/hpke-base/empty` reference-rust → go
- `interop/scs/hpke-base/2mib` reference-rust → go
- `interop/scs/hpke-base/padding-0` reference-rust → go
- `interop/scs/hpke-base/padding-1` reference-rust → go
- `interop/scs/hpke-base/padding-2` reference-rust → go
- `interop/scs/hpke-base/padding-3` reference-rust → go
- `interop/scs/hpke-base/padding-12285` reference-rust → go
- `interop/scs/hpke-base/padding-12286` reference-rust → go
- `interop/scs/signed-only` reference-rust → go
- `interop/scs/sealed-box` reference-rust → go
- `interop/scs/hpke-base/payload-sender-present` reference-rust → go
- `interop/scs/hpke-base/long-vid` reference-rust → go
- `interop/scs/hpke-pq` reference-rust → go
- `interop/ctl/hpke-base` reference-rust → go
- `interop/pad/hpke-base` reference-rust → go
- `interop/pad/hpke-base/padded` reference-rust → go
- `interop/rfi/hpke-base/direct` reference-rust → go
- `interop/rfi/hpke-base/reply-path` reference-rust → go
- `interop/rfi/sealed-box` reference-rust → go
- `interop/rfa/hpke-base/echo-own-rfi` reference-rust → go
- `interop/rfa/hpke-base/answers-receiver-rfi` reference-rust → go
- `interop/rfd/hpke-base` reference-rust → go
- `interop/rfi/hpke-pq` reference-rust → go
- `interop/rfa/sealed-box` reference-rust → go
- `interop/rfd/sealed-box` reference-rust → go
- `interop/hop/nested` reference-rust → go
- `interop/hop/nested/signed-only-inner` reference-rust → go
- `interop/hop/routed-2-hops` reference-rust → go
- `interop/hop/routed-long-hop-list` reference-rust → go
- `interop/rfi/hpke-base/long-reply-path` reference-rust → go
- `interop/hop/routed-12-hops` reference-rust → go
- `interop/hop/routed-17-hops` reference-rust → go
- `interop/scs/hpke-base/small` tsp-js → go
- `interop/scs/hpke-base/empty` tsp-js → go
- `interop/scs/hpke-base/2mib` tsp-js → go
- `interop/scs/hpke-base/padding-0` tsp-js → go
- `interop/scs/hpke-base/payload-sender-present` tsp-js → go
- `interop/scs/hpke-base/long-vid` tsp-js → go
- `interop/rfi/hpke-base/direct` tsp-js → go
- `interop/rfi/hpke-base/reply-path` tsp-js → go
- `interop/rfa/hpke-base/echo-own-rfi` tsp-js → go
- `interop/rfa/hpke-base/answers-receiver-rfi` tsp-js → go
- `interop/rfd/hpke-base` tsp-js → go
- `interop/hop/nested` tsp-js → go
- `interop/hop/routed-2-hops` tsp-js → go
- `interop/hop/routed-long-hop-list` tsp-js → go
- `interop/rfi/hpke-base/long-reply-path` tsp-js → go
- `interop/hop/routed-12-hops` tsp-js → go
- `interop/hop/routed-17-hops` tsp-js → go
- `interop/scs/hpke-base/small` go → affinidi-rust
- `interop/scs/hpke-base/empty` go → affinidi-rust
- `interop/scs/hpke-base/2mib` go → affinidi-rust
- `interop/scs/hpke-base/padding-0` go → affinidi-rust
- `interop/scs/hpke-base/padding-1` go → affinidi-rust
- `interop/scs/hpke-base/padding-2` go → affinidi-rust
- `interop/scs/hpke-base/padding-3` go → affinidi-rust
- `interop/scs/hpke-base/padding-12285` go → affinidi-rust
- `interop/scs/hpke-base/padding-12286` go → affinidi-rust
- `interop/scs/signed-only` go → affinidi-rust
- `interop/scs/sealed-box` go → affinidi-rust
- `interop/scs/hpke-base/payload-sender-null-refused` go → affinidi-rust
- `interop/scs/hpke-base/payload-sender-present` go → affinidi-rust
- `interop/scs/hpke-base/long-vid` go → affinidi-rust
- `interop/scs/hpke-pq` go → affinidi-rust
- `interop/ctl/hpke-base` go → affinidi-rust
- `interop/pad/hpke-base` go → affinidi-rust
- `interop/pad/hpke-base/padded` go → affinidi-rust
- `interop/rfi/hpke-base/direct` go → affinidi-rust
- `interop/rfi/hpke-base/reply-path` go → affinidi-rust
- `interop/rfi/hpke-base/referral` go → affinidi-rust
- `interop/rfi/sealed-box` go → affinidi-rust
- `interop/rfa/hpke-base/echo-own-rfi` go → affinidi-rust
- `interop/rfa/hpke-base/answers-receiver-rfi` go → affinidi-rust
- `interop/rfd/hpke-base` go → affinidi-rust
- `interop/rfi/hpke-base/padded` go → affinidi-rust
- `interop/rfi/hpke-pq` go → affinidi-rust
- `interop/rfa/sealed-box` go → affinidi-rust
- `interop/rfd/sealed-box` go → affinidi-rust
- `interop/hop/nested` go → affinidi-rust
- `interop/hop/nested/padded` go → affinidi-rust
- `interop/hop/nested/signed-only-inner` go → affinidi-rust
- `interop/hop/routed-2-hops` go → affinidi-rust
- `interop/hop/routed-long-hop-list` go → affinidi-rust
- `interop/rfi/hpke-base/long-reply-path` go → affinidi-rust
- `interop/hop/routed-12-hops` go → affinidi-rust
- `interop/hop/routed-17-hops` go → affinidi-rust
- `interop/scs/hpke-base/small` go → reference-rust
- `interop/scs/hpke-base/empty` go → reference-rust
- `interop/scs/hpke-base/2mib` go → reference-rust
- `interop/scs/hpke-base/padding-0` go → reference-rust
- `interop/scs/hpke-base/padding-1` go → reference-rust
- `interop/scs/hpke-base/padding-2` go → reference-rust
- `interop/scs/hpke-base/padding-3` go → reference-rust
- `interop/scs/hpke-base/padding-12285` go → reference-rust
- `interop/scs/hpke-base/padding-12286` go → reference-rust
- `interop/scs/signed-only` go → reference-rust
- `interop/scs/sealed-box` go → reference-rust
- `interop/scs/hpke-base/payload-sender-null-refused` go → reference-rust
- `interop/scs/hpke-base/payload-sender-present` go → reference-rust
- `interop/scs/hpke-base/long-vid` go → reference-rust
- `interop/scs/hpke-pq` go → reference-rust
- `interop/ctl/hpke-base` go → reference-rust
- `interop/pad/hpke-base` go → reference-rust
- `interop/pad/hpke-base/padded` go → reference-rust
- `interop/rfi/hpke-base/direct` go → reference-rust
- `interop/rfi/hpke-base/reply-path` go → reference-rust
- `interop/rfi/hpke-base/referral` go → reference-rust
- `interop/rfi/sealed-box` go → reference-rust
- `interop/rfa/hpke-base/echo-own-rfi` go → reference-rust
- `interop/rfa/hpke-base/answers-receiver-rfi` go → reference-rust
- `interop/rfd/hpke-base` go → reference-rust
- `interop/rfi/hpke-base/padded` go → reference-rust
- `interop/rfi/hpke-pq` go → reference-rust
- `interop/rfa/sealed-box` go → reference-rust
- `interop/rfd/sealed-box` go → reference-rust
- `interop/hop/nested` go → reference-rust
- `interop/hop/nested/padded` go → reference-rust
- `interop/hop/nested/signed-only-inner` go → reference-rust
- `interop/hop/routed-2-hops` go → reference-rust
- `interop/hop/routed-long-hop-list` go → reference-rust
- `interop/rfi/hpke-base/long-reply-path` go → reference-rust
- `interop/hop/routed-12-hops` go → reference-rust
- `interop/hop/routed-17-hops` go → reference-rust
- `interop/scs/hpke-base/small` go → tsp-js
- `interop/scs/hpke-base/empty` go → tsp-js
- `interop/scs/hpke-base/2mib` go → tsp-js
- `interop/scs/hpke-base/padding-0` go → tsp-js
- `interop/scs/hpke-base/padding-1` go → tsp-js
- `interop/scs/hpke-base/padding-2` go → tsp-js
- `interop/scs/hpke-base/padding-3` go → tsp-js
- `interop/scs/hpke-base/padding-12285` go → tsp-js
- `interop/scs/hpke-base/padding-12286` go → tsp-js
- `interop/scs/hpke-base/payload-sender-null-refused` go → tsp-js
- `interop/scs/hpke-base/payload-sender-present` go → tsp-js
- `interop/scs/hpke-base/long-vid` go → tsp-js
- `interop/ctl/hpke-base` go → tsp-js
- `interop/pad/hpke-base` go → tsp-js
- `interop/pad/hpke-base/padded` go → tsp-js
- `interop/rfi/hpke-base/direct` go → tsp-js
- `interop/rfi/hpke-base/reply-path` go → tsp-js
- `interop/rfi/hpke-base/referral` go → tsp-js
- `interop/rfa/hpke-base/echo-own-rfi` go → tsp-js
- `interop/rfa/hpke-base/answers-receiver-rfi` go → tsp-js
- `interop/rfd/hpke-base` go → tsp-js
- `interop/rfi/hpke-base/padded` go → tsp-js
- `interop/hop/nested` go → tsp-js
- `interop/hop/nested/padded` go → tsp-js
- `interop/hop/routed-2-hops` go → tsp-js
- `interop/hop/routed-long-hop-list` go → tsp-js
- `interop/rfi/hpke-base/long-reply-path` go → tsp-js
- `interop/hop/routed-12-hops` go → tsp-js
- `interop/hop/routed-17-hops` go → tsp-js
- `interop/scs/hpke-base/small` go → go
- `interop/scs/hpke-base/empty` go → go
- `interop/scs/hpke-base/2mib` go → go
- `interop/scs/hpke-base/padding-0` go → go
- `interop/scs/hpke-base/padding-1` go → go
- `interop/scs/hpke-base/padding-2` go → go
- `interop/scs/hpke-base/padding-3` go → go
- `interop/scs/hpke-base/padding-12285` go → go
- `interop/scs/hpke-base/padding-12286` go → go
- `interop/scs/signed-only` go → go
- `interop/scs/sealed-box` go → go
- `interop/scs/hpke-base/payload-sender-null-refused` go → go
- `interop/scs/hpke-base/payload-sender-present` go → go
- `interop/scs/hpke-base/long-vid` go → go
- `interop/scs/hpke-pq` go → go
- `interop/ctl/hpke-base` go → go
- `interop/pad/hpke-base` go → go
- `interop/pad/hpke-base/padded` go → go
- `interop/rfi/hpke-base/direct` go → go
- `interop/rfi/hpke-base/reply-path` go → go
- `interop/rfi/hpke-base/referral` go → go
- `interop/rfi/sealed-box` go → go
- `interop/rfa/hpke-base/echo-own-rfi` go → go
- `interop/rfa/hpke-base/answers-receiver-rfi` go → go
- `interop/rfd/hpke-base` go → go
- `interop/rfi/hpke-base/padded` go → go
- `interop/rfi/hpke-pq` go → go
- `interop/rfa/sealed-box` go → go
- `interop/rfd/sealed-box` go → go
- `interop/hop/nested` go → go
- `interop/hop/nested/padded` go → go
- `interop/hop/nested/signed-only-inner` go → go
- `interop/hop/routed-2-hops` go → go
- `interop/hop/routed-long-hop-list` go → go
- `interop/rfi/hpke-base/long-reply-path` go → go
- `interop/hop/routed-12-hops` go → go
- `interop/hop/routed-17-hops` go → go
- `interop/scs/hpke-base/small` go → dart
- `interop/scs/hpke-base/empty` go → dart
- `interop/scs/hpke-base/2mib` go → dart
- `interop/scs/hpke-base/padding-0` go → dart
- `interop/scs/hpke-base/padding-1` go → dart
- `interop/scs/hpke-base/padding-2` go → dart
- `interop/scs/hpke-base/padding-3` go → dart
- `interop/scs/hpke-base/padding-12285` go → dart
- `interop/scs/hpke-base/padding-12286` go → dart
- `interop/scs/signed-only` go → dart
- `interop/scs/sealed-box` go → dart
- `interop/scs/hpke-base/payload-sender-null-refused` go → dart
- `interop/scs/hpke-base/payload-sender-present` go → dart
- `interop/scs/hpke-base/long-vid` go → dart
- `interop/scs/hpke-pq` go → dart
- `interop/ctl/hpke-base` go → dart
- `interop/pad/hpke-base` go → dart
- `interop/pad/hpke-base/padded` go → dart
- `interop/rfi/hpke-base/direct` go → dart
- `interop/rfi/hpke-base/reply-path` go → dart
- `interop/rfi/hpke-base/referral` go → dart
- `interop/rfi/sealed-box` go → dart
- `interop/rfa/hpke-base/echo-own-rfi` go → dart
- `interop/rfa/hpke-base/answers-receiver-rfi` go → dart
- `interop/rfd/hpke-base` go → dart
- `interop/rfi/hpke-base/padded` go → dart
- `interop/rfi/hpke-pq` go → dart
- `interop/rfa/sealed-box` go → dart
- `interop/rfd/sealed-box` go → dart
- `interop/hop/nested` go → dart
- `interop/hop/nested/padded` go → dart
- `interop/hop/nested/signed-only-inner` go → dart
- `interop/hop/routed-2-hops` go → dart
- `interop/hop/routed-long-hop-list` go → dart
- `interop/rfi/hpke-base/long-reply-path` go → dart
- `interop/hop/routed-12-hops` go → dart
- `interop/hop/routed-17-hops` go → dart
- `interop/scs/hpke-base/small` dart → go
- `interop/scs/hpke-base/empty` dart → go
- `interop/scs/hpke-base/2mib` dart → go
- `interop/scs/hpke-base/padding-0` dart → go
- `interop/scs/hpke-base/padding-1` dart → go
- `interop/scs/hpke-base/padding-2` dart → go
- `interop/scs/hpke-base/padding-3` dart → go
- `interop/scs/hpke-base/padding-12285` dart → go
- `interop/scs/hpke-base/padding-12286` dart → go
- `interop/scs/signed-only` dart → go
- `interop/scs/sealed-box` dart → go
- `interop/scs/hpke-base/payload-sender-null-refused` dart → go
- `interop/scs/hpke-base/payload-sender-present` dart → go
- `interop/scs/hpke-base/long-vid` dart → go
- `interop/scs/hpke-pq` dart → go
- `interop/ctl/hpke-base` dart → go
- `interop/pad/hpke-base` dart → go
- `interop/pad/hpke-base/padded` dart → go
- `interop/rfi/hpke-base/direct` dart → go
- `interop/rfi/hpke-base/reply-path` dart → go
- `interop/rfi/hpke-base/referral` dart → go
- `interop/rfi/sealed-box` dart → go
- `interop/rfa/hpke-base/echo-own-rfi` dart → go
- `interop/rfa/hpke-base/answers-receiver-rfi` dart → go
- `interop/rfd/hpke-base` dart → go
- `interop/rfi/hpke-base/padded` dart → go
- `interop/rfi/hpke-pq` dart → go
- `interop/rfa/sealed-box` dart → go
- `interop/rfd/sealed-box` dart → go
- `interop/hop/nested` dart → go
- `interop/hop/nested/padded` dart → go
- `interop/hop/nested/signed-only-inner` dart → go
- `interop/hop/routed-2-hops` dart → go
- `interop/hop/routed-long-hop-list` dart → go
- `interop/rfi/hpke-base/long-reply-path` dart → go
- `interop/hop/routed-12-hops` dart → go
- `interop/hop/routed-17-hops` dart → go
- `interop/hop/nested-mixed/inner-by-affinidi-rust` affinidi-rust → go
- `interop/hop/nested-mixed/inner-by-affinidi-rust` reference-rust → go
- `interop/hop/nested-mixed/inner-by-affinidi-rust` tsp-js → go
- `interop/hop/nested-mixed/inner-by-affinidi-rust` go → affinidi-rust
- `interop/hop/nested-mixed/inner-by-affinidi-rust` go → reference-rust
- `interop/hop/nested-mixed/inner-by-affinidi-rust` go → tsp-js
- `interop/hop/nested-mixed/inner-by-affinidi-rust` go → go
- `interop/hop/nested-mixed/inner-by-affinidi-rust` go → dart
- `interop/hop/nested-mixed/inner-by-affinidi-rust` dart → go
- `interop/hop/nested-mixed/inner-by-reference-rust` affinidi-rust → go
- `interop/hop/nested-mixed/inner-by-reference-rust` reference-rust → go
- `interop/hop/nested-mixed/inner-by-reference-rust` tsp-js → go
- `interop/hop/nested-mixed/inner-by-reference-rust` go → affinidi-rust
- `interop/hop/nested-mixed/inner-by-reference-rust` go → reference-rust
- `interop/hop/nested-mixed/inner-by-reference-rust` go → tsp-js
- `interop/hop/nested-mixed/inner-by-reference-rust` go → go
- `interop/hop/nested-mixed/inner-by-reference-rust` go → dart
- `interop/hop/nested-mixed/inner-by-reference-rust` dart → go
- `interop/hop/nested-mixed/inner-by-tsp-js` affinidi-rust → go
- `interop/hop/nested-mixed/inner-by-tsp-js` reference-rust → go
- `interop/hop/nested-mixed/inner-by-tsp-js` tsp-js → go
- `interop/hop/nested-mixed/inner-by-tsp-js` go → affinidi-rust
- `interop/hop/nested-mixed/inner-by-tsp-js` go → reference-rust
- `interop/hop/nested-mixed/inner-by-tsp-js` go → tsp-js
- `interop/hop/nested-mixed/inner-by-tsp-js` go → go
- `interop/hop/nested-mixed/inner-by-tsp-js` go → dart
- `interop/hop/nested-mixed/inner-by-tsp-js` dart → go
- `interop/hop/nested-mixed/inner-by-go` affinidi-rust → go
- `interop/hop/nested-mixed/inner-by-go` reference-rust → go
- `interop/hop/nested-mixed/inner-by-go` tsp-js → go
- `interop/hop/nested-mixed/inner-by-go` go → affinidi-rust
- `interop/hop/nested-mixed/inner-by-go` go → reference-rust
- `interop/hop/nested-mixed/inner-by-go` go → tsp-js
- `interop/hop/nested-mixed/inner-by-go` go → go
- `interop/hop/nested-mixed/inner-by-go` go → dart
- `interop/hop/nested-mixed/inner-by-go` dart → go
- `interop/hop/nested-mixed/inner-by-dart` affinidi-rust → go
- `interop/hop/nested-mixed/inner-by-dart` reference-rust → go
- `interop/hop/nested-mixed/inner-by-dart` tsp-js → go
- `interop/hop/nested-mixed/inner-by-dart` go → affinidi-rust
- `interop/hop/nested-mixed/inner-by-dart` go → reference-rust
- `interop/hop/nested-mixed/inner-by-dart` go → tsp-js
- `interop/hop/nested-mixed/inner-by-dart` go → go
- `interop/hop/nested-mixed/inner-by-dart` go → dart
- `interop/hop/nested-mixed/inner-by-dart` dart → go
- `negative/self/hpke-base/draft-version-rev3` self/hpke-base → go
- `relationship/invite-accept-bidirectional` affinidi-rust → go
- `relationship/cancel-returns-to-none` affinidi-rust → go
- `relationship/cancel-by-inviter-naming-invite-digest` affinidi-rust → go
- `relationship/cancel-by-inviter-naming-accept-digest` affinidi-rust → go
- `relationship/cancel-by-accepter-naming-invite-digest` affinidi-rust → go
- `relationship/cancel-by-accepter-naming-accept-digest` affinidi-rust → go
- `relationship/rfi-race-lower-digest-wins` affinidi-rust → go
- `relationship/invite-race-raw-byte-order` affinidi-rust → go
- `relationship/invite-accept-bidirectional` reference-rust → go
- `relationship/cancel-returns-to-none` reference-rust → go
- `relationship/cancel-by-inviter-naming-invite-digest` reference-rust → go
- `relationship/cancel-by-inviter-naming-accept-digest` reference-rust → go
- `relationship/cancel-by-accepter-naming-invite-digest` reference-rust → go
- `relationship/cancel-by-accepter-naming-accept-digest` reference-rust → go
- `relationship/rfi-race-lower-digest-wins` reference-rust → go
- `relationship/invite-race-raw-byte-order` reference-rust → go
- `relationship/invite-accept-bidirectional` tsp-js → go
- `relationship/cancel-returns-to-none` tsp-js → go
- `relationship/cancel-by-inviter-naming-invite-digest` tsp-js → go
- `relationship/cancel-by-inviter-naming-accept-digest` tsp-js → go
- `relationship/cancel-by-accepter-naming-invite-digest` tsp-js → go
- `relationship/cancel-by-accepter-naming-accept-digest` tsp-js → go
- `relationship/rfi-race-lower-digest-wins` tsp-js → go
- `relationship/invite-race-raw-byte-order` tsp-js → go
- `relationship/invite-accept-bidirectional` go → affinidi-rust
- `relationship/cancel-returns-to-none` go → affinidi-rust
- `relationship/cancel-by-inviter-naming-invite-digest` go → affinidi-rust
- `relationship/cancel-by-inviter-naming-accept-digest` go → affinidi-rust
- `relationship/cancel-by-accepter-naming-invite-digest` go → affinidi-rust
- `relationship/cancel-by-accepter-naming-accept-digest` go → affinidi-rust
- `relationship/rfi-race-lower-digest-wins` go → affinidi-rust
- `relationship/invite-race-raw-byte-order` go → affinidi-rust
- `relationship/invite-accept-bidirectional` go → reference-rust
- `relationship/cancel-returns-to-none` go → reference-rust
- `relationship/cancel-by-inviter-naming-invite-digest` go → reference-rust
- `relationship/cancel-by-inviter-naming-accept-digest` go → reference-rust
- `relationship/cancel-by-accepter-naming-invite-digest` go → reference-rust
- `relationship/cancel-by-accepter-naming-accept-digest` go → reference-rust
- `relationship/rfi-race-lower-digest-wins` go → reference-rust
- `relationship/invite-race-raw-byte-order` go → reference-rust
- `relationship/invite-accept-bidirectional` go → tsp-js
- `relationship/cancel-returns-to-none` go → tsp-js
- `relationship/cancel-by-inviter-naming-invite-digest` go → tsp-js
- `relationship/cancel-by-inviter-naming-accept-digest` go → tsp-js
- `relationship/cancel-by-accepter-naming-invite-digest` go → tsp-js
- `relationship/cancel-by-accepter-naming-accept-digest` go → tsp-js
- `relationship/rfi-race-lower-digest-wins` go → tsp-js
- `relationship/invite-race-raw-byte-order` go → tsp-js

</details>

### 3. affinidi-tsp-dart still speaks Rev 3 (YTSP-AAC, 0.2), not 1.0

**Kind:** `spec-violation` · **Spec:** §9.1: the current version is YTSP-BAA (1.0); MAJOR gates processability

affinidi-tsp-dart `main` packs `YTSP-AAC` and refuses MAJOR 1, so every case pairing it with a 1.0 implementation, and every 1.0 vector, fails on the version. Separately, its driver no longer builds from a fresh checkout: the library became a workspace (`packages/affinidi_tsp`), and `drivers/dart/pubspec.yaml` still points at the repository root. Until the library moves to 1.0 every failure involving it is attributed here — remove this entry when it does.

Observed in 347 case(s); for example `vectors/control-rfa-direct/open` (spec → dart):

```text
dart refused to open the spec's message: [version] unsupported TSP version 1.0
```

<details><summary>Affected cases</summary>

- `vectors/control-rfa-direct/open` spec → dart
- `vectors/control-rfa-direct/pack-exact` spec → dart
- `vectors/control-rfa-direct/peek` spec → dart
- `vectors/control-rfd/open` spec → dart
- `vectors/control-rfd/pack-exact` spec → dart
- `vectors/control-rfd/peek` spec → dart
- `vectors/control-rfi-direct/open` spec → dart
- `vectors/control-rfi-direct/pack-exact` spec → dart
- `vectors/control-rfi-direct/peek` spec → dart
- `vectors/control-rfi-referral/open` spec → dart
- `vectors/control-rfi-referral/pack-exact` spec → dart
- `vectors/control-rfi-referral/peek` spec → dart
- `vectors/control-rfi-sealed-box/open` spec → dart
- `vectors/control-rfi-sealed-box/pack-exact` spec → dart
- `vectors/control-rfi-sealed-box/peek` spec → dart
- `vectors/direct-hpke-base/open` spec → dart
- `vectors/direct-hpke-base/pack-exact` spec → dart
- `vectors/direct-hpke-base/peek` spec → dart
- `vectors/direct-hpke-base-pq/open` spec → dart
- `vectors/direct-hpke-base-pq/peek` spec → dart
- `vectors/direct-sealed-box/open` spec → dart
- `vectors/direct-sealed-box/pack-exact` spec → dart
- `vectors/direct-sealed-box/peek` spec → dart
- `vectors/direct-signed-only/open` spec → dart
- `vectors/direct-signed-only/pack-exact` spec → dart
- `vectors/direct-signed-only/peek` spec → dart
- `vectors/nested-direct/open` spec → dart
- `vectors/nested-direct/inner-open` spec → dart
- `vectors/nested-direct/pack-exact` spec → dart
- `vectors/nested-direct/peek` spec → dart
- `vectors/routed/open` spec → dart
- `vectors/routed/inner-open` spec → dart
- `vectors/routed/pack-exact` spec → dart
- `vectors/routed/peek` spec → dart
- `vectors/direct-signed-only/resigned/minor-BAB-accepted` spec (re-signed) → dart
- `vectors/direct-signed-only/resigned/minor-BBA-accepted` spec (re-signed) → dart
- `vectors/direct-signed-only/resigned/essr-sender-present-accepted` spec (re-signed) → dart
- `interop/scs/hpke-base/small` affinidi-rust → dart
- `interop/scs/hpke-base/empty` affinidi-rust → dart
- `interop/scs/hpke-base/2mib` affinidi-rust → dart
- `interop/scs/hpke-base/padding-0` affinidi-rust → dart
- `interop/scs/hpke-base/padding-1` affinidi-rust → dart
- `interop/scs/hpke-base/padding-2` affinidi-rust → dart
- `interop/scs/hpke-base/padding-3` affinidi-rust → dart
- `interop/scs/hpke-base/padding-12285` affinidi-rust → dart
- `interop/scs/hpke-base/padding-12286` affinidi-rust → dart
- `interop/scs/signed-only` affinidi-rust → dart
- `interop/scs/sealed-box` affinidi-rust → dart
- `interop/scs/hpke-base/payload-sender-present` affinidi-rust → dart
- `interop/scs/hpke-base/long-vid` affinidi-rust → dart
- `interop/scs/hpke-pq` affinidi-rust → dart
- `interop/ctl/hpke-base` affinidi-rust → dart
- `interop/pad/hpke-base` affinidi-rust → dart
- `interop/pad/hpke-base/padded` affinidi-rust → dart
- `interop/rfi/hpke-base/direct` affinidi-rust → dart
- `interop/rfi/hpke-base/reply-path` affinidi-rust → dart
- `interop/rfi/hpke-base/referral` affinidi-rust → dart
- `interop/rfi/sealed-box` affinidi-rust → dart
- `interop/rfa/hpke-base/echo-own-rfi` affinidi-rust → dart
- `interop/rfa/hpke-base/answers-receiver-rfi` affinidi-rust → dart
- `interop/rfd/hpke-base` affinidi-rust → dart
- `interop/rfi/hpke-base/padded` affinidi-rust → dart
- `interop/rfi/hpke-pq` affinidi-rust → dart
- `interop/rfa/sealed-box` affinidi-rust → dart
- `interop/rfd/sealed-box` affinidi-rust → dart
- `interop/hop/nested` affinidi-rust → dart
- `interop/hop/nested/padded` affinidi-rust → dart
- `interop/hop/nested/signed-only-inner` affinidi-rust → dart
- `interop/hop/routed-2-hops` affinidi-rust → dart
- `interop/hop/routed-long-hop-list` affinidi-rust → dart
- `interop/rfi/hpke-base/long-reply-path` affinidi-rust → dart
- `interop/hop/routed-12-hops` affinidi-rust → dart
- `interop/hop/routed-17-hops` affinidi-rust → dart
- `interop/scs/hpke-base/small` reference-rust → dart
- `interop/scs/hpke-base/empty` reference-rust → dart
- `interop/scs/hpke-base/2mib` reference-rust → dart
- `interop/scs/hpke-base/padding-0` reference-rust → dart
- `interop/scs/hpke-base/padding-1` reference-rust → dart
- `interop/scs/hpke-base/padding-2` reference-rust → dart
- `interop/scs/hpke-base/padding-3` reference-rust → dart
- `interop/scs/hpke-base/padding-12285` reference-rust → dart
- `interop/scs/hpke-base/padding-12286` reference-rust → dart
- `interop/scs/signed-only` reference-rust → dart
- `interop/scs/sealed-box` reference-rust → dart
- `interop/scs/hpke-base/payload-sender-present` reference-rust → dart
- `interop/scs/hpke-base/long-vid` reference-rust → dart
- `interop/scs/hpke-pq` reference-rust → dart
- `interop/ctl/hpke-base` reference-rust → dart
- `interop/pad/hpke-base` reference-rust → dart
- `interop/pad/hpke-base/padded` reference-rust → dart
- `interop/rfi/hpke-base/direct` reference-rust → dart
- `interop/rfi/hpke-base/reply-path` reference-rust → dart
- `interop/rfi/sealed-box` reference-rust → dart
- `interop/rfa/hpke-base/echo-own-rfi` reference-rust → dart
- `interop/rfa/hpke-base/answers-receiver-rfi` reference-rust → dart
- `interop/rfd/hpke-base` reference-rust → dart
- `interop/rfi/hpke-pq` reference-rust → dart
- `interop/rfa/sealed-box` reference-rust → dart
- `interop/rfd/sealed-box` reference-rust → dart
- `interop/hop/nested` reference-rust → dart
- `interop/hop/nested/signed-only-inner` reference-rust → dart
- `interop/hop/routed-2-hops` reference-rust → dart
- `interop/hop/routed-long-hop-list` reference-rust → dart
- `interop/rfi/hpke-base/long-reply-path` reference-rust → dart
- `interop/hop/routed-12-hops` reference-rust → dart
- `interop/hop/routed-17-hops` reference-rust → dart
- `interop/scs/hpke-base/small` tsp-js → dart
- `interop/scs/hpke-base/empty` tsp-js → dart
- `interop/scs/hpke-base/2mib` tsp-js → dart
- `interop/scs/hpke-base/padding-0` tsp-js → dart
- `interop/scs/hpke-base/payload-sender-present` tsp-js → dart
- `interop/scs/hpke-base/long-vid` tsp-js → dart
- `interop/rfi/hpke-base/direct` tsp-js → dart
- `interop/rfi/hpke-base/reply-path` tsp-js → dart
- `interop/rfa/hpke-base/echo-own-rfi` tsp-js → dart
- `interop/rfa/hpke-base/answers-receiver-rfi` tsp-js → dart
- `interop/rfd/hpke-base` tsp-js → dart
- `interop/hop/nested` tsp-js → dart
- `interop/hop/routed-2-hops` tsp-js → dart
- `interop/hop/routed-long-hop-list` tsp-js → dart
- `interop/rfi/hpke-base/long-reply-path` tsp-js → dart
- `interop/hop/routed-12-hops` tsp-js → dart
- `interop/hop/routed-17-hops` tsp-js → dart
- `interop/scs/hpke-base/small` dart → affinidi-rust
- `interop/scs/hpke-base/empty` dart → affinidi-rust
- `interop/scs/hpke-base/2mib` dart → affinidi-rust
- `interop/scs/hpke-base/padding-0` dart → affinidi-rust
- `interop/scs/hpke-base/padding-1` dart → affinidi-rust
- `interop/scs/hpke-base/padding-2` dart → affinidi-rust
- `interop/scs/hpke-base/padding-3` dart → affinidi-rust
- `interop/scs/hpke-base/padding-12285` dart → affinidi-rust
- `interop/scs/hpke-base/padding-12286` dart → affinidi-rust
- `interop/scs/signed-only` dart → affinidi-rust
- `interop/scs/sealed-box` dart → affinidi-rust
- `interop/scs/hpke-base/payload-sender-null-refused` dart → affinidi-rust
- `interop/scs/hpke-base/payload-sender-present` dart → affinidi-rust
- `interop/scs/hpke-base/long-vid` dart → affinidi-rust
- `interop/scs/hpke-pq` dart → affinidi-rust
- `interop/ctl/hpke-base` dart → affinidi-rust
- `interop/pad/hpke-base` dart → affinidi-rust
- `interop/pad/hpke-base/padded` dart → affinidi-rust
- `interop/rfi/hpke-base/direct` dart → affinidi-rust
- `interop/rfi/hpke-base/reply-path` dart → affinidi-rust
- `interop/rfi/hpke-base/referral` dart → affinidi-rust
- `interop/rfi/sealed-box` dart → affinidi-rust
- `interop/rfa/hpke-base/echo-own-rfi` dart → affinidi-rust
- `interop/rfa/hpke-base/answers-receiver-rfi` dart → affinidi-rust
- `interop/rfd/hpke-base` dart → affinidi-rust
- `interop/rfi/hpke-base/padded` dart → affinidi-rust
- `interop/rfi/hpke-pq` dart → affinidi-rust
- `interop/rfa/sealed-box` dart → affinidi-rust
- `interop/rfd/sealed-box` dart → affinidi-rust
- `interop/hop/nested` dart → affinidi-rust
- `interop/hop/nested/padded` dart → affinidi-rust
- `interop/hop/nested/signed-only-inner` dart → affinidi-rust
- `interop/hop/routed-2-hops` dart → affinidi-rust
- `interop/hop/routed-long-hop-list` dart → affinidi-rust
- `interop/rfi/hpke-base/long-reply-path` dart → affinidi-rust
- `interop/hop/routed-12-hops` dart → affinidi-rust
- `interop/hop/routed-17-hops` dart → affinidi-rust
- `interop/scs/hpke-base/small` dart → reference-rust
- `interop/scs/hpke-base/empty` dart → reference-rust
- `interop/scs/hpke-base/2mib` dart → reference-rust
- `interop/scs/hpke-base/padding-0` dart → reference-rust
- `interop/scs/hpke-base/padding-1` dart → reference-rust
- `interop/scs/hpke-base/padding-2` dart → reference-rust
- `interop/scs/hpke-base/padding-3` dart → reference-rust
- `interop/scs/hpke-base/padding-12285` dart → reference-rust
- `interop/scs/hpke-base/padding-12286` dart → reference-rust
- `interop/scs/signed-only` dart → reference-rust
- `interop/scs/sealed-box` dart → reference-rust
- `interop/scs/hpke-base/payload-sender-null-refused` dart → reference-rust
- `interop/scs/hpke-base/payload-sender-present` dart → reference-rust
- `interop/scs/hpke-base/long-vid` dart → reference-rust
- `interop/scs/hpke-pq` dart → reference-rust
- `interop/ctl/hpke-base` dart → reference-rust
- `interop/pad/hpke-base` dart → reference-rust
- `interop/pad/hpke-base/padded` dart → reference-rust
- `interop/rfi/hpke-base/direct` dart → reference-rust
- `interop/rfi/hpke-base/reply-path` dart → reference-rust
- `interop/rfi/hpke-base/referral` dart → reference-rust
- `interop/rfi/sealed-box` dart → reference-rust
- `interop/rfa/hpke-base/echo-own-rfi` dart → reference-rust
- `interop/rfa/hpke-base/answers-receiver-rfi` dart → reference-rust
- `interop/rfd/hpke-base` dart → reference-rust
- `interop/rfi/hpke-base/padded` dart → reference-rust
- `interop/rfi/hpke-pq` dart → reference-rust
- `interop/rfa/sealed-box` dart → reference-rust
- `interop/rfd/sealed-box` dart → reference-rust
- `interop/hop/nested` dart → reference-rust
- `interop/hop/nested/padded` dart → reference-rust
- `interop/hop/nested/signed-only-inner` dart → reference-rust
- `interop/hop/routed-2-hops` dart → reference-rust
- `interop/hop/routed-long-hop-list` dart → reference-rust
- `interop/rfi/hpke-base/long-reply-path` dart → reference-rust
- `interop/hop/routed-12-hops` dart → reference-rust
- `interop/hop/routed-17-hops` dart → reference-rust
- `interop/scs/hpke-base/small` dart → tsp-js
- `interop/scs/hpke-base/empty` dart → tsp-js
- `interop/scs/hpke-base/2mib` dart → tsp-js
- `interop/scs/hpke-base/padding-0` dart → tsp-js
- `interop/scs/hpke-base/padding-1` dart → tsp-js
- `interop/scs/hpke-base/padding-2` dart → tsp-js
- `interop/scs/hpke-base/padding-3` dart → tsp-js
- `interop/scs/hpke-base/padding-12285` dart → tsp-js
- `interop/scs/hpke-base/padding-12286` dart → tsp-js
- `interop/scs/hpke-base/payload-sender-null-refused` dart → tsp-js
- `interop/scs/hpke-base/payload-sender-present` dart → tsp-js
- `interop/scs/hpke-base/long-vid` dart → tsp-js
- `interop/ctl/hpke-base` dart → tsp-js
- `interop/pad/hpke-base` dart → tsp-js
- `interop/pad/hpke-base/padded` dart → tsp-js
- `interop/rfi/hpke-base/direct` dart → tsp-js
- `interop/rfi/hpke-base/reply-path` dart → tsp-js
- `interop/rfi/hpke-base/referral` dart → tsp-js
- `interop/rfa/hpke-base/echo-own-rfi` dart → tsp-js
- `interop/rfa/hpke-base/answers-receiver-rfi` dart → tsp-js
- `interop/rfd/hpke-base` dart → tsp-js
- `interop/rfi/hpke-base/padded` dart → tsp-js
- `interop/hop/nested` dart → tsp-js
- `interop/hop/nested/padded` dart → tsp-js
- `interop/hop/routed-2-hops` dart → tsp-js
- `interop/hop/routed-long-hop-list` dart → tsp-js
- `interop/rfi/hpke-base/long-reply-path` dart → tsp-js
- `interop/hop/routed-12-hops` dart → tsp-js
- `interop/hop/routed-17-hops` dart → tsp-js
- `interop/scs/hpke-base/small` dart → dart
- `interop/scs/hpke-base/empty` dart → dart
- `interop/scs/hpke-base/2mib` dart → dart
- `interop/scs/hpke-base/padding-0` dart → dart
- `interop/scs/hpke-base/padding-1` dart → dart
- `interop/scs/hpke-base/padding-2` dart → dart
- `interop/scs/hpke-base/padding-3` dart → dart
- `interop/scs/hpke-base/padding-12285` dart → dart
- `interop/scs/hpke-base/padding-12286` dart → dart
- `interop/scs/signed-only` dart → dart
- `interop/scs/sealed-box` dart → dart
- `interop/scs/hpke-base/payload-sender-null-refused` dart → dart
- `interop/scs/hpke-base/payload-sender-present` dart → dart
- `interop/scs/hpke-base/long-vid` dart → dart
- `interop/scs/hpke-pq` dart → dart
- `interop/ctl/hpke-base` dart → dart
- `interop/pad/hpke-base` dart → dart
- `interop/pad/hpke-base/padded` dart → dart
- `interop/rfi/hpke-base/direct` dart → dart
- `interop/rfi/hpke-base/reply-path` dart → dart
- `interop/rfi/hpke-base/referral` dart → dart
- `interop/rfi/sealed-box` dart → dart
- `interop/rfa/hpke-base/echo-own-rfi` dart → dart
- `interop/rfa/hpke-base/answers-receiver-rfi` dart → dart
- `interop/rfd/hpke-base` dart → dart
- `interop/rfi/hpke-base/padded` dart → dart
- `interop/rfi/hpke-pq` dart → dart
- `interop/rfa/sealed-box` dart → dart
- `interop/rfd/sealed-box` dart → dart
- `interop/hop/nested` dart → dart
- `interop/hop/nested/padded` dart → dart
- `interop/hop/nested/signed-only-inner` dart → dart
- `interop/hop/routed-2-hops` dart → dart
- `interop/hop/routed-long-hop-list` dart → dart
- `interop/rfi/hpke-base/long-reply-path` dart → dart
- `interop/hop/routed-12-hops` dart → dart
- `interop/hop/routed-17-hops` dart → dart
- `interop/hop/nested-mixed/inner-by-affinidi-rust` affinidi-rust → dart
- `interop/hop/nested-mixed/inner-by-affinidi-rust` reference-rust → dart
- `interop/hop/nested-mixed/inner-by-affinidi-rust` tsp-js → dart
- `interop/hop/nested-mixed/inner-by-affinidi-rust` dart → affinidi-rust
- `interop/hop/nested-mixed/inner-by-affinidi-rust` dart → reference-rust
- `interop/hop/nested-mixed/inner-by-affinidi-rust` dart → tsp-js
- `interop/hop/nested-mixed/inner-by-affinidi-rust` dart → dart
- `interop/hop/nested-mixed/inner-by-reference-rust` affinidi-rust → dart
- `interop/hop/nested-mixed/inner-by-reference-rust` reference-rust → dart
- `interop/hop/nested-mixed/inner-by-reference-rust` tsp-js → dart
- `interop/hop/nested-mixed/inner-by-reference-rust` dart → affinidi-rust
- `interop/hop/nested-mixed/inner-by-reference-rust` dart → reference-rust
- `interop/hop/nested-mixed/inner-by-reference-rust` dart → tsp-js
- `interop/hop/nested-mixed/inner-by-reference-rust` dart → dart
- `interop/hop/nested-mixed/inner-by-tsp-js` affinidi-rust → dart
- `interop/hop/nested-mixed/inner-by-tsp-js` reference-rust → dart
- `interop/hop/nested-mixed/inner-by-tsp-js` tsp-js → dart
- `interop/hop/nested-mixed/inner-by-tsp-js` dart → affinidi-rust
- `interop/hop/nested-mixed/inner-by-tsp-js` dart → reference-rust
- `interop/hop/nested-mixed/inner-by-tsp-js` dart → tsp-js
- `interop/hop/nested-mixed/inner-by-tsp-js` dart → dart
- `interop/hop/nested-mixed/inner-by-go` affinidi-rust → dart
- `interop/hop/nested-mixed/inner-by-go` reference-rust → dart
- `interop/hop/nested-mixed/inner-by-go` tsp-js → dart
- `interop/hop/nested-mixed/inner-by-go` dart → affinidi-rust
- `interop/hop/nested-mixed/inner-by-go` dart → reference-rust
- `interop/hop/nested-mixed/inner-by-go` dart → tsp-js
- `interop/hop/nested-mixed/inner-by-go` dart → dart
- `interop/hop/nested-mixed/inner-by-dart` affinidi-rust → dart
- `interop/hop/nested-mixed/inner-by-dart` reference-rust → dart
- `interop/hop/nested-mixed/inner-by-dart` tsp-js → dart
- `interop/hop/nested-mixed/inner-by-dart` dart → affinidi-rust
- `interop/hop/nested-mixed/inner-by-dart` dart → reference-rust
- `interop/hop/nested-mixed/inner-by-dart` dart → tsp-js
- `interop/hop/nested-mixed/inner-by-dart` dart → dart
- `negative/self/hpke-base/draft-version-rev3` self/hpke-base → dart
- `relationship/invite-accept-bidirectional` affinidi-rust → dart
- `relationship/cancel-returns-to-none` affinidi-rust → dart
- `relationship/cancel-by-inviter-naming-invite-digest` affinidi-rust → dart
- `relationship/cancel-by-inviter-naming-accept-digest` affinidi-rust → dart
- `relationship/cancel-by-accepter-naming-invite-digest` affinidi-rust → dart
- `relationship/cancel-by-accepter-naming-accept-digest` affinidi-rust → dart
- `relationship/rfi-race-lower-digest-wins` affinidi-rust → dart
- `relationship/invite-race-raw-byte-order` affinidi-rust → dart
- `relationship/invite-accept-bidirectional` reference-rust → dart
- `relationship/cancel-returns-to-none` reference-rust → dart
- `relationship/cancel-by-inviter-naming-invite-digest` reference-rust → dart
- `relationship/cancel-by-inviter-naming-accept-digest` reference-rust → dart
- `relationship/cancel-by-accepter-naming-invite-digest` reference-rust → dart
- `relationship/cancel-by-accepter-naming-accept-digest` reference-rust → dart
- `relationship/rfi-race-lower-digest-wins` reference-rust → dart
- `relationship/invite-race-raw-byte-order` reference-rust → dart
- `relationship/invite-accept-bidirectional` tsp-js → dart
- `relationship/cancel-returns-to-none` tsp-js → dart
- `relationship/cancel-by-inviter-naming-invite-digest` tsp-js → dart
- `relationship/cancel-by-inviter-naming-accept-digest` tsp-js → dart
- `relationship/cancel-by-accepter-naming-invite-digest` tsp-js → dart
- `relationship/cancel-by-accepter-naming-accept-digest` tsp-js → dart
- `relationship/rfi-race-lower-digest-wins` tsp-js → dart
- `relationship/invite-race-raw-byte-order` tsp-js → dart
- `relationship/invite-accept-bidirectional` dart → affinidi-rust
- `relationship/cancel-returns-to-none` dart → affinidi-rust
- `relationship/cancel-by-inviter-naming-invite-digest` dart → affinidi-rust
- `relationship/cancel-by-inviter-naming-accept-digest` dart → affinidi-rust
- `relationship/cancel-by-accepter-naming-invite-digest` dart → affinidi-rust
- `relationship/cancel-by-accepter-naming-accept-digest` dart → affinidi-rust
- `relationship/rfi-race-lower-digest-wins` dart → affinidi-rust
- `relationship/invite-race-raw-byte-order` dart → affinidi-rust
- `relationship/invite-accept-bidirectional` dart → reference-rust
- `relationship/cancel-returns-to-none` dart → reference-rust
- `relationship/cancel-by-inviter-naming-invite-digest` dart → reference-rust
- `relationship/cancel-by-inviter-naming-accept-digest` dart → reference-rust
- `relationship/cancel-by-accepter-naming-invite-digest` dart → reference-rust
- `relationship/cancel-by-accepter-naming-accept-digest` dart → reference-rust
- `relationship/rfi-race-lower-digest-wins` dart → reference-rust
- `relationship/invite-race-raw-byte-order` dart → reference-rust
- `relationship/invite-accept-bidirectional` dart → tsp-js
- `relationship/cancel-returns-to-none` dart → tsp-js
- `relationship/cancel-by-inviter-naming-invite-digest` dart → tsp-js
- `relationship/cancel-by-inviter-naming-accept-digest` dart → tsp-js
- `relationship/cancel-by-accepter-naming-invite-digest` dart → tsp-js
- `relationship/cancel-by-accepter-naming-accept-digest` dart → tsp-js
- `relationship/rfi-race-lower-digest-wins` dart → tsp-js
- `relationship/invite-race-raw-byte-order` dart → tsp-js

</details>

### 4. a nested message whose inner message affinidi-tsp-go packed (Rev 3) is refused

**Kind:** `spec-violation` · **Spec:** §9.1: the current version is YTSP-BAA (1.0); MAJOR gates processability

See `go-rev3`. The outer parties here are 1.0 implementations, so the `implementation` filter of that entry cannot reach these; the inner message is the Go library's, at YTSP-AAC.

Observed in 9 case(s); for example `interop/hop/nested-mixed/inner-by-go` (affinidi-rust → affinidi-rust):

```text
affinidi-rust refused to open go's message: [version] TSP revision mismatch: message is 0.2, this build speaks 1.0 (unsupported TSP major version 0 (this build speaks 1))
```

<details><summary>Affected cases</summary>

- `interop/hop/nested-mixed/inner-by-go` affinidi-rust → affinidi-rust
- `interop/hop/nested-mixed/inner-by-go` affinidi-rust → reference-rust
- `interop/hop/nested-mixed/inner-by-go` affinidi-rust → tsp-js
- `interop/hop/nested-mixed/inner-by-go` reference-rust → affinidi-rust
- `interop/hop/nested-mixed/inner-by-go` reference-rust → reference-rust
- `interop/hop/nested-mixed/inner-by-go` reference-rust → tsp-js
- `interop/hop/nested-mixed/inner-by-go` tsp-js → affinidi-rust
- `interop/hop/nested-mixed/inner-by-go` tsp-js → reference-rust
- `interop/hop/nested-mixed/inner-by-go` tsp-js → tsp-js

</details>

### 5. a nested message whose inner message affinidi-tsp-dart packed (Rev 3) is refused

**Kind:** `spec-violation` · **Spec:** §9.1: the current version is YTSP-BAA (1.0); MAJOR gates processability

See `dart-rev3`. The outer parties here are 1.0 implementations; the inner message is the Dart library's, at YTSP-AAC.

Observed in 9 case(s); for example `interop/hop/nested-mixed/inner-by-dart` (affinidi-rust → affinidi-rust):

```text
affinidi-rust refused to open dart's message: [version] TSP revision mismatch: message is 0.2, this build speaks 1.0 (unsupported TSP major version 0 (this build speaks 1))
```

<details><summary>Affected cases</summary>

- `interop/hop/nested-mixed/inner-by-dart` affinidi-rust → affinidi-rust
- `interop/hop/nested-mixed/inner-by-dart` affinidi-rust → reference-rust
- `interop/hop/nested-mixed/inner-by-dart` affinidi-rust → tsp-js
- `interop/hop/nested-mixed/inner-by-dart` reference-rust → affinidi-rust
- `interop/hop/nested-mixed/inner-by-dart` reference-rust → reference-rust
- `interop/hop/nested-mixed/inner-by-dart` reference-rust → tsp-js
- `interop/hop/nested-mixed/inner-by-dart` tsp-js → affinidi-rust
- `interop/hop/nested-mixed/inner-by-dart` tsp-js → reference-rust
- `interop/hop/nested-mixed/inner-by-dart` tsp-js → tsp-js

</details>

## Coverage notes

**tsp-js: the #77 XSCS-body cases cannot run end to end** (tswg-tsp-specification#77)

The #77 cases are built by editing and re-signing a signed-only message, because the runner implements no HPKE, and tsp-js has no signed-only reader, so they skip for tsp-js. The rule is covered at unit level in the library: `packages/tsp-js/tests/payload.app-stream.mjs` feeds `decodePayloadFrame` a single Bytes primitive (accepted), an `-H##` group, a second primitive, and data after the stream (all refused). No confidential path exists either: no driver can pack those bodies, and the deterministic packers (go, dart) take ephemeral keys but only for bodies they build themselves.

## Warnings

Correct rejections under an error code other than the one expected (the specification mandates no taxonomy).

| Implementation | Case → code | Count |
|---|---|---:|
| affinidi-rust | `flip-envelope-sender → [malformed]` | 1 |
| go | `signature-index-nonzero → [unsupported]` | 1 |
| tsp-js | `flip-version-minor → [malformed]` | 3 |

## Skip reasons

| Reason | Cases |
|---|---:|
| baseline does not open ([version] tsp: version: unsupported TSP major version 1), so corruption proves nothing | 75 |
| baseline does not open ([version] unsupported TSP version … so corruption proves nothing | 75 |
| tsp-js lacks padding:pack | 35 |
| tsp-js lacks signed-only:open | 34 |
| tsp-js lacks sealed-box:open | 33 |
| tsp-js lacks hpke-pq:open | 24 |
| tsp-js lacks sealed-box:pack | 22 |
| the re-signed but untouched vector does not open here | 12 |
| tsp-js lacks signed-only:pack | 11 |
| mutation not applicable to this source | 10 |
| reference-rust pack: unsupported — padding is reachable only for scs/ctl/pad (SecureStore SendOptions) | 10 |
| tsp-js lacks hpke-pq:pack | 10 |
| reference-rust lacks deterministic | 9 |
| tsp-js lacks rfi.referral:pack | 6 |
| reference-rust lacks rfi.referral:pack | 5 |
| the vector publishes no ephemeral material (spec: the hybrid KEM draws encapsulation randomness) | 5 |
| tsp-js lacks payload.ctl:pack | 5 |
| tsp-js lacks payload.pad:pack | 5 |
| tsp-js lacks payload.pad:pack, padding:pack | 5 |
| a re-signed but untouched signed-only rfi does not open here ([unsupported] payload type is not supported by this implementation) | 1 |
| a re-signed but untouched signed-only rfi does not open here ([version] tsp: version: unsupported TSP major version 1) | 1 |
| a re-signed but untouched signed-only rfi does not open here ([version] unsupported TSP version … | 1 |
| the runner-sealed control (sender present) does not open ([version] tsp: version: unsupported TSP major version 1), so the probe proves nothing | 1 |
| the runner-sealed control (sender present) does not open ([version] unsupported TSP version … so the probe proves nothing | 1 |
