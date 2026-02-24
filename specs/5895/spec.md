# Issue 5895 Spec - Live Runtime De-syntheticization

- Issue: #5895
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Status: Implemented
- Priority: P0

## Problem Statement
Runtime behavior in `kamn-node` still relies on synthetic projection paths in critical areas:
- daemon execution does not run a real per-tick message processing loop,
- observability metrics can fall back to fabricated zero/default projections,
- websocket events route is one-shot,
- delivery progression relies on narrow projection paths instead of continuous runtime processing.

This prevents the runtime from behaving like a continuously operating message service under daemon/full execution modes.

## Scope
In scope:
- `kamn-node` daemon runtime tick processing for relay spool -> message lifecycle projection.
- Runtime-measured daemon/service observability derivation used by `/metrics` and report snapshots.
- Persistent websocket event streaming semantics for `/v1/events/ws`.
- End-to-end relay->delivery lifecycle wiring validation across send/query/websocket.

Out of scope:
- New external dependencies.
- Wire protocol redesign.
- Cross-repo changes outside this repository.

## Acceptance Criteria

### AC-1 Daemon Tick Loop Processes Real Relay Work
Given daemon/full runtime execution with relay spool entries present,
when daemon ticks execute,
then each tick must process available relay entries and advance state projection through runtime operations (not hardcoded scenario-only projection).

### AC-2 Observability Metrics Are Runtime-Measured
Given daemon/service runtime execution,
when `/metrics` and report observability fields are emitted,
then latency/throughput/error/availability values must be derived from runtime-measured processing outcomes rather than fabricated unknown-zero defaults.

### AC-3 Websocket Events Are Persistent Streams
Given a successful websocket upgrade on `/v1/events/ws`,
when runtime events are available,
then the server must stream multiple event frames over one upgraded connection until deterministic close/error conditions are met.

### AC-4 Delivery Lifecycle Is Runtime-Wired
Given a message send request with recipient routing,
when daemon processing runs and recipient queries follow,
then lifecycle transitions must follow `created -> relayed -> delivered` through runtime-processed state transitions.

## Conformance Cases
- C-01 (Integration, AC-1, AC-4): send message, run daemon ticks, verify relay spool drain count > 0 and state projection reaches `relayed`.
- C-02 (Integration, AC-4): recipient query after relay processing transitions `relayed` to `delivered`.
- C-03 (Functional, AC-2): `/metrics` includes non-placeholder runtime-derived observability values and source marker indicating runtime-measured path.
- C-04 (Integration, AC-3): websocket route emits at least two ordered event frames on a single upgraded connection before deterministic close.
- C-05 (Regression, AC-1): daemon execution remains bounded by max tick/shutdown contracts while processing loop is active.

## Success Metrics / Observable Signals
- Daemon relay processing logs include per-tick processed/projection counts.
- Service API metrics expose runtime-measured observability values during daemon/full runs.
- Websocket integration tests verify multi-frame streaming contract.
- Targeted suite passes:
  - `cargo test -p kamn-node -- main_tests::daemon_tests::runtime_contract_tests`
  - `cargo test -p kamn-node -- main_tests::service_api_endpoint_tests`
