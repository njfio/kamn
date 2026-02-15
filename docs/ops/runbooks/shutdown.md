# Runtime Shutdown Runbook

This runbook documents daemon shutdown behavior when the runtime is configured
to consume OS shutdown signals.

## Scope

- Runtime mode: `daemon` and `full` daemon execution path.
- Signal source: local process signals on Unix (`SIGINT`, `SIGTERM`).
- Trigger mode: `--daemon-shutdown-os-signals` enabled and no
  `--daemon-shutdown-signal-tick` overrides provided.

## Signal Handling Contract

1. The first received `SIGINT` or `SIGTERM` opens graceful shutdown.
2. Drain and timeout budgets are resolved from:
- `--daemon-shutdown-drain-ticks`
- `--daemon-shutdown-timeout-ticks`
3. Additional shutdown signals received after the first signal are counted as
   ignored signals and reported in completion telemetry.

## Completion Reason Semantics

- Graceful completion:
- `graceful-shutdown:signal@<tick>;drain_ticks=<n>;timeout_ticks=<n>;ignored_signals=<n>`
- Timeout completion:
- `graceful-shutdown-timeout:signal@<tick>;drain_ticks=<n>;timeout_ticks=<n>;ignored_signals=<n>`
- No signal received:
- `tick-budget-exhausted`

## Operator Validation

Use local tests to validate signal handling contracts:

```bash
cargo test -p kamn-node daemon_shutdown::tests::
```

The test suite covers:
- first-signal graceful shutdown behavior,
- timeout behavior when drain exceeds budget,
- repeated signal accounting (`ignored_signals`),
- fail-closed path verification for the tokio signal runtime path.
