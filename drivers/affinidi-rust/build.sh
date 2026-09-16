#!/bin/sh
# Build the affinidi-tsp driver.
#
# By default this is a plain `cargo build` against the path dependency in
# Cargo.toml (the sibling affinidi-tdk-rs checkout). Set AFFINIDI_TSP_PATH to
# build against another copy of the crate — a worktree or branch — without
# editing Cargo.toml: a manifest with the path substituted is generated under
# target/override/ and built into the same target directory, so the run
# command does not change. Usually set from drivers.local.toml.
set -eu
cd "$(dirname "$0")"

if [ -z "${AFFINIDI_TSP_PATH:-}" ]; then
  exec cargo +stable build --release
fi

here=$(pwd)
mkdir -p target/override
sed -e "s#path = \"../../../affinidi-tdk-rs/crates/messaging/affinidi-tsp\"#path = \"${AFFINIDI_TSP_PATH}\"#" \
    -e "s#path = \"src/main.rs\"#path = \"${here}/src/main.rs\"#" \
    Cargo.toml > target/override/Cargo.toml
grep -q "${AFFINIDI_TSP_PATH}" target/override/Cargo.toml || { echo "path substitution failed" >&2; exit 1; }
echo "building against ${AFFINIDI_TSP_PATH}" >&2
exec cargo +stable build --release --manifest-path target/override/Cargo.toml --target-dir target
