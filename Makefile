SHELL := /bin/bash
LOCAL_GATE_BASH_CANDIDATES := bash /opt/homebrew/bin/bash /usr/local/bin/bash /bin/bash
LOCAL_GATE_BASH ?= $(shell for shell_path in $(LOCAL_GATE_BASH_CANDIDATES); do if command -v "$$shell_path" >/dev/null 2>&1 && "$$shell_path" -c 'test "$${BASH_VERSINFO[0]}" -ge 5' >/dev/null 2>&1; then command -v "$$shell_path"; break; fi; done)
LOCAL_GATE_BASH_DIR := $(dir $(LOCAL_GATE_BASH))
PRE_PUSH_PYTHON3_CANDIDATES := python3 .venv/bin/python3 /opt/homebrew/opt/python@3.12/libexec/bin/python3 /usr/bin/python3 /Applications/Xcode.app/Contents/Developer/usr/bin/python3
PRE_PUSH_PYTHON3 ?= $(shell for py in $(PRE_PUSH_PYTHON3_CANDIDATES); do if command -v "$$py" >/dev/null 2>&1 && "$$py" -c "import cryptography, tomllib" >/dev/null 2>&1; then command -v "$$py"; break; fi; done)
PRE_PUSH_PYTHON3_DIR := $(dir $(PRE_PUSH_PYTHON3))
PRE_PUSH_WORKSPACE_TARGET_DIR ?= target/local-pre-push-workspace
PRE_PUSH_WORKSPACE_TIMEOUT_SECONDS ?= 14400
LOCAL_GATE_ENV = PATH="$(PRE_PUSH_PYTHON3_DIR):$(LOCAL_GATE_BASH_DIR):$(PATH)"
PRE_PUSH_ENV = PATH="$(PRE_PUSH_PYTHON3_DIR):$(LOCAL_GATE_BASH_DIR):$(PATH)"

.PHONY: help check test pre-push smoke-live-network deep-live-network demo demo-mvp demo-agent-transaction demo-localhost-transport ci-tools kolme-local-heavy kolme-fork-rust-tests-local

help:
	@echo "KAMN developer lanes"
	@echo "  make check  - fast static verification (cargo fmt + strict clippy)"
	@echo "  make test   - default bounded test lane (cargo test)"
	@echo "  make pre-push - full local gate sequence before publishing"
	@echo "  make smoke-live-network - bounded pilot smoke lane + JSON report"
	@echo "  make deep-live-network - scheduled/manual pilot deep lane summary report"
	@echo "  make demo   - two-process localhost signed-message demo"
	@echo "  make demo-mvp - local-only compatibility proof"
	@echo "  make demo-agent-transaction - canonical Pi/devnet transaction demo"
	@echo "  make demo-localhost-transport - explicit localhost sender/listener transport demo"
	@echo "  make ci-tools - CI helper regression suite"
	@echo "  make kolme-local-heavy - local-only Kolme heavy validation matrix (requires explicit opt-in for run mode)"
	@echo "  make kolme-fork-rust-tests-local - local-only bounded kolme_fork Rust test matrix plan"
	@echo "Deep/scheduled lanes remain opt-in via scripts/sdk/run_rust_live_transport_deep_lane.sh and related scripts."

check:
	cargo fmt --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test

pre-push:
	@if [ -z "$(LOCAL_GATE_BASH)" ]; then echo "make pre-push requires Bash 5+ for local shell contract lanes" >&2; exit 2; fi
	@if [ -z "$(PRE_PUSH_PYTHON3)" ]; then echo "make pre-push requires a python3 interpreter with cryptography and tomllib installed" >&2; exit 2; fi
	@"$(PRE_PUSH_PYTHON3)" -c "import cryptography, tomllib"
	$(PRE_PUSH_ENV) $(MAKE) check
	$(PRE_PUSH_ENV) $(MAKE) ci-tools
	$(PRE_PUSH_ENV) bash scripts/ci/check_touched_rust_size_policy.sh \
		--base-ref main \
		--threshold-file fixtures/ci/touched_rust_size_policy_thresholds.json \
		--baseline-file fixtures/ci/touched_rust_size_policy_baseline.json \
		--output-json /tmp/kamn-touched-rust-size-policy-pre-push.json
	$(PRE_PUSH_ENV) bash scripts/ci/run_with_retry.sh \
		--label local-pre-push-workspace-tests \
		--max-attempts 2 \
		-- bash -lc 'CARGO_TARGET_DIR="$(PRE_PUSH_WORKSPACE_TARGET_DIR)" timeout "$(PRE_PUSH_WORKSPACE_TIMEOUT_SECONDS)" bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --workspace --locked --all-features --no-fail-fast'
	$(PRE_PUSH_ENV) bash scripts/ci/run_critical_path_coverage_gate.sh \
		--threshold-file .ci/critical-path-coverage-thresholds.json \
		--core-json /tmp/kamn-critical-path-core-coverage-pre-push.json \
		--node-json /tmp/kamn-critical-path-node-coverage-pre-push.json \
		--output-json /tmp/kamn-critical-path-coverage-policy-pre-push.json
	$(PRE_PUSH_ENV) bash scripts/ci/run_critical_path_mutation_gate.sh \
		--output-json /tmp/kamn-critical-path-mutation-report-pre-push.json \
		--timeout-seconds 900

smoke-live-network:
	bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json

deep-live-network:
	bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name workflow_dispatch --output-json /tmp/live-network-pilot-report.json

demo:
	bash scripts/sdk/run_localhost_signed_demo.sh

demo-mvp:
	CARGO_TARGET_DIR=target/mvp-demo-proof cargo run -p kamn-e2e-harness -- demo-mvp

demo-agent-transaction:
	CARGO_TARGET_DIR=target/mvp-demo-proof cargo build -p kamn-node -p kamn-mcp-server
	KAMN_MVP_LOCAL_NODE_BINARY="$${KAMN_MVP_LOCAL_NODE_BINARY:-target/mvp-demo-proof/debug/kamn-node}" \
	KAMN_MVP_LIVE_MCP_BINARY="$${KAMN_MVP_LIVE_MCP_BINARY:-target/mvp-demo-proof/debug/kamn-mcp-server}" \
	CARGO_TARGET_DIR=target/mvp-demo-proof cargo run -p kamn-e2e-harness -- demo-agent-transaction

demo-localhost-transport:
	bash scripts/sdk/run_localhost_signed_demo.sh

ci-tools:
	@if [ -z "$(LOCAL_GATE_BASH)" ]; then echo "make ci-tools requires Bash 5+ for local shell contract lanes" >&2; exit 2; fi
	@if [ -z "$(PRE_PUSH_PYTHON3)" ]; then echo "make ci-tools requires a python3 interpreter with cryptography and tomllib installed" >&2; exit 2; fi
	$(LOCAL_GATE_ENV) "$(LOCAL_GATE_BASH)" scripts/ci/test_ci_tools.sh

kolme-local-heavy:
	bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode dry-run --output-json /tmp/kolme-local-heavy-validation-summary.json

kolme-fork-rust-tests-local:
	bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json
