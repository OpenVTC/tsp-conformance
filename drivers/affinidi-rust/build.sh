#!/bin/sh
# Build the affinidi-tsp driver.
#
# By default this is a plain `cargo build` against the path dependency in
# Cargo.toml (the sibling affinidi-tdk-rs checkout). Set AFFINIDI_TSP_PATH to
# build against another copy of the crate — a worktree or branch — without
# editing Cargo.toml: a manifest with the path substituted is generated under
# target/override/ and built into the same target directory, so the run
# command does not change. Usually set from drivers.local.toml.
#
# If the crate declares its `test-vectors` feature, the generated manifest also
# enables it on the dependency and the build enables the driver's
# `deterministic` feature, which advertises the `deterministic` capability. A
# copy of the crate without the feature builds as before, without it.
set -eu
cd "$(dirname "$0")"

default_path="../../../affinidi-tdk-rs/crates/messaging/affinidi-tsp"
crate="${AFFINIDI_TSP_PATH:-${default_path}}"
deterministic=""
if grep -q '^test-vectors *=' "${crate}/Cargo.toml" 2>/dev/null; then
  deterministic=yes
fi

if [ -z "${AFFINIDI_TSP_PATH:-}" ] && [ -z "${deterministic}" ]; then
  exec cargo +stable build --release
fi

here=$(pwd)
case "${crate}" in
  /*) abs_crate="${crate}" ;;
  *) abs_crate="${here}/${crate}" ;;
esac
features='features = ["pq"]'
if [ -n "${deterministic}" ]; then
  features='features = ["pq", "test-vectors"]'
fi
mkdir -p target/override
sed -e "s#path = \"${default_path}\"#path = \"${abs_crate}\"#" \
    -e "s#features = \[\"pq\"\] }#${features} }#" \
    -e "s#path = \"src/main.rs\"#path = \"${here}/src/main.rs\"#" \
    Cargo.toml > target/override/Cargo.toml
grep -q "${abs_crate}" target/override/Cargo.toml || { echo "path substitution failed" >&2; exit 1; }
echo "building against ${abs_crate}" >&2
if [ -n "${deterministic}" ]; then
  grep -q '"test-vectors"' target/override/Cargo.toml || { echo "feature substitution failed" >&2; exit 1; }
  echo "affinidi-tsp declares test-vectors: building with the deterministic capability" >&2
  exec cargo +stable build --release --features deterministic --manifest-path target/override/Cargo.toml --target-dir target
fi
exec cargo +stable build --release --manifest-path target/override/Cargo.toml --target-dir target
