# Plan — #4283

Status: Reviewed

## Approach

- Extend `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh` with a dedicated convergence-check mode that compares:
  - preflight drift summary report
  - preflight policy report
  - promotion decision reason mapping markers
- Add RED tests first in `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh` for:
  - missing evidence-link marker rejection
  - tampered payload rejection
  - deterministic repeated mismatch ordering
- Emit deterministic promotion decision reason mapping fields in policy output.
- Update docs and docs-contract tests:
  - `docs/planning/kolme-devnet-ops.md`
  - `docs/foundation/release-gonogo-checklist.md`
  - `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
  - `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Affected Areas

- `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`
- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`
- `scripts/runtime/test_run_failover_sync_drill_suite.sh`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: checker and docs marker drift.
  - Mitigation: docs-contract assertions for command and marker strings.
- Risk: nondeterministic failure reason ordering.
  - Mitigation: centralized deterministic reason insertion order and repeated-run regression assertions.

## Interfaces and Contracts

- New checker mode:
  - `check-evidence-convergence --report-file <path> --policy-file <path> [--output-json <path>]`
- Policy output contract extension:
  - promotion decision reason taxonomy markers
  - deterministic promotion decision reason code mapping
