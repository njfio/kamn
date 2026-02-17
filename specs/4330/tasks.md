# Tasks: Issue #4330

Status: Reviewed
Issue: #4330

## Ordered Tasks

T1 (RED)
- Add failing-first tests for:
  - SIGINT-first OS-signal capture in daemon shutdown flow.
  - OS-signal timeout-phase supervisor stop marker parity in full runtime flow.

T2 (GREEN)
- Keep shutdown behavior deterministic under new assertions.
- Ensure timeout-phase marker assertions reflect stable fail-closed reason format.

T3 (DOCS)
- Update `docs/ops/configuration.md` with explicit signal-failure matrix reference.

T4 (VERIFY)
- Run:
  - `cargo test -p kamn-node daemon_shutdown::tests::integration_daemon_completion_with_os_signals_applies_sigint_graceful_shutdown -- --exact`
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity -- --exact`
  - `cargo test -p kamn-node daemon_shutdown::tests::integration_daemon_completion_with_os_signals_applies_graceful_shutdown -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`

## TDD Evidence

- RED command/output:
  - New tests introduced to fail if SIGINT path or timeout phase-marker parity regresses.

- GREEN command/output:
  - Commands in T4 pass after implementation.

- Regression summary:
  - Existing SIGTERM OS-signal graceful path selector remains green.
