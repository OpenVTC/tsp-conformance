# TSP Rev 3 conformance suite.
#
#   make build                 build the runner and every driver in drivers.toml
#   make run                   run every suite over every enabled driver
#   make run DRIVERS=affinidi-rust,tsp-js SUITES=interop CASE=rfi SEED=1
#   make report                regenerate reports/report.md from reports/report.json
#   make test                  the runner's own unit tests

RUNNER   := runner/target/release/tsp-conformance
CARGO    ?= cargo +stable
DRIVERS  ?=
SUITES   ?=
CASE     ?=
SEED     ?=
TIMEOUT  ?= 60

ARGS := $(if $(DRIVERS),--drivers $(DRIVERS)) \
        $(if $(SUITES),--suites $(SUITES)) \
        $(if $(CASE),--case $(CASE)) \
        $(if $(SEED),--seed $(SEED)) \
        --timeout $(TIMEOUT)

.PHONY: build runner run report test clean

runner:
	$(CARGO) build --release --manifest-path runner/Cargo.toml

# Builds the runner, then runs every selected driver's `build` command. A
# driver whose directory is missing or whose build fails is skipped (and says so).
build: runner
	$(RUNNER) --build-only $(if $(DRIVERS),--drivers $(DRIVERS))

run: runner
	$(RUNNER) $(ARGS)

report: runner
	$(RUNNER) --from-json reports/report.json --markdown reports/report.md

test:
	$(CARGO) test --manifest-path runner/Cargo.toml

clean:
	rm -rf reports/logs
