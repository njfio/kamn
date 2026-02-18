# Tasks - Issue #3798

- [x] T1 (Red): define capture-lane summary marker drift scenarios.
- [x] T2 (Green): deliver deterministic capture-lane and summary behavior.
- [x] T3 (Refactor/Docs): preserve flaky-capture governance references.
- [x] T4 (Verify): run quarantine-lane and report/registry suites.

## Planned Verification Commands

- `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`
- `bash scripts/ci/test_post_flaky_report_comment.sh`
- `bash scripts/ci/test_sync_flaky_registry_issues.sh`
