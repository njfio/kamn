# Token Launch Configuration and Handoff Contracts (Issue #714)

This document defines deterministic token launch handoff evidence required before enabling production token operations.

## Scope Delivered
- Token launch handoff evidence generator and policy checker:
  - `scripts/token/generate_token_launch_handoff_evidence_bundle.sh`
  - `scripts/token/check_token_launch_handoff_policy.sh`
- Shared Python implementation backing the token wrappers:
  - `scripts/token/token_launch_handoff_contract.py`
- Shared Python implementation (contract lane):
  - `scripts/token/token_launch_handoff_contract_lane_contract.py`
- Fast/deep lane entrypoints:
  - `scripts/token/run_token_launch_handoff_contract_lane.sh`
  - `scripts/token/run_token_launch_handoff_deep_lane.sh`
- Deterministic fixture cases:
  - `fixtures/token_launch/handoff_invariant_cases.json`

## Token Launch Handoff Evidence Contract
- Evidence bundle generator:
  - `bash scripts/token/generate_token_launch_handoff_evidence_bundle.sh --output-file /tmp/token-launch-handoff.json --token-symbol KAMN --configured-total-supply 1000000000 --expected-total-supply 1000000000 --configured-allocation-sum 1000000000 --expected-allocation-sum 1000000000 --allocation-bucket-count 5 --expected-bucket-count 5 --genesis-hash sha256:token-launch-handoff-go-2026-02-09 --required-approvals 2 --received-approvals 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/token/check_token_launch_handoff_policy.sh --bundle-file /tmp/token-launch-handoff.json`
- PR fast contract lane:
  - `bash scripts/token/run_token_launch_handoff_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/token/run_token_launch_handoff_deep_lane.sh --output-json token-launch-handoff-report.json`
- Regression policy:
  - supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`).
  - shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1270`).

## CI Cost Strategy
- Fast lane runs only for token-launch-relevant changes (token core, token docs/contracts, token scripts/fixtures).
- Deep lane remains scheduled/manual and emits a machine-readable report artifact.

## Local Validation
Run from repository root:

```bash
bash scripts/framework/test_contract_framework.sh
bash scripts/token/test_generate_token_launch_handoff_evidence_bundle.sh
bash scripts/token/test_run_token_launch_handoff_contract_lane.sh
bash scripts/token/test_run_token_launch_handoff_deep_lane.sh
cargo test -p kamn-core --test token_config
cargo test -p kamn-core --test token_config_docs
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo fmt --check
cargo clippy -- -D warnings
```
