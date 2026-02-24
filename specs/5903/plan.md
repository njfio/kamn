# Plan: Issue #5903 - Replace Static Service API Observability with Live Runtime Telemetry

## Approach
1. Add runtime telemetry state to `ServiceApiRuntimeState` and update it on each request outcome.
2. Implement projection helper that computes p50/p99 latency, throughput, error rate, availability, and health from telemetry.
3. Render `/healthz` and `/metrics` in live server path from a runtime-updated snapshot.
4. Add/adjust integration tests to prove runtime observability values change with traffic and fail closed on regression.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/runtime_observability.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: test contracts currently assert static `unknown` markers.
  - Mitigation: limit behavior change to live server request path and update only affected assertions.
- Risk: high-lock contention in request path telemetry updates.
  - Mitigation: bounded sample retention and minimal critical-section work.

## Interfaces / Contracts
- No public CLI or wire-format route changes.
- `/healthz` and `/metrics` observability values become runtime-derived when request telemetry exists.

## ADR
- Not required (no new dependency or protocol shape change).
