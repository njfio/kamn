# KAMN

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable coordination layer for autonomous agents. This repository contains the Rust core state machine, node/runtime scaffolding, SDK surfaces, deterministic fixture lanes, and CI policy tooling used to evolve the protocol safely.

## What This Repository Contains

- `crates/kamn-core`: core protocol/domain logic and contract tests.
- `crates/kamn-node`: node/runtime entrypoint scaffolding.
- `crates/kamn-sdk`: Rust SDK surface.
- `scripts/`: deterministic validation lanes and CI helper tooling.
- `fixtures/`: replay/contract fixtures used by fast and deep lanes.
- `docs/foundation/`: implementation contracts mapped to PRD scope.

## Quickstart

### Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- Bash shell
- Node.js/npm (only needed for dashboard/TypeScript lanes)
- Dashboard package lane auto-falls back to `npx -y node@22` when local Node lacks `--experimental-strip-types`

### Dashboard Runtime Compatibility (CI + Local)

```bash
# Runtime compatibility regression contract
bash scripts/frontend/test_dashboard_package_runtime_compat.sh

# Frontend/backend dashboard contract lane
bash scripts/frontend/test_dashboard_contract_lane.sh
```

For hosts where the default `node` binary does not support `--experimental-strip-types`, run:

```bash
KAMN_DASHBOARD_NODE_BIN=node \
KAMN_DASHBOARD_FALLBACK_NODE_CMD="npx -y node@22" \
bash scripts/frontend/test_dashboard_package.sh
```

### Validate Local Environment

```bash
# Format
cargo fmt --check

# Lint (strict)
cargo clippy -- -D warnings

# Core tests
cargo test

# CI tool regression suite (fast/deep routing guards, script contracts)
bash scripts/ci/test_ci_tools.sh

# kamn-core missing-docs policy contract checker
bash scripts/ci/check_kamn_core_missing_docs_policy.sh

# settlement evidence verification (payout/refund/dispute schema policy)
bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS
bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json
```

### Kolme Native Parity Command Matrix (Fast Gate vs Local-Only Heavy)

```bash
# Fast-gate native parity contract lane (PR-safe, bounded)
bash scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh --output-json /tmp/kolme-fast-gate-native-api-parity-summary.json
python3 scripts/kolme/check_fast_gate_native_api_parity_policy.py --report-file /tmp/kolme-fast-gate-native-api-parity-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-fast-gate-native-api-parity-policy.json
# budget marker
KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120

# Local-only heavy native API parity live proof (explicit opt-in)
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_native_api_parity_live_proof_lane.sh --mode run --nonce-command "curl --silent --show-error --fail http://127.0.0.1:3000/get-next-nonce?pubkey=test-key" --broadcast-command "curl --silent --show-error --fail --request PUT --data '{\"message\":\"native-parity\",\"signature\":\"sig\",\"recovery_id\":1}' http://127.0.0.1:3000/broadcast" --finality-command "curl --silent --show-error --fail http://127.0.0.1:3000/block/1" --max-seconds 180 --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json
```

Local-only heavy Kolme run-mode commands stay excluded from ci-fast-gate.

### Fast Make Lanes

```bash
# Fast static gates
make check

# Default bounded test lane
make test

# Two-process localhost signed-message demo
make demo

# Explicit localhost transport sender/listener demo
make demo-localhost-transport

# CI helper regression suite (command/selector/docs contracts)
make ci-tools

# dry-run make target execution contract (no command execution)
bash scripts/ci/test_makefile_execution_contract.sh
```

Deep/scheduled lanes remain opt-in via scripts in `scripts/sdk/` and `scripts/ci/`.
CI fast gate provisions Node.js 22 for frontend and TypeScript contract lanes.

### Run A Focused Core Slice

```bash
cargo test -p kamn-core --test trust_score_engine --test trust_score_engine_docs --test reputation_state_model_docs
bash scripts/ci/test_select_targets.sh
```

### Run A2A/MCP Conformance Contract Lane

```bash
bash scripts/message/run_a2a_mcp_conformance_contract_lane.sh \
  --output-json /tmp/a2a-mcp-conformance-report.json
bash scripts/message/check_a2a_mcp_conformance_policy.sh --report-file /tmp/a2a-mcp-conformance-report.json
# schema: kamn.a2a_mcp.conformance-report.v1
```

### Run A Local End-to-End Demo

```bash
bash scripts/sdk/run_local_e2e_demo.sh
# localhost signed sender/listener transport
bash scripts/sdk/run_localhost_signed_demo.sh

# inspect CLI/session arguments
bash scripts/sdk/run_localhost_signed_demo.sh --help

# explicit deterministic session arguments
bash scripts/sdk/run_localhost_signed_demo.sh \
  --addr 127.0.0.1:17879 \
  --from kamn:did:agent:sender-1 \
  --to kamn:did:agent:listener-1 \
  --nonce 1 \
  --timeout-seconds 15

# emit signed exchange + receipt reconciliation artifact
bash scripts/sdk/run_localhost_signed_demo.sh --output-json /tmp/localhost-signed-demo-artifact.json
# schema: kamn.sdk.localhost-signed.demo-receipt-artifact.v1

# integration harness scenarios
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario success
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario signature-mismatch
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario timeout --timeout-seconds 1
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario replay-nonce
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario admission-guards
```

Kolme local-operability alignment for this localhost demo path is documented in:
`docs/planning/kolme-devnet-ops.md#localhost-two-process-signed-message-demo-contract-issue-1612`.

### Run Localhost Bridge Relay Demo Lane

```bash
bash scripts/bridge/run_localhost_bridge_relay_demo_contract_lane.sh
```

### Run Localhost Signed Integration Contract Lane

```bash
bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh \
  --output-json /tmp/localhost-signed-integration-contract-report.json
bash scripts/sdk/check_localhost_signed_integration_evidence_policy.sh --report-file /tmp/localhost-signed-integration-contract-report.json
# schema: kamn.sdk.localhost-signed.integration-contract.v1
# deterministic keys:
# contract_key=localhost_signed_integration_contract:v1
# success_evidence_key=localhost_signed_integration:success:v1
# signature_mismatch_evidence_key=localhost_signed_integration:signature-mismatch:v1
# malformed_signature_evidence_key=localhost_signed_integration:malformed-signature:v1
# timeout_evidence_key=localhost_signed_integration:timeout:v1
# session_expired_evidence_key=localhost_signed_integration:session-expired:v1
# replay_nonce_evidence_key=localhost_signed_integration:replay-nonce:v1
# admission_guards_evidence_key=localhost_signed_integration:admission-guards:v1
# final_decision=GO
# scenario_fixture_schema_version=kamn.sdk.localhost-signed.integration-fixtures.v1
# scenario_fixture_ids=["success-v1","signature-mismatch-v1","timeout-v1"]
# malformed_signature_reason_code=malformed_signature_detected
# session_expired_reason_code=session_expired_detected
# replay_nonce_reason_code=replay_nonce_detected
# admission_guards_reason_code=session_admission_guards_detected
# expiry_guard_status=pass
# admission_reason_codes=["stale_session_detected","unauthorized_sender_detected","malformed_payload_detected"]
# fixture_file=fixtures/runtime/localhost_signed_integration_cases.json
```

CI fast-gate routes this lane when selector output
`run_localhost_signed_integration_contract_lane_tests` is `true`.

### Run Localhost Signed Demo Contract Lane

```bash
bash scripts/sdk/run_localhost_signed_demo_contract_lane.sh \
  --output-json /tmp/localhost-signed-demo-contract-report.json
# schema: kamn.sdk.localhost-signed.demo-contract.v1
# localhost_signed_demo_status=pass
# localhost_signed_integration_status=pass
# localhost signed demo contract lane tests passed.
```

### Run Unified Local Signed-to-Kolme Demo Contract Lane

```bash
bash scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh \
  --mode dry-run \
  --output-json /tmp/kolme-local-signed-to-kolme-demo-summary.json
python3 scripts/kolme/check_local_signed_to_kolme_demo_policy.py \
  --report-file /tmp/kolme-local-signed-to-kolme-demo-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-signed-to-kolme-demo-policy.json
# schema: kamn.kolme.local-signed-to-kolme-demo-summary.v1
```

### Run Local Fork Profile Preflight Lane

```bash
bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-profile-preflight-summary.json
python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py --report-file /tmp/kolme-local-fork-profile-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-profile-preflight-policy.json
bash scripts/kolme/run_local_kolme_fork_profile_preflight_contract_lane.sh --output-json /tmp/kolme-local-fork-profile-preflight-summary.json --policy-output-json /tmp/kolme-local-fork-profile-preflight-policy.json
# schema: kamn.kolme.local-fork-profile-preflight-summary.v1
# schema: kamn.kolme.local-fork-profile-preflight-policy-report.v1
```

### Run Local Fork Self-Test Lane

```bash
bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-self-test-summary.json
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --matrix-max-seconds 60 --matrix-cargo-profile portable --output-json /tmp/kolme-local-fork-self-test-summary.json
python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py --report-file /tmp/kolme-local-fork-self-test-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-self-test-policy.json
# schema: kamn.kolme.local-fork-self-test-summary.v1
```

### Run Local Fork Checkout Bootstrap Lane

```bash
bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_lane.sh --mode run --checkout-path /tmp/kolme_fork --fork-remote-url https://github.com/njfio/kolme_fork.git --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json
python3 scripts/kolme/check_local_kolme_fork_checkout_bootstrap_policy.py --report-file /tmp/kolme-local-fork-checkout-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json
bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json --policy-output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json
# schema: kamn.kolme.local-fork-checkout-bootstrap-summary.v1
```

### Run Real Fork Local Process Wrapper Contract Lane

```bash
bash scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh \
  --mode dry-run \
  --checkout-path /tmp/kolme_fork \
  --output-json /tmp/kolme-local-fork-real-process-summary.json
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh \
  --mode run \
  --checkout-path /tmp/kolme_fork \
  --fork-remote-url https://github.com/njfio/kolme_fork.git \
  --expected-remote-url https://github.com/njfio/kolme_fork.git \
  --expected-ref refs/heads/main \
  --max-seconds 360 \
  --bootstrap-max-seconds 120 \
  --preflight-max-seconds 45 \
  --self-test-max-seconds 120 \
  --self-test-matrix-max-seconds 60 \
  --lifecycle-max-seconds 300 \
  --lifecycle-startup-max-seconds 45 \
  --lifecycle-integration-max-seconds 240 \
  --lifecycle-bootstrap-max-seconds 90 \
  --lifecycle-conformance-max-seconds 180 \
  --lifecycle-runtime-commit-max-seconds 30 \
  --output-json /tmp/kolme-local-fork-real-process-summary.json
python3 scripts/kolme/check_local_kolme_fork_real_process_policy.py \
  --report-file /tmp/kolme-local-fork-real-process-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-fork-real-process-policy.json
# run mode composes checkout bootstrap + policy prerequisites before preflight/self-test/lifecycle
# schema: kamn.kolme.local-fork-real-process-summary.v1
```

### Run Live Transport Replay/Tamper Contract Lane

```bash
bash scripts/sdk/run_live_transport_replay_tamper_contract_lane.sh \
  --output-report /tmp/live-transport-replay-tamper-contract-report.json
bash scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh \
  --output-report /tmp/live-transport-replay-tamper-fast-report.json
bash scripts/sdk/run_live_transport_replay_tamper_deep_lane.sh \
  --output-report /tmp/live-transport-replay-tamper-deep-report.json
bash scripts/sdk/check_live_transport_replay_tamper_policy.sh \
  --bundle-file /tmp/live-transport-replay-tamper-contract-report.json
# schema: kamn.sdk.live-transport-replay-tamper-evidence.v1
# lane_mode markers: fast/deep
# deep lane marker: deep_no_go_status=verified
```

### Python Live Backend Adapter (Kolme)

```python
from kamn_sdk import LiveKAMNClient, LiveTransportBackendAdapterError

class KolmeBackendAdapter:
    def invoke(self, request):
        # request = {"endpoint": str, "operation": str, "payload": {...}}
        # Map operation/payload to your kolme_fork API surface.
        return {"status": "ok", "value": "kamn:did:agent:backend-1"}

endpoint = "https://live.kamn.testnet/python-backend-adapter"
LiveKAMNClient.register_backend_adapter(endpoint, KolmeBackendAdapter())
client = LiveKAMNClient(endpoint)

try:
    did = client.register("autonomous", "claude-4", ["text"])
except LiveTransportBackendAdapterError as error:
    print(error.operation, error.reason)
finally:
    LiveKAMNClient.clear_backend_adapters()
```

Validation commands:

```bash
python3 -m unittest tests/python/test_sdk.py
bash scripts/sdk/run_live_transport_parity_contract_lane.sh
```

### Troubleshoot Localhost Transport Failures

- Troubleshooting taxonomy and remediation details:
  - `docs/foundation/runtime-network.md#live-transport-demo-failure-taxonomy-and-troubleshooting`
  - `docs/planning/live-network-wave.md#troubleshooting-runbook`
- Primary failure taxonomy markers:
  - `signature_mismatch_detected`
  - `session_expired_detected`
  - `tamper_payload_detected`
- Deterministic repro + policy loop:

```bash
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario malformed-signature
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario session-expired
bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario replay-nonce
bash scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh --output-report /tmp/live-transport-replay-tamper-fast-report.json
bash scripts/sdk/check_live_transport_replay_tamper_policy.sh --bundle-file /tmp/live-transport-replay-tamper-fast-report.json
bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed-integration-contract-report.json
```

### Run Localhost Bridge Demo Evidence Contract Lane (Fast)

```bash
bash scripts/bridge/run_localhost_bridge_demo_evidence_contract_lane.sh
# 120-second budget guard, schema: kamn.bridge.localhost-demo-evidence.v1
```

### Run Localhost Bridge Demo Evidence Deep Lane (Scheduled/Manual)

```bash
bash scripts/bridge/run_localhost_bridge_demo_evidence_deep_lane.sh --output-json /tmp/localhost-bridge-demo-evidence-deep-report.json
# 300-second budget guard, schema: kamn.bridge.localhost-demo-evidence.v1
```

### Run Live-Network Pilot Smoke Lane

```bash
make smoke-live-network
# emits /tmp/live-network-smoke-report.json (schema: kamn.runtime.live-network-smoke-report.v1)
```

### Run Live-Network Pilot Deep Lane (Scheduled/Manual)

```bash
make deep-live-network
# emits /tmp/live-network-pilot-report.json (schema: kamn.runtime.live-network-pilot-artifact-summary.v1)
```

### Run Triadic Devnet Smoke (Kolme)

```bash
bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file /tmp/triadic-devnet-markers.txt
python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file /tmp/triadic-devnet-markers.txt --output-json /tmp/triadic-devnet-report.json
```

### Run Local Fork Sync Metadata Lane (Kolme)

```bash
# deterministic metadata sync plan (no command execution)
bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-sync-metadata-summary.json

# deterministic metadata validation for local fork checkout
bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-sync-metadata-summary.json
# schema: kamn.kolme.local-fork-sync-metadata-summary.v1
```

### Run Local Fork Smoke Evidence Lane (Kolme)

```bash
# deterministic smoke lane plan (no command execution)
bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json

# bounded local-only smoke run against fork checkout
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --smoke-command "cargo test -p merkle-map --test version -- --exact load_from_zero_example" --max-seconds 120 --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json
# schema: kamn.kolme.local-fork-smoke-evidence-summary.v1
```

### Run Local Fork Rust Test Matrix Lane (Kolme)

```bash
# deterministic local matrix plan (no command execution)
bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json

# explicit local-only bounded matrix execution against fork checkout
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --max-seconds 120 --cargo-profile portable --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json
# schema: kamn.kolme.local-fork-rust-test-matrix-summary.v1

# policy checker contract
python3 scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py --report-file /tmp/kolme-local-fork-rust-test-matrix-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json

# bounded contract lane (dry-run + local-only run + fail-closed policy checks)
bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json --policy-output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json
```

### Run Local Kolme API Probe Lane

```bash
# deterministic local API probe plan (no endpoint execution)
bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-api-probe-summary.json

# bounded local API probe execution (healthz + fork-info checks)
bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 30 --output-json /tmp/kolme-local-api-probe-summary.json
# schema: kamn.kolme.local-api-probe-summary.v1
```

### Run Local-Only Kolme API Smoke Lane

```bash
# deterministic local API smoke plan (no command execution)
bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --output-json /tmp/kolme-local-api-smoke-summary.json

# bounded local-only API smoke execution (requires explicit opt-in)
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --max-seconds 60 --output-json /tmp/kolme-local-api-smoke-summary.json
# schema: kamn.kolme.local-api-smoke-summary.v1
```

### Run Local-Only Live Kolme API Conformance Harness

```bash
# deterministic local live conformance plan (no command execution)
bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode dry-run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-live-api-conformance-summary.json

# local-only live conformance execution (probe + native parity checks)
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 180 --probe-max-seconds 30 --native-max-seconds 120 --output-json /tmp/kolme-local-live-api-conformance-summary.json
# schema: kamn.kolme.local-live-api-conformance-summary.v1

# policy checker contract
python3 scripts/kolme/check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-api-conformance-policy.json

# bounded contract lane (spawns local mock API server for deterministic integration validation)
bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json
```

### Run Local Kolme Fork Bootstrap/Readiness Lane

```bash
# deterministic bootstrap/readiness plan (no command execution)
bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json

# explicit local-only bootstrap/readiness execution
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 90 --probe-max-seconds 20 --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json

# policy checker contract
python3 scripts/kolme/check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json

# bounded contract lane (spawns local mock API server + pinned checkout fixture)
bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_contract_lane.sh --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json
# schema: kamn.kolme.local-fork-bootstrap-readiness-summary.v1
```

### Run Local KAMN Live Runtime Integration Lane

```bash
# deterministic runtime integration plan (no command execution)
bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json

# explicit local-only runtime integration execution
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --max-seconds 210 --bootstrap-max-seconds 90 --localhost-signed-max-seconds 45 --conformance-max-seconds 180 --runtime-commit-max-seconds 30 --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json

# policy checker contract
python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json

# bounded contract lane (spawns local mock API server + pinned checkout fixture)
bash scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json
# schema: kamn.kolme.local-kamn-live-runtime-integration-summary.v1
```

### Run Local Kolme Fork Process Lifecycle Integration Lane

```bash
# deterministic process lifecycle plan (no command execution)
bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json

# explicit local-only process lifecycle execution (start -> readiness -> integration -> teardown)
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --serve-command "python3 /tmp/mock_kolme_api.py 3000 v0.15.2" --max-seconds 300 --startup-max-seconds 45 --integration-max-seconds 240 --integration-bootstrap-max-seconds 90 --integration-conformance-max-seconds 180 --integration-runtime-commit-max-seconds 30 --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json

# policy checker contract
python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-fork-process-lifecycle-policy.json

# bounded contract lane (spawns local mock API process command + pinned checkout fixture)
bash scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json
# schema: kamn.kolme.local-fork-process-lifecycle-summary.v1
```

### Run Local-Only Heavy Kolme Validation Matrix

```bash
# deterministic bootstrap health-check plan (no command execution)
bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode dry-run --output-json /tmp/kolme-local-bootstrap-summary.json

# deterministic bootstrap health checks (explicit local-only opt-in)
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json
# schema: kamn.kolme.local-bootstrap-summary.v1

# policy checker contract
python3 scripts/kolme/check_local_bootstrap_health_policy.py --report-file /tmp/kolme-local-bootstrap-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-bootstrap-policy.json

# bounded contract lane (dry-run + policy)
bash scripts/kolme/run_local_bootstrap_health_checks_contract_lane.sh --output-json /tmp/kolme-local-bootstrap-summary.json --policy-output-json /tmp/kolme-local-bootstrap-policy.json
# schema: kamn.kolme.local-bootstrap-policy-report.v1

# local-only heavy end-to-end lane plan (no command execution)
bash scripts/kolme/run_local_e2e_integration_lane.sh --mode dry-run --output-json /tmp/kolme-local-e2e-integration-summary.json

# local-only heavy end-to-end lane execution
# (bootstrap + checkout bootstrap contract + runtime commit + sdk parity + fork rust matrix + live API conformance)
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_e2e_integration_lane.sh --mode run --output-json /tmp/kolme-local-e2e-integration-summary.json
# schema: kamn.kolme.local-e2e-integration-summary.v1

# policy checker contract
python3 scripts/kolme/check_local_e2e_integration_policy.py --report-file /tmp/kolme-local-e2e-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-e2e-integration-policy.json

# bounded contract lane (dry-run + policy)
bash scripts/kolme/run_local_e2e_integration_contract_lane.sh --output-json /tmp/kolme-local-e2e-integration-summary.json --policy-output-json /tmp/kolme-local-e2e-integration-policy.json
# schema: kamn.kolme.local-e2e-integration-policy-report.v1

# command surface + artifact schema validation (no heavy execution)
bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode dry-run --output-json /tmp/kolme-local-heavy-validation-summary.json

# explicit local-only heavy execution
# (bootstrap preflight + deep replay + fork rust matrix contract + live API conformance contract)
KAMN_KOLME_LOCAL_HEAVY=1 \
bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode run --output-json /tmp/kolme-local-heavy-validation-summary.json
# schema: kamn.kolme.local-heavy-validation-summary.v1

# policy checker contract
python3 scripts/kolme/check_local_heavy_validation_matrix_policy.py --report-file /tmp/kolme-local-heavy-validation-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-heavy-validation-policy.json

# bounded contract lane (dry-run + policy)
bash scripts/kolme/run_local_heavy_validation_matrix_contract_lane.sh --output-json /tmp/kolme-local-heavy-validation-summary.json --policy-output-json /tmp/kolme-local-heavy-validation-policy.json
# schema: kamn.kolme.local-heavy-validation-policy-report.v1
```

Local-only heavy Kolme run-mode commands stay excluded from ci-fast-gate.
Fast-gate validates baseline command surfaces; aggregate `make ci-tools` also validates policy-contract surfaces:
- `bash scripts/kolme/test_run_local_bootstrap_health_checks.sh`
- `bash scripts/kolme/test_check_local_bootstrap_health_policy.sh`
- `bash scripts/kolme/test_run_local_bootstrap_health_checks_contract_lane.sh`
- `bash scripts/kolme/test_run_local_e2e_integration_lane.sh`
- `bash scripts/kolme/test_check_local_e2e_integration_policy.sh`
- `bash scripts/kolme/test_run_local_e2e_integration_contract_lane.sh`
- `bash scripts/kolme/test_check_local_heavy_validation_matrix_policy.sh`
- `bash scripts/kolme/test_run_local_heavy_validation_matrix_contract_lane.sh`

Local Kolme API probe/smoke lane contract tests:
- `bash scripts/kolme/test_run_local_kolme_api_probe_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_api_smoke_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh`
- `bash scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_live_api_conformance_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_process_lifecycle_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_profile_preflight_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_profile_preflight_contract_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_self_test_lane.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_lane.sh`
- `bash scripts/kolme/test_check_local_kolme_fork_checkout_bootstrap_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_contract_lane.sh`
- `bash scripts/kolme/test_check_local_kolme_fork_real_process_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_fork_real_process_contract_lane.sh`

## Workflow

All code changes are issue-first and follow strict Red → Green → Refactor → Regression TDD. Before implementation:

1. Create or select a GitHub task issue with required labels.
2. Move the issue to `status:in-progress`.
3. Create a branch: `codex/issue-<id>-<short-slug>`.
4. Log progress comments on the issue using the required status template.

Canonical contributor rules are in `.github/CONTRIBUTING.md` (`AGENTS.md` remains a compatibility redirect).

## Key Links

- `.github/CONTRIBUTING.md`: mandatory execution contract (issue hierarchy, TDD, PR standards).
- `AGENTS.md`: compatibility redirect for agent tooling.
- `PRD.md`: product requirements and phase scope baseline.
- `docs/planning/engineering-hardening-wave.md#commands`: baseline hardening and missing-doc policy command surface.
- `docs/architecture/kamn-core-module-map.md#ownership-matrix`: `kamn-core` domain ownership map.
- `docs/architecture/kamn-core-module-map.md#contributor-entrypoint-matrix`: contributor entrypoints by architecture/workflow need.
- `docs/developer/rustdoc-publishing.md#contract-enforcement`: bounded rustdoc generation and publication policy checks.
- `docs/planning/live-network-wave.md`: pilot smoke/deep lane commands, budgets, and evidence contracts.
- `docs/testing/invariant-and-fuzz-strategy.md`: bounded lifecycle property/fuzz/concurrency strategy and command contracts.
- `docs/foundation/`: domain contracts used by docs tests and release gates.
- `.github/workflows/`: CI lane orchestration.
