# Issue #5283 Plan

## Objective
Close the remaining Phase-5 validation/documentation gap by adding bounded-load realtime guardrail coverage and documenting websocket presence-mode operations.

## Affected Modules
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `docs/ops/configuration.md`

## Approach
1. Add failing tests first for bounded-load backpressure, anti-spam rejection, and ordering/duplicate suppression behavior in the websocket realtime slice.
2. Implement minimal route/runtime changes needed to satisfy deterministic decision handling and stable reason markers.
3. Update operator documentation with presence-mode header requirements, owner-scope constraints, and fail-closed troubleshooting map.
4. Run scoped verification (`fmt`, strict `clippy`, targeted tests), then prep PR evidence.

## Risks and Mitigations
- Risk: test flakiness from realtime timing assumptions.
  - Mitigation: use deterministic fixtures and bounded synthetic inputs instead of wall-clock dependent waits.
- Risk: over-coupling docs assertions to exact prose.
  - Mitigation: assert required markers and config snippets, not broad paragraph text.
- Risk: regressions in default websocket behavior.
  - Mitigation: retain existing baseline integration tests and run them in the same targeted suite.

## Interfaces and Contracts
- Preserve existing `/v1/events/ws` behavior for default mode requests.
- Presence-mode handling remains owner-scoped and fail-closed on invalid headers/scope.
- Keep reason markers stable for anti-spam/backpressure denial paths used by operator troubleshooting.

## ADR
- No new ADR required; this task extends existing Phase-5 integration contracts without introducing new architecture or dependencies.
