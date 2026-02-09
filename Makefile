SHELL := /bin/bash

.PHONY: help check test smoke-live-network deep-live-network demo demo-localhost-transport ci-tools

help:
	@echo "KAMN developer lanes"
	@echo "  make check  - fast static verification (cargo fmt + strict clippy)"
	@echo "  make test   - default bounded test lane (cargo test)"
	@echo "  make smoke-live-network - bounded pilot smoke lane + JSON report"
	@echo "  make deep-live-network - scheduled/manual pilot deep lane summary report"
	@echo "  make demo   - two-process localhost signed-message demo"
	@echo "  make demo-localhost-transport - explicit localhost sender/listener transport demo"
	@echo "  make ci-tools - CI helper regression suite"
	@echo "Deep/scheduled lanes remain opt-in via scripts/sdk/run_rust_live_transport_deep_lane.sh and related scripts."

check:
	cargo fmt --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test

smoke-live-network:
	bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json

deep-live-network:
	bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name workflow_dispatch --output-json /tmp/live-network-pilot-report.json

demo:
	bash scripts/sdk/run_localhost_signed_demo.sh

demo-localhost-transport:
	bash scripts/sdk/run_localhost_signed_demo.sh

ci-tools:
	bash scripts/ci/test_ci_tools.sh
