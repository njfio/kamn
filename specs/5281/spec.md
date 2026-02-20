# Issue #5281 Spec

- Title: Task: integrate service-api websocket presence route with M9 gateway bridge contracts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`kamn-core` now exposes deterministic M9 gateway bridge contracts, but `kamn-node` websocket runtime still emits only static state-transition events. The websocket route must integrate bridge-derived presence projections with deterministic fail-closed scope handling.

## Scope
In:
- Add websocket presence-mode projection path in service API route handling.
- Consume `data_layer_m9_gateway_project_presence_event` for deterministic owner-scoped payload emission.
- Add fail-closed handling for:
  - unsupported websocket event mode,
  - missing/invalid presence headers,
  - cross-owner and visibility denial branches from M9.
- Add integration/regression tests for success and fail-closed behavior.

Out:
- Multi-node websocket fan-out and session replication.
- External push integrations.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 420
- shell_to_rust_ratio_delta_estimate: -0.0020
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: `/v1/events/ws` keeps existing state-transition behavior when no presence-mode header is requested.
- AC-2: Presence-mode websocket requests emit deterministic bridge-derived presence payloads for valid owner-scoped requests.
- AC-3: Unsupported mode, malformed presence headers, and scope/visibility denials fail closed with stable reason markers.
- AC-4: Unit/Functional/Integration/Regression coverage for this slice passes with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | websocket upgrade request without presence-mode header | existing state-transition payload shape remains stable |
| C-02 | AC-2 | Integration | websocket upgrade with `x-kamn-events-mode: presence` and valid owner/agent headers | deterministic presence payload with bridge-derived fields and reason marker |
| C-03 | AC-3 | Regression | websocket upgrade with unsupported events mode | fail-closed error response with stable unsupported-mode reason code |
| C-04 | AC-3 | Regression | websocket presence mode with missing required presence owner header | fail-closed error response with stable missing-header reason code |
| C-05 | AC-3 | Regression | websocket presence mode with cross-owner query headers | fail-closed error response preserving `DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE` |
| C-06 | AC-4 | Verification | fmt/clippy + targeted websocket endpoint tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-node -- main_tests::service_api_endpoint_tests::integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event`
- `cargo test -p kamn-node -- main_tests::service_api_endpoint_tests::integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event`
- `cargo test -p kamn-node -- main_tests::service_api_endpoint_tests::regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode`
- `cargo test -p kamn-node -- main_tests::service_api_endpoint_tests::regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Story `#5252` progresses from bridge-only contracts to node runtime consumption for websocket presence integration.
- No shell-surface growth while increasing runtime integration coverage.
