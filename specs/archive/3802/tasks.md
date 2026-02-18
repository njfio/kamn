# Tasks - Issue #3802

- [x] T1 (Red): define stale quarantine and metadata drift failure scenarios.
- [x] T2 (Green): deliver deterministic registry and metadata-policy guard behavior.
- [x] T3 (Refactor/Docs): preserve quarantine governance references.
- [x] T4 (Verify): run registry and ignored-test policy suites.

## Planned Verification Commands

- `bash scripts/ci/test_check_flaky_registry.sh`
- `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
- `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh`
