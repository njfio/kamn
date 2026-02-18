# Tasks — Issue #4878

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: `bash scripts/lib/test_test_harness_migration_contract.sh` failed with `expected at least 75 migrated shell test scripts to source test_harness.sh, found: 4`.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: migrated 96 high-duplication `test_*.sh` scripts to source `scripts/lib/test_harness.sh` and replaced canonical `if [ ! -x/-f ... ]` blocks with shared `test_harness_require_*` helpers.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: `git diff --stat` for migration scope shows `357 insertions(+), 984 deletions(-)` across migrated script families.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/lib/test_test_harness_migration_contract.sh` -> passed (`migrated_script_count=100 helper_usage_count=100`)
    - `bash scripts/ci/test_readme_contract.sh` -> passed
    - `bash scripts/ci/test_check_no_production_expect.sh` -> passed
    - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` -> passed
