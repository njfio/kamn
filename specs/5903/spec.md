# Spec: Issue #5903 - Replace Static Service API Observability with Live Runtime Telemetry

- Issue: #5903
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The service API server currently renders `/healthz` and `/metrics` observability fields from static bootstrap snapshots. In API mode, this commonly yields static `unknown` + zero-value observability that does not reflect real request traffic and error behavior.

## Scope
In scope:
- Add runtime telemetry tracking for service API request outcomes and latencies.
- Derive live observability projections (latency p50/p99, throughput TPS, error rate bps, availability bps, health, alert_count).
- Project live observability values into `/metrics` and `/healthz` responses in server mode.
- Add contract/regression tests that validate telemetry changes under traffic.

Out of scope:
- Cross-process metrics aggregation across daemon + API + kolme-live.
- Replacing current taxonomy/version marker metrics.
- Prometheus push/export redesign.

## Acceptance Criteria
### AC-1 Runtime request telemetry is captured
Given a running service API endpoint,
When requests are processed,
Then the runtime tracks total requests, error requests, and request latency samples.

### AC-2 Metrics observability values are live
Given mixed success and failure traffic,
When `/metrics` is queried,
Then observability values are derived from runtime telemetry and are not fixed bootstrap constants.

### AC-3 Health observability projects runtime status
Given runtime telemetry has recorded requests,
When `/healthz` is queried,
Then observability source/health are projected from runtime telemetry.

### AC-4 Regression tests fail closed on static drift
Given future refactors,
When runtime observability projection is removed or bypassed,
Then service API tests fail on expected live metric/health markers.

## Conformance Cases
- C-01 (AC-1, Integration): request path tests assert runtime counters progress after served traffic.
- C-02 (AC-2, Integration): `/metrics` after traffic reports non-static runtime-derived observability values.
- C-03 (AC-3, Integration): `/healthz` after traffic reports runtime observability source/health markers.
- C-04 (AC-4, Regression): test fails if metrics endpoint reverts to static unknown/zero projection under served traffic.

## Success Metrics / Observable Signals
- Service API `/metrics` observability lines change deterministically with runtime request outcomes.
- Service API `/healthz` observability markers reflect runtime telemetry once request samples exist.
- New/updated tests in `service_api_endpoint_tests` pass and enforce live projection behavior.

## Verification Summary
- `cargo test -p kamn-node regression_service_api_runtime_observability_projects_live_metrics_under_traffic -- --nocapture`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests:: -- --nocapture`
- `cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract -- --nocapture`
- `cargo clippy -p kamn-node -- -D warnings`
- `cargo fmt --check`
- `cargo mutants --in-diff /tmp/issue-5903.diff -p kamn-node --minimum-test-timeout 120` (4/4 caught)
