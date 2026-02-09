SHELL := /bin/bash

.PHONY: help check test demo

help:
	@echo "KAMN developer lanes"
	@echo "  make check  - fast static verification (cargo fmt + strict clippy)"
	@echo "  make test   - default bounded test lane (cargo test)"
	@echo "  make demo   - two-process localhost signed-message demo"
	@echo "Deep/scheduled lanes remain opt-in via scripts/sdk/run_rust_live_transport_deep_lane.sh and related scripts."

check:
	cargo fmt --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test

demo:
	bash scripts/sdk/run_localhost_signed_demo.sh
