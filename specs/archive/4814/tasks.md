# Tasks — Issue #4814

- [x] T1 (Red): Add failing migration tests before helper implementations.
  - `#4825`: `scripts/lib/test_test_harness_migration_contract.sh` failed before harness existed.
  - `#4826`: `scripts/lib/test_json_write_helper_migration_contract.sh` failed before JSON helper primitives existed.
- [x] T2 (Green): Implement shared harness/JSON helper infrastructure.
  - `#4825` merged via PR `#4842`.
  - `#4826` merged via PR `#4843`.
- [x] T3 (Refactor): Migrate high-duplication script cohorts to helper-based patterns.
  - Wrapper-family baseline/trend test impl consolidation (`#4825`).
  - JSON write helper migration across 89 scripts / 168 write sites (`#4826`).
- [x] T4 (Verify): Run conformance/regression suites and update process logs.
  - Harness migration contract: pass.
  - JSON helper migration contract: pass.
  - Full CI tool regression: `bash scripts/ci/test_ci_tools.sh` pass after each subtask.
