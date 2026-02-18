# Tasks - Issue #3631

- [x] T1 (Red): define flaky reproduction, recurrence, metadata, and merge-policy drift scenarios.
- [x] T2 (Green): deliver deterministic anti-flake behavior via child tasks `#3645`, `#3646`, and `#3647`.
- [x] T3 (Refactor/Docs): preserve anti-flake governance marker and remediation references.
- [x] T4 (Verify): run full anti-flake reproducer, recurrence, metadata, and merge-gate suites.

## Planned Verification Commands

- `bash scripts/ci/test_run_flaky_reproducer.sh`
- `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`
- `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh`
- `bash scripts/ci/test_check_flaky_registry.sh`
- `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
- `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh`
- `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`
- `bash scripts/ci/test_check_anti_flake_policy.sh`
- `bash scripts/ci/test_post_flaky_report_comment.sh`
- `bash scripts/ci/test_sync_flaky_registry_issues.sh`
