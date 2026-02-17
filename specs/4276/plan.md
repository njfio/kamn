# Plan — #4276

Status: Reviewed

- Implement `scripts/ci/check_websocket_session_ci_smoke_convergence.py` by following existing convergence checker patterns.
- Add fixture-driven shell test harness `scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh` with RED/GREEN coverage.
- Wire smoke commands + checker invocation into fast/full `scripts/ci/test_ci_tools.sh` paths.
