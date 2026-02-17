# Plan — #4196 Deterministic Full-Stack Harness Checker Reason Outputs

## Approach
1. Extend `scripts/runtime/full_io_scenario_matrix_live_contract.py` policy-check path with explicit reason-taxonomy constants, canonical reason-codes CSV, and deterministic reason-codes value outputs for both pass/fail.
2. Extend `scripts/runtime/test_check_full_io_scenario_matrix_live_policy.sh` to assert:
- deterministic reason marker output on success,
- deterministic reason mapping output on fail-closed mismatches,
- repeated-run output stability.
3. Add release checklist gate text to `docs/foundation/release-gonogo-checklist.md` for full-stack harness checker reason outputs.
4. Add docs-contract assertions in `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`.

## Affected Modules
- `scripts/runtime/full_io_scenario_matrix_live_contract.py`
- `scripts/runtime/test_check_full_io_scenario_matrix_live_policy.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks / Mitigations
- Risk: reason-code ordering drift between `decision_reasons`, `failed_checks`, and emitted stdout markers.
- Mitigation: emit reason codes from a single canonical ordered list and assert it in tests.

- Risk: docs contract brittleness.
- Mitigation: assert stable, policy-significant markers only (taxonomy/version/codes/regression anchors), not prose phrasing.

## Interfaces / Contracts
- Policy checker deterministic markers:
  - `full_io_harness_policy_reason_taxonomy_version`
  - `full_io_harness_policy_reason_codes_csv`
  - `full_io_harness_policy_reason_codes_value`
- Fail-closed mappings include existing checker reasons such as:
  - `full_io_scenario_matrix_policy_process_harness_mismatch`
  - `full_io_scenario_matrix_policy_dry_run_command_count_mismatch`
  - `full_io_scenario_matrix_policy_dry_run_command_status_mismatch`
  - `full_io_scenario_matrix_policy_expected_decision_mismatch`
