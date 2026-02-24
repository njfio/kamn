# Plan: Issue #5861 - Service API Relay Spool to Daemon Drain Integration

- Issue: #5861
- Spec: `specs/5861/spec.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Approach
1. Add shared relay spool path and spool I/O helpers in `service_api_endpoint` for append/drain operations.
2. Resolve relay spool path during endpoint server startup and attach it to runtime state.
3. Append relay entries for recipient-addressed `POST /v1/messages/send` after durable message-store write succeeds.
4. Extend daemon runtime options/execution to drain configured relay spool and emit deterministic relay drain telemetry.
5. Add targeted integration/regression tests for API enqueue and daemon drain behavior.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs`

## Risks and Mitigations
- Risk: Relay spool append failures could silently drop delivery intents.
  - Mitigation: fail closed with deterministic `service_api_state_persistence_failed` response and explicit error text.
- Risk: Spool path drift between API and daemon could prevent draining.
  - Mitigation: centralize path derivation in shared helpers and cover with tests.
- Risk: Backward incompatibility for existing route contracts.
  - Mitigation: keep response payload schema unchanged and run regression route tests.

## Interfaces / Contracts
- New internal relay spool entry schema (JSON lines) with message_id/sender/recipient/body.
- Shared path derivation helpers for state-file and relay-spool resolution.
- Daemon runtime relay drain contract:
  - consume all valid entries from spool file;
  - truncate/clear drained file;
  - emit deterministic log markers for drained counts.

## ADR Requirement
- No new dependency or external protocol change; ADR not required.
