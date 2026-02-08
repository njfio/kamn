# Observability Stack and SLO Dashboard Baseline (Issues #206, #593)

This document captures the first implementation slice for deterministic observability and SLO health reporting.

## Scope Delivered
- Added `crates/kamn-core/src/observability.rs` with:
  - `ObservabilitySample` input model.
  - `ObservabilitySloProfile` baseline thresholds for latency, throughput, error rate, and availability.
  - `ObservabilityMonitor` evaluator with alert generation and historical rollup.
  - `ObservabilityReport` and `ObservabilitySnapshot` outputs.
  - `ObservabilityAlert`, `ObservabilityMetric`, `ObservabilitySeverity`, and `ObservabilityHealth`.
  - `ObservabilityError` typed validation failures.
- Added integration tests in `crates/kamn-core/tests/observability_stack.rs`.
- Added frontend dashboard projection linkage in `packages/kamn-dashboard`:
  - deterministic severity mapping from SLO values to UI badge classes.
  - stale snapshot banner behavior for operator triage.

## SLO Evaluation Rules
- `LatencyP50`: warning when above max threshold.
- `LatencyP99`: critical when above max threshold.
- `Throughput`: warning when below minimum threshold.
- `ErrorRate`:
  - warning when above max threshold.
  - critical when above 2x max threshold.
- `Availability`: critical when below minimum threshold.

## Dashboard Rollup Semantics
- Snapshot fields are deterministic:
  - total sample count.
  - healthy/degraded/critical sample counts.
  - latest health status.
- Overall health for each sample:
  - `Critical` if any critical alert exists.
  - `Degraded` if warning alerts exist without critical alerts.
  - `Healthy` when no alerts exist.
- Frontend dashboard mapping:
  - critical samples map to `severity-critical` badges.
  - stale snapshots map to `stale-data-banner` indicators.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test observability_stack
npm --prefix packages/kamn-dashboard test
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
