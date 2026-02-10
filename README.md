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
# timeout_evidence_key=localhost_signed_integration:timeout:v1
# replay_nonce_evidence_key=localhost_signed_integration:replay-nonce:v1
# admission_guards_evidence_key=localhost_signed_integration:admission-guards:v1
# replay_nonce_reason_code=replay_nonce_detected
# admission_guards_reason_code=session_admission_guards_detected
# admission_reason_codes=["stale_session_detected","unauthorized_sender_detected","malformed_payload_detected"]
```

CI fast-gate routes this lane when selector output
`run_localhost_signed_integration_contract_lane_tests` is `true`.

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
