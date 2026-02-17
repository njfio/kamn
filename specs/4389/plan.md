# Plan — #4389

Status: Reviewed

## Approach

- Extend persistence live validation tests with RED assertions for deterministic taxonomy/version and tamper/freshness gating markers.
- Implement persistence marker/taxonomy/boundary outputs in runtime validation script.
- Add fail-closed mismatch checks for persistence evidence marker drift.
- Update docs and docs-contract tests for persistence marker parity.
- Run targeted script + docs-contract suites, then repo gates.

## Affected Areas

- `scripts/runtime/test_validate_persistence_adapters_live.sh`
- `scripts/runtime/validate_persistence_adapters_live.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations

- Risk: stricter marker checks may fail existing lanes unexpectedly.
  - Mitigation: add deterministic reason codes and update tests/docs in same change.
- Risk: CI budget drift from additional checks.
  - Mitigation: keep checks marker-level and bounded; preserve smoke/local-heavy split.

## Interfaces / Contracts

- Add deterministic persistence marker outputs:
  - `persistence_gate_reason_taxonomy_version`
  - `persistence_gate_reason_codes_csv`
  - `persistence_tamper_freshness_drift_fail_closed_status`
  - `persistence_evidence_completeness_status`
  - `persistence_ci_smoke_local_heavy_boundary_status`
