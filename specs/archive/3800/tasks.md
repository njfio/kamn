# Tasks - Issue #3800

- [x] T1 (Red): define reproducer matrix determinism and artifact drift scenarios.
- [x] T2 (Green): deliver deterministic reproducer matrix behavior.
- [x] T3 (Refactor/Docs): preserve artifact-contract references.
- [x] T4 (Verify): run reproducer and quarantine-lane suites.

## Planned Verification Commands

- `bash scripts/ci/test_run_flaky_reproducer.sh`
- `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`
