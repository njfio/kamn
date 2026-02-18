# Tasks - Issue #3647

- [x] T1 (Red): define unresolved-flake merge-policy and evidence drift scenarios.
- [x] T2 (Green): deliver deterministic gate-policy and evidence-report behavior.
- [x] T3 (Refactor/Docs): preserve anti-flake taxonomy and remediation marker references.
- [x] T4 (Verify): run merge-gate policy and evidence-report suites.

## Planned Verification Commands

- `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`
- `bash scripts/ci/test_check_anti_flake_policy.sh`
- `bash scripts/ci/test_post_flaky_report_comment.sh`
- `bash scripts/ci/test_sync_flaky_registry_issues.sh`
