# Watchdog Node Prototype for Integrity and Censorship Signals (Issue #204)

This document captures the first watchdog implementation slice for detecting invalid blocks, censorship indicators, and quorum anomalies.

## Scope Delivered
- Added `crates/kamn-core/src/watchdog.rs` with:
  - `WatchdogNode` observation evaluator.
  - `WatchdogConfig` configurable thresholds.
  - `WatchdogObservation` block and gossip delivery inputs.
  - `WatchdogAlert` + `WatchdogAlertKind` outputs.
  - `WatchdogSnapshot` rollup counts.
  - `WatchdogError` typed validation failures.
- Added tests in `crates/kamn-core/tests/watchdog_node.rs`.

## Detection Rules
- Invalid block parent:
  - critical alert when observed block parent does not match previously observed state hash.
- Quorum anomaly:
  - critical alert when block signatures are below minimum configured quorum threshold.
- Censorship signal:
  - warning alert when gossip delivery ratio is below configured threshold.
  - single-recipient deliveries are excluded from censorship classification.

## Snapshot Semantics
`WatchdogSnapshot` tracks deterministic counters:
- total observations
- total alerts
- warning alerts
- critical alerts

## Validation and Error Handling
- Config rejects zero quorum threshold.
- Config rejects delivery ratio outside `1..=100`.
- Observations reject empty identifiers and impossible recipient counts.
- Invalid observation inputs return explicit typed errors.

## Fast and Cost-Effective Validation
This slice keeps PR verification lightweight:

```bash
cargo test -p kamn-core --test watchdog_node
```

The test set is deterministic and sub-second, minimizing CI runtime and cost.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test watchdog_node
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test watchdog_node_docs
cargo test -p kamn-core
```
