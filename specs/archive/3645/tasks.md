# Tasks - Issue #3645

- [x] T1 (Red): define reproducer and capture/report drift scenarios.
- [x] T2 (Green): deliver deterministic reproducer matrix and capture-report behavior.
- [x] T3 (Refactor/Docs): preserve reproducer artifact and summary marker references.
- [x] T4 (Verify): run reproducer, quarantine-lane, and report suites.

## Planned Verification Commands

- `bash scripts/ci/test_run_flaky_reproducer.sh`
- `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`
- `bash scripts/ci/test_post_flaky_report_comment.sh`
- `bash scripts/ci/test_sync_flaky_registry_issues.sh`
