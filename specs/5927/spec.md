# Spec: Issue #5927 - Task: Replace synthetic daemon tick loop behavior with real queue processing

- Issue: #5927
- Status: Implemented
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5917

## Problem Statement
Current tick loop increments counters and executes hardcoded scenarios without real message processing.

## Scope
In scope:
- Implement queue polling, work dispatch, and lifecycle updates per tick.

Out of scope:
- Daemon redesign unrelated to message processing correctness.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Tick loop processes queued message work and updates durable state.
- AC-2: Telemetry reflects real processed work, not fabricated counters.
- AC-3: Runtime tests prove processing continues across ticks and restart boundaries.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): `main_tests::runtime_tests::integration_runtime_daemon_relay_drain_projects_message_state_to_relayed` verifies daemon tick loop drains queue entries and updates durable state.
- C-02 (Functional, AC-2): `main_tests::runtime_tests::integration_runtime_daemon_relay_drain_projects_message_state_to_relayed` and `main_tests::runtime_tests::integration_runtime_daemon_processes_relay_entries_arriving_during_tick_loop` assert daemon observability throughput/latency are non-zero for real relay work.
- C-03 (Integration, AC-3): `main_tests::runtime_tests::integration_runtime_daemon_processes_relay_entries_arriving_during_tick_loop` verifies continued processing across ticks; `main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract` verifies restart boundary behavior with projected delivery state.
- C-04 (Verify, AC-4): `cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_daemon_relay_drain_projects_message_state_to_relayed -- --exact`, `cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_daemon_processes_relay_entries_arriving_during_tick_loop -- --exact`, `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact`, `cargo fmt --check`, and `cargo clippy -p kamn-node --bin kamn-node -- -D warnings` pass.

## Success Metrics / Observable Signals
- Daemon tick loop consumes relay spool queue entries and applies durable state projections.
- Observability metrics report non-zero throughput/latency when relay work is processed.
- Relay processing continues across multi-tick runs and remains coherent across runtime restart boundaries.
- Scoped verification commands pass without regressions.


## Required Test Categories
- Unit: tick scheduler and queue worker behavior
- Functional: daemon processes queued messages
- Integration: daemon + API + store
- Regression: synthetic counter-only mode removed from production path
- Performance: bounded per-tick processing time

## Dependencies
- #5917
