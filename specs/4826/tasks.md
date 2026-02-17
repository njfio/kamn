# Tasks — Issue #4826

- [x] T1 (Red): add failing migration contract test before helper implementation.
  - Added `scripts/lib/test_json_write_helper_migration_contract.sh`.
  - RED evidence: failed with `expected write_json_file() helper in common library`.
- [x] T2 (Green): add shared JSON helper primitives and executable helper command.
  - Updated `scripts/lib/common.sh` with:
    - `emit_json_object()`
    - `write_json_file()`
    - `write_json_object()`
    - `write_decision_json()`
  - Added `scripts/lib/write_json_file.sh`.
- [x] T3 (Refactor): run scripted migration over high-duplication JSON heredoc writers.
  - Migrated 89 scripts and 168 manual JSON heredoc write sites to helper command usage.
- [x] T4 (Verify): execute migration contract and full regression.
  - `bash scripts/lib/test_json_write_helper_migration_contract.sh`
  - `bash -n $(git diff --name-only -- '*.sh')`
  - `bash scripts/ci/test_ci_tools.sh`
