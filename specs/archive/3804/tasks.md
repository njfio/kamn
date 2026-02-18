# Tasks - Issue #3804

- [x] T1 (Red): define evidence-report and docs-policy drift scenarios.
- [x] T2 (Green): deliver deterministic evidence-report emission behavior.
- [x] T3 (Refactor/Docs): preserve policy marker parity and remediation references.
- [x] T4 (Verify): run anti-flake report and registry-sync suites.

## Planned Verification Commands

- `bash scripts/ci/test_post_flaky_report_comment.sh`
- `bash scripts/ci/test_sync_flaky_registry_issues.sh`
- `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`
