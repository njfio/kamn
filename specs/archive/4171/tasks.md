# Tasks — Issue #4171

- [x] T1 (Red): add failing custody/rotation convergence contract tests before checker exists.
  Evidence:
  - `bash scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh` failed with:
    `No such file or directory`.
- [x] T2 (Green): implement custody/rotation CI smoke convergence checker with deterministic
  taxonomy outputs and fail-closed boundaries.
  Evidence:
  - Added `scripts/ci/check_custody_rotation_ci_smoke_convergence.py`.
  - `bash scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh` passed.
- [x] T3 (Refactor): wire convergence test into ci-tools fast/full suites.
  Evidence:
  - Added `bash "$ROOT_DIR/scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh"` to
    fast/full branches in `scripts/ci/test_ci_tools.sh`.
- [x] T4 (Verify): run targeted and fast-mode regression suites.
  Evidence:
  - `bash scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh`
  - `bash scripts/ci/test_production_service_next_steps_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
