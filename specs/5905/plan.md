# Plan: Issue #5905 - Persistent Service API WebSocket Streaming with Live Event Delivery

## Approach
1. Introduce runtime websocket fan-out state in service API runtime (bounded broadcast queue).
2. Publish message lifecycle events from HTTP route handlers into the websocket fan-out.
3. Replace one-shot websocket close path with a long-lived loop that forwards initial snapshot event plus live fan-out events.
4. Preserve current websocket validation and error mappings.
5. Add integration/regression tests for persistent stream and live follow-up event delivery.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: fan-out channel overflow may drop events under burst load.
  - Mitigation: use explicit bounded capacity and emit deterministic overflow markers for tests.
- Risk: long-lived tasks may leak on client disconnect.
  - Mitigation: terminate streaming loop on socket send/read error and rely on task completion.
- Risk: behavior drift for existing websocket validation contracts.
  - Mitigation: keep validation flow unchanged and extend only post-upgrade stream handling.

## Interfaces / Contracts
- No route or header contract removals.
- Websocket frame contract is additive: stream includes initial event plus follow-up lifecycle events.

## ADR
- Not required (no new dependencies or external protocol changes).
