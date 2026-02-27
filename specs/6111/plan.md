# Plan: Issue #6111

## Approach
1. Add RED unit tests for frame payload extraction covering inline, 16-bit, 64-bit, and truncation/overflow errors.
2. Extract websocket frame payload parsing into a helper function and implement RFC 6455 length-marker decoding.
3. Keep fail-closed guards for opcode, masking, truncation, and overflow.
4. Add/extend integration harness websocket response helper to emit extended payload frames (>125 bytes) and assert SDK decoding.
5. Run targeted `kamn-sdk` tests plus fmt/clippy gates.

## Affected Modules
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `specs/6111/spec.md`
- `specs/6111/plan.md`
- `specs/6111/tasks.md`

## Risks / Mitigations
- Risk: incorrect length parsing introduces silent truncation.
  Mitigation: explicit bounds checks for each marker and deterministic errors for short/overflow frames.
- Risk: integration harness still emits only short frames.
  Mitigation: add explicit extended-frame fixture path and assertion.

## Interfaces / Contracts
- `ServiceApiClient::read_event_once` keeps the same public API and error type.
- Internal wire contract now accepts server frames encoded with RFC 6455 payload length markers `126` and `127`.
