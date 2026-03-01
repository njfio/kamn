# Spec: Issue #6283 - Observability event projection contract

## Objective

Add a deterministic projection API that converts `ObservabilityReport` into a typed,
machine-readable event payload for unified tracing/log ingestion across runtime call sites.

## Inputs/Outputs

- Inputs:
  - `ObservabilityReport` produced by `ObservabilityMonitor::evaluate`.
- Outputs:
  - `ObservabilityEventProjection` containing:
    - `health`
    - `alert_count`
    - ordered `reason_codes`
    - sample timestamp

## Boundaries/Non-goals

- In scope:
  - New projection types/functions in `kamn-core` observability module.
  - Integration tests for healthy/degraded/critical outcomes.
- Out of scope:
  - Exporter plumbing (Prometheus/OpenTelemetry).
  - Global logging/tracing framework migration.
  - Script/workflow changes.

## Failure Modes

- FM-1: projection omits reason codes for one or more `(metric, severity)` combinations.
- FM-2: reason-code order is unstable between runs for identical input.
- FM-3: healthy reports emit non-empty reason codes.
- FM-4: projection drops required metadata (health/alert_count/timestamp).

## Acceptance Criteria

- AC-1: `ObservabilityReport` can be projected into a typed event payload with deterministic fields.
- AC-2: projection maps every alert to a stable reason code format:
  `observability_<metric>_<severity>_threshold_breached`.
- AC-3: reason codes in projection are ordered by alert encounter order and are deterministic.
- AC-4: healthy reports project zero reason codes with `alert_count == 0`.
- AC-5: Existing observability suites in `kamn-core` remain green.

## Files To Touch

- `crates/kamn-core/src/observability.rs`
- `crates/kamn-core/src/lib.rs` (if re-exports are required)
- `crates/kamn-core/tests/observability_stack.rs`
- `specs/6283-observability-event-projection-contract.md`

## Error Semantics

- Projection itself is infallible because source data is a validated `ObservabilityReport`.
- Validation failures remain in `ObservabilityMonitor::evaluate` and continue returning
  `Result<_, ObservabilityError>` with existing typed errors.

## Test Plan

- RED:
  - Add failing integration tests asserting projected payload fields for:
    - healthy sample
    - degraded sample
    - critical sample
  - Add failing assertions for deterministic reason-code mapping and ordering.
- GREEN:
  - Implement minimal projection type + mapping logic to satisfy tests.
- REFACTOR:
  - Extract helper mapping function for metric/severity -> reason code.
  - Ensure names and function sizes stay within AGENTS constraints.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
  - `cargo test -p kamn-core --test observability_stack`
