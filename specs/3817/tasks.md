# Tasks - Issue #3817

- [x] T1 (Red): define deterministic drift/failure scenarios for this issue scope.
- [x] T2 (Green): deliver stable contract behavior for mapped shutdown/signal suites.
- [x] T3 (Refactor/Docs): preserve marker and governance traceability.
- [x] T4 (Verify): run mapped conformance suites.

## Planned Verification Commands

- 'bash scripts/ci/test_run_daemon_os_signal_reproducer.sh'
- 'bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh'
- 'bash scripts/runtime/test_validate_daemon_os_signal_live.sh'
