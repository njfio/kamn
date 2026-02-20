# Issue #3999 Plan

- Issue: #3999
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Implementation Approach

1. Deliver dry-run capacity governance checker and threshold fixtures (subtask #4006).
2. Add deterministic docs/runbook parity and remediation marker contracts (subtask #4007).
3. Compose checker/docs/CI selector contracts and verify targeted conformance suites.

## Affected Modules

- `scripts/ci/check_capacity_ci_dry_run_governance.py`
- `fixtures/ci/capacity_ci_dry_run_governance_thresholds.env`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/capacity_ci_dry_run_governance_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: checker/docs/fixture taxonomy drift.
  - Mitigation: explicit docs-contract assertions and deterministic reason/remediation marker checks.
- Risk: accidental heavy-run leakage into fast-gate.
  - Mitigation: CI tools and workflow exclusion contract assertions in integration tests.

## ADR

- Not required (governance checker/docs/test wiring only; no dependency/protocol change).
