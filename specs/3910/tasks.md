# Tasks - Issue #3910

- [x] T1 (Red): define websocket convergence instability scenarios.
- [x] T2 (Green): deliver deterministic websocket CI-smoke convergence behavior.
- [x] T3 (Refactor/Docs): preserve anti-flake policy traceability.
- [x] T4 (Verify): run websocket convergence and quarantine-lane suites.

## Planned Verification Commands

- `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh`
- `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`
