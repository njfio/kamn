# Spec: Issue 6375 - Extract service API WebSocket contract tests from monolith

## Objective
Move WebSocket-specific service API endpoint helpers and contract tests out of `service_api_endpoint_tests.rs` into a dedicated module while preserving behavior and assertions.

## Inputs/Outputs
- Inputs:
  - Existing `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` monolith (8k+ LOC).
- Outputs:
  - Dedicated WebSocket contract test module under `service_api_endpoint_tests/`.
  - Root service API endpoint test file no longer containing moved WebSocket test blocks.
  - Source-level split contract tests asserting module ownership boundaries.

## Boundaries/Non-goals
- Do not change runtime behavior or service API semantics.
- Do not change WebSocket endpoint contract payloads/reason codes.
- Do not add dependencies.

## Failure modes
- Coverage regressions if WebSocket assertions are dropped during extraction.
- Helper visibility errors break compile-time wiring.
- Moved tests become disconnected from existing shared setup helpers.

## Acceptance criteria (testable booleans)
- [x] WebSocket helper functions and WebSocket contract tests are moved into a dedicated module file.
- [x] `service_api_endpoint_tests.rs` no longer contains moved WebSocket test function blocks.
- [x] Split contract tests enforce ownership markers for root vs. WebSocket module files.
- [x] `kamn-node` service API endpoint tests remain green.

## Files to touch
- `specs/6375-extract-service-api-websocket-contract-tests.md`
- `crates/kamn-node/src/main_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs` (new)
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` (new)

## Error semantics
- No runtime error behavior changes.
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Add split contract tests that fail until WebSocket helpers/tests are extracted.
- Green/Refactor/Integration:
  - `cargo test -p kamn-node service_api_endpoint_tests_split_contract`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::websocket_contract_tests::integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::websocket_contract_tests::integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::websocket_contract_tests::regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers -- --exact`

## Phase 6 integration evidence
- Wiring:
  - Added `#[path = "service_api_endpoint_tests/websocket_contract_tests.rs"] mod websocket_contract_tests;` in `service_api_endpoint_tests.rs`.
  - Moved WebSocket helpers and nine WebSocket contract tests into the dedicated module.
  - Added split ownership contract module `service_api_endpoint_tests_split_contract.rs` and wired it via `main_tests.rs`.
- Executed:
  - `cargo test -p kamn-node service_api_endpoint_tests_split_contract`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::websocket_contract_tests::integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::websocket_contract_tests::integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event -- --exact`
  - `cargo test -p kamn-node main_tests::service_api_endpoint_tests::websocket_contract_tests::regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers -- --exact`

## Deviations
- Targeted test filter commands required fully-qualified module paths after extraction.
