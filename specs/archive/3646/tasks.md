# Tasks - Issue #3646

- [x] T1 (Red): define flaky recurrence and quarantine metadata drift scenarios.
- [x] T2 (Green): deliver deterministic recurrence stability and cleanup-policy behavior.
- [x] T3 (Refactor/Docs): preserve anti-flake cleanup governance references.
- [x] T4 (Verify): run websocket convergence, quarantine lane, and metadata policy suites.

## Planned Verification Commands

- `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh`
- `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`
- `bash scripts/ci/test_check_flaky_registry.sh`
- `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
- `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh`
