# Spec: Issue #5937 - Task: Add async and real HTTP/WebSocket/TLS integration coverage for service/runtime

- Issue: #5937
- Status: Implemented
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5920

## Problem Statement
Current coverage misses core async/network interactions and secure transport integration.

## Scope
In scope:
- Add async runtime tests plus HTTP, websocket, and TLS integration suites with real servers.

Out of scope:
- Synthetic-only unit assertions without end-to-end validation.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Async runtime code paths are exercised by non-trivial integration tests.
- AC-2: HTTP/WebSocket/TLS integration lanes run in CI and validate real protocol behavior.
- AC-3: Regression tests capture prior gaps (auth chain, ws persistence, tls config failures).
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Async runtime code paths are exercised by non-trivial integration tests.
- C-02 (Functional, AC-2): Verify HTTP/WebSocket/TLS integration lanes run in CI and validate real protocol behavior.
- C-03 (Functional, AC-3): Verify Regression tests capture prior gaps (auth chain, ws persistence, tls config failures).
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.

## Implementation Evidence
- `integration_service_api_endpoint_async_runtime_handles_concurrent_http_routes`
- `integration_service_api_endpoint_tls_mode_serves_required_https_routes`
- `integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event`
- `integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event`
- `integration_service_api_client_reads_websocket_event_frame`
- CI fast-gate selector/output + lane step wiring:
  - `scripts/ci/select_targets.sh` (`run_service_runtime_network_integration_tests`)
  - `.github/workflows/ci-fast-gate.yml` (`Run service/runtime HTTP+WS+TLS integration lane`)


## Required Test Categories
- Unit: async helper/fixture utilities
- Functional: runtime/API behavior cases
- Integration: real server/client HTTP+WS+TLS
- Regression: previously missing protocol-path tests
- Performance: integration lane budget tracked

## Dependencies
- #5920
