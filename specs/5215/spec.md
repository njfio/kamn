# Issue #5215 Spec

- Title: Task: Extract service_api_endpoint auth, router, and websocket logic into submodules
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
`crates/kamn-node/src/service_api_endpoint.rs` (1,813 lines) is a monolithic coordinator that mixes middleware/auth validation, route dispatch, websocket upgrade behavior, and payload/render helpers. R43 identifies this as the top remaining structural maintainability concern.

## Scope
In:
- Decompose `service_api_endpoint.rs` into focused submodules under `crates/kamn-node/src/service_api_endpoint/`.
- Preserve existing route/auth/websocket behavior and reason-code contracts.
- Keep root file as a coordinator with explicit `mod` boundaries and required re-exports.

Out:
- New HTTP/WebSocket routes or payload schema changes.
- Protocol/wire-format changes.
- Shell/Python/workflow changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 220
- shell_to_rust_ratio_delta_estimate: -0.0018
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: `service_api_endpoint.rs` root is structurally decomposed into focused submodules (auth/middleware, routing, websocket, and payload/render helpers) and root line count is reduced below 1,000 lines.
- AC-2: Existing route dispatch and request-auth contracts remain behaviorally identical (including reason-code projection for rejection paths).
- AC-3: Existing websocket upgrade/header validation and stream behavior remain unchanged.
- AC-4: Existing service API unit/functional/integration/regression suites pass without introducing duplicate dead code between root and submodules.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Compile and inspect module layout after extraction | Root file delegates to submodules and remains below line budget |
| C-02 | AC-2 | Unit | Auth header/nonce/signature parsing and projection tests | Same reason codes and failure mapping as pre-refactor |
| C-03 | AC-2 | Functional | Route render tests for send/read/channel/task/agent/health/metrics | Response status/body/reason markers remain stable |
| C-04 | AC-3 | Regression | Websocket upgrade negative tests (missing/invalid headers) | Same rejection reason codes and status responses |
| C-05 | AC-3 | Integration | Live websocket upgrade/stream test path | Upgrade and state-transition streaming remain stable |
| C-06 | AC-4 | Integration/Regression | Service API endpoint suite | All mapped service-api tests pass; no compilation duplicates |

## Test Mapping
- C-01 -> `wc -l crates/kamn-node/src/service_api_endpoint.rs` + module declaration checks
- C-02 -> `cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts`
- C-03 -> `cargo test -p kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts`
- C-04 -> `cargo test -p kamn-node main_tests::service_api_endpoint_tests::regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers`
- C-05 -> `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event`
- C-06 -> `cargo test -p kamn-node main_tests::service_api_endpoint_tests`

## Success Metrics
- Root file drops below 1,000 lines.
- Service API endpoint test suite remains green.
- No shell LOC increase from this task.
