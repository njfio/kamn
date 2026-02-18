# Tasks — Issue #4867

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: migration contracts for harness adoption and JSON helper usage were tightened with explicit fail-closed ratchets before final rollout.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: merged subtasks #4878/#4879 migrated high-duplication tests to `test_harness.sh` and manual JSON writers to `write_json_file.sh`.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: script families now share reusable harness/helper plumbing, with migration contracts preventing regression to manual duplicated patterns.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/lib/test_test_harness_migration_contract.sh` -> passed
    - `bash scripts/lib/test_json_write_helper_migration_contract.sh` -> passed
    - `bash scripts/ci/test_ci_tools.sh` -> passed
