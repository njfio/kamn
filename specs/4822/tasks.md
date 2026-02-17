# Tasks — Issue #4822

- [x] T1 (Red): add failing migration contract guard before implementation.
  - Added: `scripts/lib/test_exec_dispatch_registry.sh`
  - RED evidence captured (missing dispatcher executable before implementation).
- [x] T2 (Green): implement dispatcher+registry and migrate eligible wrappers.
  - Added: `scripts/lib/exec_dispatch.sh`, `scripts/lib/exec_dispatch.py`, `scripts/lib/exec_registry.json`
  - Migrated eligible tiny wrappers to symlink dispatcher model.
- [x] T3 (Refactor): update stale wrapper-content assertions to symlink+registry contract checks.
  - Updated affected tests across CI/runtime/sdk/frontend/compliance contract lanes.
  - Updated runtime checker/baseline for symlink-aware LOC accounting.
- [x] T4 (Verify): run required suites and capture deterministic evidence.
  - `bash scripts/lib/test_exec_dispatch_registry.sh` ✅
  - `bash scripts/ci/test_check_non_kolme_wave_trend_test_loc_soft_budget.sh` ✅
  - `bash scripts/ci/test_ci_tools.sh` ✅
