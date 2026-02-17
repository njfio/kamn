# Plan — #4239

Status: Reviewed

## Approach

- Add a new deterministic CI smoke convergence checker for sqlite crash-recovery replay-integrity governance.
- Validate four required smoke commands in `scripts/ci/test_ci_tools.sh` fast-mode block.
- Validate heavy sqlite run-mode exclusion in both `.github/workflows/ci-fast-gate.yml` and ci-tools fast mode.
- Update CI strategy and production next-steps docs with new taxonomy/boundary markers.
- Extend docs-contract tests to pin required markers and active chain references.

## Affected Areas

- `scripts/ci/check_sqlite_crash_recovery_ci_smoke_convergence.py`
- `scripts/ci/test_check_sqlite_crash_recovery_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks and Mitigations

- Risk: taxonomy/marker drift between checker code and docs/tests.
  - Mitigation: keep reason taxonomy constants in checker and assert exact marker strings in docs-contract tests.
- Risk: adding smoke commands increases fast-mode runtime.
  - Mitigation: checker enforces smoke-only tests, keeps run-mode lanes excluded, and enforces 120s CI smoke budget marker.
