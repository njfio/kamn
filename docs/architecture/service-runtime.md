# Service Runtime Shutdown Contracts

This document defines deterministic shutdown marker semantics for daemon and
full runtime execution paths.

## Scope

- Runtime mode: `daemon`, `full`
- Trigger sources:
- deterministic signal-tick controls (`--daemon-shutdown-signal-tick`)
- OS signals (`--daemon-shutdown-os-signals`) on Unix

## Drain Marker Contract

`shutdown_drain_status` is emitted on daemon completion and full-supervisor stop
markers with these values:

- `completed`: shutdown drain target completed before timeout budget.
- `timeout`: shutdown drain exceeded timeout budget and failed closed.
- `not-signaled`: no shutdown signal triggered during runtime tick budget.

## Snapshot Flush Marker Contract

`shutdown_snapshot_flush_status` is emitted on daemon completion and full
supervisor stop markers with these values:

- `snapshot-flushed`: graceful shutdown path committed final snapshot flush.
- `snapshot-flush-timeout`: timeout shutdown path emitted forced final flush
  marker and failed closed.
- `snapshot-not-requested`: no signal-triggered shutdown occurred.

## Fail-Closed Validation

The full-supervisor stop contract validator enforces deterministic consistency
between completion reason, drain status, and snapshot flush status:

- Unknown status values are rejected.
- `tick-budget-exhausted` requires `not-signaled` + `snapshot-not-requested`.
- `graceful-shutdown:*` requires `completed` + `snapshot-flushed`.
- `graceful-shutdown-timeout:*` requires `timeout` + `snapshot-flush-timeout`.

Invalid combinations emit deterministic reason codes and fail closed.
