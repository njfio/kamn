# Objective
Add a high-level bridge lifecycle API to the SDK so callers can submit, forward, and query bridges through the same top-level workflow surface in both in-memory and live transports.

# Inputs/Outputs
- Input: source `MessageId` and target network string for bridge submission.
- Output: stable SDK `BridgeId` for the created bridge.
- Input: `BridgeId` for bridge forwarding and status lookup.
- Output: SDK `BridgeStatus` containing the bridge identifier, lifecycle status, optional forwarded target message id, and optional forward transaction hash.
- Input: live transport bridge lifecycle operations.
- Output: bridge lifecycle results derived from the existing `/v1/bridge/*` route family.

# Boundaries/Non-goals
- Do not add or modify service routes.
- Do not add bridge listing APIs.
- Do not change existing low-level bridge payload shapes.
- Do not alter high-level message send/receive semantics.

# Failure modes
- Unknown source message ids must fail closed for bridge submission.
- Unknown bridge ids must fail closed for bridge forward and status lookup.
- Empty or malformed live bridge response payloads must fail closed as transport failures.
- In-memory bridge lifecycle state must remain deterministic across submit, forward, and later query operations.

# Acceptance criteria
- [ ] `KamnAgent` exposes high-level bridge submit, forward, and status methods with stable SDK bridge types.
- [ ] `InMemoryKamnClient` supports deterministic bridge lifecycle transitions and rejects unknown message/bridge ids.
- [ ] `LiveTransportKamnClient` maps stable SDK bridge ids onto the existing `submit`, `forward`, and `query` bridge routes.
- [ ] SDK tests cover in-memory and live bridge lifecycle behavior, including unknown-id and malformed-response failures.
- [ ] Existing lower-level `ServiceApiClient` bridge route coverage remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/types.rs`
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/state.rs`
- `crates/kamn-sdk/src/live/task_escrow/aliases.rs` or a split bridge alias helper seam
- dedicated bridge-focused tests under `crates/kamn-sdk/tests/`
- `specs/6606-add-sdk-bridge-lifecycle.md`

# Error semantics
- Return `SdkError::NotFound { entity: "message", ... }` when bridge submission references an unknown message id.
- Return `SdkError::NotFound { entity: "bridge", ... }` when bridge forward/status lookup references an unknown bridge id.
- Return `SdkError::InvalidInput` when bridge target network is empty.
- Return `SdkError::TransportFailure` when live bridge responses omit required fields or return malformed message/bridge identifiers.
- Preserve existing `SdkError::ServiceApiError` mappings for live auth or HTTP failures.

# Test plan
- Add red tests for in-memory bridge submit, forward, and query behavior.
- Add red tests for live bridge lifecycle through the real service route contract server.
- Add fail-closed tests for unknown bridge ids and malformed live bridge responses.
- Re-run existing signed low-level `ServiceApiClient` bridge route contract coverage.
- Run strict `clippy` for `kamn-sdk` and the full `kamn-sdk` test suite.
