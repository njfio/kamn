# Plan — Issue #4180

## Approach

- Extend existing `scripts/kolme/test_run_version_compatibility_contract_lane.sh` with
  mismatch fixtures that invoke a compatibility marker matrix checker and assert fail-closed
  reasons.
- Capture RED before checker implementation by requiring checker execution in tests.

## Affected Modules

- `scripts/kolme/test_run_version_compatibility_contract_lane.sh`

## Risks and Mitigations

- Risk: brittle fixture handling around temp files.
  - Mitigation: use deterministic JSON tamper steps and strict reason-code assertions.
- Risk: mismatch reasons become non-deterministic.
  - Mitigation: enforce taxonomy version and reason-code ordering checks in fixtures.

## Interfaces/Contracts

- Compatibility marker matrix policy checker command (implemented in sibling issue #4181):
  - `python3 scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py ...`
- Deterministic fail reasons asserted by this issue:
  - `version_report_schema_mismatch`
  - `version_report_reason_taxonomy_mismatch`
  - `fork_policy_report_reason_codes_csv_mismatch`
  - `fork_policy_report_rehearsal_bypass_guard_status_mismatch`

## ADR

- Not required (test-fixture coverage only).
