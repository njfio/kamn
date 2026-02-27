# Plan: Issue 6190 - SDK WebSocket Extended-Length Frame Support

- Issue: #6190
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extract frame parsing logic from `read_event_once` into a small helper that:
   - validates opcode and mask bit,
   - parses payload length for 7-bit/16-bit/64-bit modes,
   - validates truncation bounds,
   - returns payload slice.
2. Keep current error markers for unsupported opcode/mask/truncation.
3. Add RED unit tests for 126/127 frames and malformed truncation paths.
4. Verify existing integration test `integration_service_api_client_reads_websocket_event_frame`.

## Affected Modules

- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/tests/service_api_client.rs` (regression verification only if needed)

## Risks and Mitigations

1. Integer conversion overflow for 64-bit length:
   - Mitigation: checked `usize::try_from` and fail closed on overflow.
2. Behavior drift in current short-frame handling:
   - Mitigation: preserve opcode/mask checks and route integration tests.

## Contracts / Interfaces

No public API shape changes.
Internal parser becomes RFC6455-compliant for server unmasked text frame length encoding.
