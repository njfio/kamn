# Plan: Issue #6043

## Approach
1. Introduce explicit heartbeat interval and stale timeout constants in websocket stream module.
2. Extend `stream_websocket_event` loop with a periodic tick branch that sends server ping.
3. Track last inbound activity timestamp and close the connection when stale timeout is exceeded.
4. Add focused async tests for ping emission and stale timeout behavior.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`

## Risks / Mitigations
- Risk: time-driven async tests can become flaky.
  Mitigation: keep deterministic durations small in test helper, assert bounded outcomes.
- Risk: keepalive changes may interfere with existing event flow.
  Mitigation: preserve existing message/event handling branches and add compatibility test.

## Interfaces / Contracts
- No public API/wire payload changes.
- Runtime behavior contract: websocket lifecycle gains server ping + stale-timeout enforcement.
