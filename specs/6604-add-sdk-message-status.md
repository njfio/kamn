# Objective
Add high-level message status to the SDK `KamnAgent` API so callers can read message lifecycle through the same top-level workflow surface in both in-memory and live transports.

# Inputs/Outputs
- Input: `MessageId` from a previously sent message.
- Output: SDK `MessageStatus` containing the message identifier and lifecycle status.
- Input: live transport message-status lookup.
- Output: service-backed `MessageStatus` derived from existing `GET /v1/messages/{id}`.

# Boundaries/Non-goals
- Do not add new message routes or change existing service contracts.
- Do not add message listing APIs.
- Do not change send or receive behavior.
- Do not add persistence formats beyond what is required to expose in-memory message lifecycle state.

# Failure modes
- Unknown message identifiers must fail closed as not found.
- Live transport must fail closed when the message alias map does not contain the requested `MessageId`.
- Empty or malformed live message-status response payloads must fail closed as transport failures.
- In-memory message status must remain deterministic for known sent messages even after mailbox drains.

# Acceptance criteria
- [ ] `KamnAgent` exposes a message status method returning a stable SDK `MessageStatus` type.
- [ ] `InMemoryKamnClient` reports status for known sent messages and rejects unknown message ids.
- [ ] `LiveTransportKamnClient` resolves `MessageId` through the existing alias map and uses `GET /v1/messages/{id}`.
- [ ] SDK tests cover in-memory and live high-level message status, including unknown-alias and malformed-response failures.
- [ ] Existing lower-level `ServiceApiClient::get_message()` behavior remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/types.rs`
- `crates/kamn-sdk/src/lib.rs` only if the new type needs re-export
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs` or `crates/kamn-sdk/tests/live_transport_receive.rs` depending on the real-path fit
- `specs/6604-add-sdk-message-status.md`

# Error semantics
- Return `SdkError::NotFound { entity: "message", ... }` for unknown message aliases/records.
- Return `SdkError::TransportFailure` when the live message-status response omits required fields or returns malformed message identifiers.
- Preserve existing `SdkError::ServiceApiError` mappings for live auth or HTTP failures.
- Do not fabricate fallback message statuses when the live route fails.

# Test plan
- Add red tests for in-memory known/unknown message status.
- Add red tests for live message status using the real service route contract server.
- Add a live fail-closed test for malformed message-status response payloads.
- Re-run existing signed low-level `ServiceApiClient` route contract coverage.
- Run strict `clippy` for `kamn-sdk` and targeted SDK tests.
