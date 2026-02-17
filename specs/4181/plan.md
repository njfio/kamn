# Plan — Issue #4181

## Approach

- Add `scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py` to validate:
  - version report schema/taxonomy/csv/rehearsal markers,
  - fork policy report schema/taxonomy/csv/rehearsal markers,
  - expected final decision and CI fast-gate state.
- Integrate checker in `scripts/kolme/contracts/version_compatibility_contract_lane.py`.
- Extend `scripts/kolme/test_run_version_compatibility_contract_lane.sh` with mismatch fixtures.
- Update docs:
  - `docs/ops/configuration.md`
  - `docs/foundation/release-gonogo-checklist.md`
- Extend docs tests:
  - `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
  - `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Affected Modules

- `scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py` (new)
- `scripts/kolme/contracts/version_compatibility_contract_lane.py`
- `scripts/kolme/test_run_version_compatibility_contract_lane.sh`
- `docs/ops/configuration.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: checker reason set drifts from docs.
  - Mitigation: docs-contract assertions for taxonomy + reason-code CSV.
- Risk: contract lane runtime inflation.
  - Mitigation: checker operates on already-generated reports and keeps bounded runtime.

## Interfaces/Contracts

- Checker CLI:
  - `--version-report-file`
  - `--fork-policy-report-file`
  - `--expected-final-decision GO|NO-GO`
  - `--ci-fast-gate PASS|FAIL`
  - `--output-json` (optional)
- Output markers:
  - `reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1`
  - deterministic `reason_codes_csv` and `reason_codes_value`
  - `final_decision=GO|NO-GO`

## ADR

- Not required (bounded checker and docs/test integration).
