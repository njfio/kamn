# Plan — #4246

Status: Reviewed

## Approach

- Add `scripts/ci/check_sqlite_crash_recovery_ci_smoke_convergence.py` using existing CI smoke checker pattern.
- Add `scripts/ci/test_check_sqlite_crash_recovery_ci_smoke_convergence.sh` for pass/fail fixtures.
- Integrate checker test into `scripts/ci/test_ci_tools.sh`.

## Affected Areas

- `scripts/ci/check_sqlite_crash_recovery_ci_smoke_convergence.py`
- `scripts/ci/test_check_sqlite_crash_recovery_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`

## Risks and Mitigations

- Risk: checker diverges from existing CI smoke checker conventions.
  - Mitigation: mirror established structure and reason-order normalization from existing checkers.
