# Plan — #4238

Status: Reviewed

## Approach

- Extend `sqlite_crash_recovery_live_contract.py` with a `check-evidence-convergence` command.
- Add deterministic convergence taxonomy constants and reason mapping checks tied to policy outputs.
- Add a wrapper script and dedicated test lane for crash-replay evidence convergence.
- Wire convergence checker into sqlite contract-lane composition and docs contract surfaces.

## Affected Areas

- `scripts/runtime/sqlite_crash_recovery_live_contract.py`
- `scripts/runtime/check_sqlite_crash_recovery_live_evidence_convergence.sh`
- `scripts/runtime/test_check_sqlite_crash_recovery_live_evidence_convergence.sh`
- `scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh`
- `scripts/runtime/test_validate_sqlite_crash_recovery_live_contract_lane.sh`
- `docs/ci/strategy.md`
- `docs/planning/kolme-devnet-ops.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations

- Risk: reason-taxonomy drift between code, tests, and docs.
  - Mitigation: centralize constants in python checker and update doc-contract assertions in the same change.
- Risk: convergence checker reads stale/non-existent linked artifacts.
  - Mitigation: enforce required `source_report_file` checks and fail closed deterministically.
