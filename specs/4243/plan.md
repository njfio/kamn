# Plan — #4243 Implement Replay Taxonomy and Runbook Parity Contract

Status: Reviewed

## Approach

1. Add replay taxonomy/runbook constants and marker emitters in
   `sqlite_crash_recovery_live_contract.py`.
2. Add `--runbook-file` policy input (defaulting to `docs/deploy/kolme_devnet_ops.md`) and runbook
   marker parity validation.
3. Add deterministic reason mapping:
   - replay taxonomy drift
   - runbook marker parity mismatch
   - replay runbook taxonomy marker mismatch
4. Update contract lane to pass runbook path and surface new markers in lane reports.
5. Update deploy/release docs and Rust docs-contract tests.

## Affected Surfaces

- `scripts/runtime/sqlite_crash_recovery_live_contract.py`
- `scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: policy reason mapping ambiguity on multiple mismatches.
  Mitigation: deterministic preferred reason ordering and explicit regression assertions.
- Risk: docs drift after marker additions.
  Mitigation: update include_str docs tests in same change.

ADR: not required.
