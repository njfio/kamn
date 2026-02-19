# Plan — #4236 Journal Append and Checkpoint Marker Integrity

Status: Reviewed

## Approach

1. Add append/checkpoint integrity marker constants and output fields in `sqlite_crash_recovery_live_contract.py`.
2. Extend policy checks with append/checkpoint parity fail-closed reason mapping.
3. Extend crash-recovery policy regression script with WAL append mismatch and parity mismatch fixtures.
4. Align runtime lane tests and contract-lane tests to assert new deterministic markers.
5. Update ops/release/CI docs and docs contract tests.

## Affected Surfaces

- `scripts/runtime/sqlite_crash_recovery_live_contract.py`
- `scripts/runtime/test_check_sqlite_crash_recovery_live_policy.sh`
- `scripts/runtime/test_validate_sqlite_crash_recovery_live.sh`
- `scripts/runtime/test_validate_sqlite_crash_recovery_live_contract_lane.sh`
- `docs/ops/configuration.md`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: widening marker surface causes docs drift.
  Mitigation: add explicit docs-contract assertions in Rust doc tests.
- Risk: regression fixtures become brittle.
  Mitigation: assert deterministic reason markers only, not full output ordering.

## Interface Notes

- New deterministic marker family:
  - `append_checkpoint_integrity_status=verified`
  - `append_checkpoint_reason_taxonomy_version=...`
  - `append_checkpoint_reason_codes_csv=...`
- New fail-closed policy reason:
  - `sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch`

ADR: not required (no dependency or protocol architecture change).
