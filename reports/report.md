# TSP Rev 3 conformance report

Target **trustoverip/tswg-tsp-specification** at commit `f5b8668` (`YTSP-AAC`). Generated 2026-09-16T12:32:19Z with key seed `20260916`.

Each case is **pass**, **fail** (with the diff), **skip** (a capability the implementation does not offer) or **error** (the harness or a driver broke). Cells read `✓ passed/ran` when nothing failed, `✗ passed/ran` when something did, `–` when every case skipped.

## Implementations

| Driver | Library | Version | Language | Status |
|---|---|---|---|---|
| affinidi-rust | affinidi-tsp | 0.2.0 | rust | ran |
| reference-rust | tsp_sdk | 0.11.0 | rust | ran |
| tsp-js | @openvtc/vti-tsp-js | 0.3.0 | typescript | ran |
| go | affinidi-tsp-go | 0.1.0 | go | ran |
| dart | affinidi-tsp-dart | 0.1.0 | dart | ran |

Local overrides in effect (`drivers.local.toml`, not committed):

- **affinidi-rust**: `AFFINIDI_TSP_PATH=/Users/glenngore/devel/affinidi-tdk-rs-wt-tsprel/crates/messaging/affinidi-tsp`
- **tsp-js**: `TSP_JS_DIR=/Users/glenngore/devel/pnm-browser-plugin-wt-tsprel/packages/tsp-js`

## Summary

| Suite | Pass | Fail | Error | Skip |
|---|---:|---:|---:|---:|
| interop | 868 | 25 | 0 | 157 |
| negative | 377 | 8 | 0 | 70 |
| relationship | 250 | 0 | 0 | 0 |
| vectors | 137 | 0 | 0 | 38 |

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
| `deterministic` | — | — | — | ✓ | ✓ |
| `peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `endpoint` | ✓ | ✓ | ✓ | ✓ | ✓ |
| *not reported by `open`* | `payload.padding`, `payload.payloadSender` | `payload.nonce`, `payload.padding`, `payload.payloadSender`, `peek.version`, `version` | `payload.padding`, `payload.payloadSender` | — | — |

## Vectors matrix

| Case | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|:-:|:-:|:-:|:-:|:-:|
| `control-rfa-direct/open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `control-rfa-direct/pack-exact` | – | – | – | ✓ | ✓ |
| `control-rfa-direct/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `control-rfd/open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `control-rfd/pack-exact` | – | – | – | ✓ | ✓ |
| `control-rfd/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `control-rfi-direct/open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `control-rfi-direct/pack-exact` | – | – | – | ✓ | ✓ |
| `control-rfi-direct/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `control-rfi-sealed-box/open` | ✓ | ✓ | – | ✓ | ✓ |
| `control-rfi-sealed-box/pack-exact` | – | – | – | ✓ | ✓ |
| `control-rfi-sealed-box/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `direct-hpke-base/open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `direct-hpke-base/pack-exact` | – | – | – | ✓ | ✓ |
| `direct-hpke-base/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `direct-hpke-base-pq/open` | ✓ | ✓ | – | ✓ | ✓ |
| `direct-hpke-base-pq/pack-exact` | – | – | – | – | – |
| `direct-hpke-base-pq/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `direct-sealed-box/open` | ✓ | ✓ | – | ✓ | ✓ |
| `direct-sealed-box/pack-exact` | – | – | – | ✓ | ✓ |
| `direct-sealed-box/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `direct-signed-only/open` | ✓ | ✓ | – | ✓ | ✓ |
| `direct-signed-only/pack-exact` | – | ✓ | – | ✓ | ✓ |
| `direct-signed-only/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `nested-direct/open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `nested-direct/inner-open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `nested-direct/pack-exact` | – | – | – | ✓ | ✓ |
| `nested-direct/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `routed/open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `routed/inner-open` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `routed/pack-exact` | – | – | – | ✓ | ✓ |
| `routed/peek` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `direct-signed-only/resigned/minor-ABA-accepted` | ✓ | ✓ | – | ✓ | ✓ |
| `direct-signed-only/resigned/minor-AAD-accepted` | ✓ | ✓ | – | ✓ | ✓ |
| `direct-signed-only/resigned/essr-sender-present-accepted` | ✓ | ✓ | – | ✓ | ✓ |

## Interop matrix

| packs ↓ / opens → | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|---|---|---|---|---|
| **affinidi-rust** | ✗ 40/41 | ✗ 40/41 | ✗ 31/33 | ✗ 40/41 | ✗ 40/41 |
| **reference-rust** | ✗ 37/38 | ✓ 38/38 | ✗ 28/30 | ✓ 38/38 | ✓ 38/38 |
| **tsp-js** | ✗ 20/22 | ✗ 20/22 | ✗ 20/22 | ✗ 20/22 | ✗ 20/22 |
| **go** | ✗ 41/42 | ✓ 42/42 | ✗ 32/34 | ✓ 42/42 | ✓ 42/42 |
| **dart** | ✗ 41/42 | ✓ 42/42 | ✗ 32/34 | ✓ 42/42 | ✓ 42/42 |

<details><summary>Per case (42 cases × 25 pairs)</summary>

| Case | affin→affin | affin→refer | affin→tsp | affin→go | affin→dart | refer→affin | refer→refer | refer→tsp | refer→go | refer→dart | tsp→affin | tsp→refer | tsp→tsp | tsp→go | tsp→dart | go→affin | go→refer | go→tsp | go→go | go→dart | dart→affin | dart→refer | dart→tsp | dart→go | dart→dart |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `scs/hpke-base/small` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/empty` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/2mib` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-0` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-1` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-2` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-3` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-12285` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/padding-12286` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/signed-only` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `scs/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `scs/hpke-base/payload-sender-null` | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/payload-sender-present` | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-base/long-vid` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `scs/hpke-pq` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `ctl/hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `pad/hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `pad/hpke-base/padded` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/direct` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/reply-path` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/referral` | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `rfa/hpke-base/echo-own-rfi` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfa/hpke-base/answers-receiver-rfi` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfd/hpke-base` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/padded` | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-pq` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `rfa/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `rfd/sealed-box` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `hop/nested` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested/padded` | ✓ | ✓ | ✓ | ✓ | ✓ | – | – | – | – | – | – | – | – | – | – | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested/signed-only-inner` | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ | – | – | – | – | – | ✓ | ✓ | – | ✓ | ✓ | ✓ | ✓ | – | ✓ | ✓ |
| `hop/routed-2-hops` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/routed-long-hop-list` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi/hpke-base/long-reply-path` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/routed-12-hops` | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ |
| `hop/routed-17-hops` | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-affinidi-rust` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-reference-rust` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-tsp-js` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-go` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `hop/nested-mixed/inner-by-dart` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

</details>

## Negative matrix

Aggregated over the message sources (the implementation's own output and the Appendix A vectors); see the JSON report for each source.

| Case | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|:-:|:-:|:-:|:-:|:-:|
| `flip-ciphertext` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 | ✓ 5/5 | ✓ 5/5 |
| `flip-signature` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `flip-envelope-sender` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `flip-version-minor` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `unknown-major-version` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `truncate-one-triplet` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `truncate-half` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `trailing-bytes` | ✓ 6/6 | ✗ 0/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `frame-count-too-large` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `frame-count-too-small` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `signature-index-nonzero` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 | ✓ 5/5 | ✓ 5/5 |
| `attachment-count-too-large` | ✓ 6/6 | ✓ 6/6 | ✓ 3/3 | ✓ 6/6 | ✓ 6/6 |
| `wrong-receiver-key` | ✓ 4/4 | ✓ 4/4 | ✓ 3/3 | ✓ 4/4 | ✓ 4/4 |
| `wrong-sender-key` | ✓ 5/5 | ✓ 5/5 | ✓ 3/3 | ✓ 5/5 | ✓ 5/5 |
| `digest-tampered` | ✓ 1/1 | – | – | ✓ 1/1 | ✓ 1/1 |
| `non-canonical-lead-byte` | ✓ 1/1 | ✗ 0/1 | – | ✓ 1/1 | ✓ 1/1 |
| `essr-sender-mismatch` | ✓ 1/1 | ✗ 0/1 | – | ✓ 1/1 | ✓ 1/1 |
| `payload-count-too-large` | ✓ 1/1 | ✓ 1/1 | – | ✓ 1/1 | ✓ 1/1 |
| `xscs-body-h-group-json` | ✓ 1/1 | ✓ 1/1 | – | ✓ 1/1 | ✓ 1/1 |
| `xscs-body-two-bytes-primitives` | ✓ 1/1 | ✓ 1/1 | – | ✓ 1/1 | ✓ 1/1 |
| `xscs-body-data-after-stream` | ✓ 1/1 | ✓ 1/1 | – | ✓ 1/1 | ✓ 1/1 |

## Relationship matrix

| A ↓ / B → | affinidi-rust | reference-rust | tsp-js | go | dart |
|---|---|---|---|---|---|
| **affinidi-rust** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 |
| **reference-rust** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 |
| **tsp-js** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 |
| **go** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 |
| **dart** | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 | ✓ 10/10 |

<details><summary>Per case (10 cases × 25 pairs)</summary>

| Case | affin→affin | affin→refer | affin→tsp | affin→go | affin→dart | refer→affin | refer→refer | refer→tsp | refer→go | refer→dart | tsp→affin | tsp→refer | tsp→tsp | tsp→go | tsp→dart | go→affin | go→refer | go→tsp | go→go | go→dart | dart→affin | dart→refer | dart→tsp | dart→go | dart→dart |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| `invite-accept-bidirectional` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `message-before-relationship-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-returns-to-none` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-inviter-naming-invite-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-inviter-naming-accept-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-accepter-naming-invite-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `cancel-by-accepter-naming-accept-digest` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `rfi-race-lower-digest-wins` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `invite-race-raw-byte-order` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `accept-unknown-digest-refused` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

</details>

## Findings

Failures attributed to an investigated root cause (`findings.toml`). *Kind*: `spec-violation` contradicts a MUST; `disagreement` — implementations differ where the specification is silent or ambiguous; `limit` — a bound the specification does not set; `api-gap` — the library cannot express what the protocol needs.

| # | Finding | Kind | Cases |
|---|---|---|---:|
| 1 | tsp_sdk accepts a primitive whose lead bytes are non-zero | `spec-violation` | 1 |
| 2 | tsp_sdk ignores bytes after the signature attachment; affinidi-tsp and tsp-js reject them | `disagreement` | 6 |
| 3 | tsp_sdk does not check the ESSR sender field of a signed-only message | `disagreement` | 1 |
| 4 | Route length limits differ: tsp-js 10 hops, affinidi-tsp 16, tsp_sdk none | `limit` | 25 |

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

### 4. Route length limits differ: tsp-js 10 hops, affinidi-tsp 16, tsp_sdk none

**Kind:** `limit` · **Spec:** §5.3: "the number of intermediaries in the route path may not be limited to 2"; no maximum is set

tsp-js `cesr/wire.ts` `MAX_HOPS = 10`, enforced when packing (`packRouted`) and when decoding any VID list (`rev3/fields.ts` `decodeVidList`, which also reads Reply_Path). affinidi-tsp `message/routed.rs` `MAX_HOPS = 16`, enforced when packing and in `wire.rs` `decode_hops`. tsp_sdk sets none. So a 12-hop route packed by affinidi-tsp or tsp_sdk cannot be opened by tsp-js, and a 17-hop route packed by tsp_sdk cannot be opened by affinidi-tsp. A bound is reasonable; three different ones are not interoperable. The specification could state a minimum every receiver must accept.

Observed in 25 case(s); for example `interop/hop/routed-17-hops` (affinidi-rust → affinidi-rust):

```text
affinidi-rust refused to pack: [malformed] invalid message: route has 17 hops, exceeds maximum of 16
```

<details><summary>Affected cases</summary>

- `interop/hop/routed-17-hops` affinidi-rust → affinidi-rust
- `interop/hop/routed-17-hops` affinidi-rust → reference-rust
- `interop/hop/routed-12-hops` affinidi-rust → tsp-js
- `interop/hop/routed-17-hops` affinidi-rust → tsp-js
- `interop/hop/routed-17-hops` affinidi-rust → go
- `interop/hop/routed-17-hops` affinidi-rust → dart
- `interop/hop/routed-17-hops` reference-rust → affinidi-rust
- `interop/hop/routed-12-hops` reference-rust → tsp-js
- `interop/hop/routed-17-hops` reference-rust → tsp-js
- `interop/hop/routed-12-hops` tsp-js → affinidi-rust
- `interop/hop/routed-17-hops` tsp-js → affinidi-rust
- `interop/hop/routed-12-hops` tsp-js → reference-rust
- `interop/hop/routed-17-hops` tsp-js → reference-rust
- `interop/hop/routed-12-hops` tsp-js → tsp-js
- `interop/hop/routed-17-hops` tsp-js → tsp-js
- `interop/hop/routed-12-hops` tsp-js → go
- `interop/hop/routed-17-hops` tsp-js → go
- `interop/hop/routed-12-hops` tsp-js → dart
- `interop/hop/routed-17-hops` tsp-js → dart
- `interop/hop/routed-17-hops` go → affinidi-rust
- `interop/hop/routed-12-hops` go → tsp-js
- `interop/hop/routed-17-hops` go → tsp-js
- `interop/hop/routed-17-hops` dart → affinidi-rust
- `interop/hop/routed-12-hops` dart → tsp-js
- `interop/hop/routed-17-hops` dart → tsp-js

</details>

## Coverage notes

**tsp-js: the #77 XSCS-body cases cannot run end to end** (tswg-tsp-specification#77)

The #77 cases are built by editing and re-signing a signed-only message, because the runner implements no HPKE, and tsp-js has no signed-only reader, so they skip for tsp-js. The rule is covered at unit level in the library: `packages/tsp-js/tests/payload.app-stream.mjs` feeds `decodePayloadFrame` a single Bytes primitive (accepted), an `-H##` group, a second primitive, and data after the stream (all refused). No confidential path exists either: no driver can pack those bodies, and the deterministic packers (go, dart) take ephemeral keys but only for bodies they build themselves.

## Warnings

Correct rejections under an error code other than the one expected (the specification mandates no taxonomy).

| Implementation | Case → code | Count |
|---|---|---:|
| affinidi-rust | `flip-envelope-sender → [malformed]` | 1 |
| dart | `flip-envelope-sender → [malformed]` | 1 |
| go | `flip-envelope-sender → [malformed]` | 1 |
| go | `signature-index-nonzero → [unsupported]` | 5 |

## Skip reasons

| Reason | Cases |
|---|---:|
| tsp-js lacks padding:pack | 35 |
| tsp-js lacks signed-only:open | 33 |
| tsp-js lacks sealed-box:open | 32 |
| tsp-js lacks hpke-pq:open | 23 |
| mutation not applicable to this source | 20 |
| tsp-js lacks sealed-box:pack | 20 |
| tsp-js lacks signed-only:pack | 11 |
| reference-rust pack: unsupported — padding is reachable only for scs/ctl/pad (SecureStore SendOptions) | 10 |
| tsp-js lacks hpke-pq:pack | 10 |
| affinidi-rust lacks deterministic | 8 |
| reference-rust lacks deterministic | 8 |
| tsp-js lacks deterministic | 8 |
| affinidi-rust pack: unsupported — affinidi-tsp always carries the sender VID in the payload; NULL cannot be requested | 6 |
| reference-rust lacks rfi.referral:pack | 5 |
| reference-rust pack: unsupported — tsp_sdk chooses the ESSR sender field by scheme (NULL under HPKE-Base and signed-only) | 5 |
| the vector publishes no ephemeral material (spec: the hybrid KEM draws encapsulation randomness) | 5 |
| tsp-js lacks payload.ctl:pack | 5 |
| tsp-js lacks payload.pad:pack | 5 |
| tsp-js lacks payload.pad:pack, padding:pack | 5 |
| tsp-js lacks rfi.referral:pack | 5 |
| tsp-js pack: unsupported — tsp-js always writes the sender VID in the ESSR field | 5 |
| a re-signed but untouched signed-only rfi does not open here ([unsupported] payload type is not supported by this implementation) | 1 |
