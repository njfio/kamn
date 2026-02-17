# Plan — #4237 Replay Idempotency Taxonomy and Runbook Marker Parity

Status: Reviewed

## Approach

1. Extend sqlite crash-recovery contract constants/outputs with replay idempotency taxonomy mapping
   and runbook parity marker family.
2. Add policy `--runbook-file` input and required runbook marker checks.
3. Wire deterministic fail-closed reason mapping for:
   - replay taxonomy drift
   - replay runbook taxonomy marker mismatch
   - runbook marker parity mismatch
4. Add/extend policy and contract-lane red tests for taxonomy drift + runbook divergence.
5. Update deploy/release docs and Rust docs-contract tests.

## Affected Surfaces

- `scripts/runtime/sqlite_crash_recovery_live_contract.py`
- `scripts/runtime/test_check_sqlite_crash_recovery_live_policy.sh`
- `scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh`
- `scripts/runtime/test_validate_sqlite_crash_recovery_live_contract_lane.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: runbook parity checks become brittle on prose-only edits.
  Mitigation: enforce explicit marker strings only; avoid full paragraph coupling.
- Risk: new reason mapping introduces non-deterministic failure ordering.
  Mitigation: assert exact deterministic reason codes in shell regression tests.

## Interface Notes

New marker family:
- `replay_idempotency_taxonomy_mapping_status`
- `runbook_marker_parity_status`
- `replay_idempotency_runbook_reason_taxonomy_version`
- `replay_idempotency_runbook_reason_codes_csv`
- `replay_idempotency_runbook_reason_code`

New deterministic fail-closed reasons:
- `replay_idempotency_taxonomy_mapping_drift_detected`
- `runbook_marker_parity_mismatch`
- `sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_taxonomy_version_mismatch`
- `sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_codes_csv_mismatch`

ADR: not required (no dependency/protocol architecture change).
