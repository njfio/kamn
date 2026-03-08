# Objective
Add high-level artifact expire to the SDK `KamnAgent` API so callers can advance submitted artifacts into the existing expired lifecycle state through the same top-level workflow surface in both in-memory and live transports.

# Inputs/Outputs
- Input: `ArtifactId` from a previously submitted artifact.
- Output: updated `ArtifactStatus` with lifecycle state `expired` and redaction status `none`.
- Input: live transport artifact expire request.
- Output: service-backed `ArtifactStatus` derived from existing `POST /v1/content/{id}/expire`.

# Boundaries/Non-goals
- Do not add tombstone support in this issue.
- Do not add artifact byte retrieval or artifact listing APIs.
- Do not change existing low-level service route contracts.
- Do not add new persistence formats beyond what is required to track in-memory artifact lifecycle state.

# Failure modes
- Unknown artifact identifiers must fail closed as not found.
- Live transport must fail closed when the artifact alias map does not contain the requested `ArtifactId`.
- Empty or malformed live expire response payloads must fail closed as transport failures.
- Repeated in-memory expire calls must remain deterministic and return expired status without corrupting state.

# Acceptance criteria
- [ ] `KamnAgent` exposes an artifact expire method returning `ArtifactStatus`.
- [ ] `InMemoryKamnClient` expires known artifacts, returns expired status, and rejects unknown artifacts.
- [ ] `LiveTransportKamnClient` resolves `ArtifactId` through the existing alias map and uses `POST /v1/content/{id}/expire`.
- [ ] SDK tests cover in-memory and live high-level artifact expire, including unknown-alias and malformed-response failures.
- [ ] Existing lower-level `ServiceApiClient::expire_content()` behavior remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/task_escrow/aliases.rs` only if shared artifact alias lookup needs reuse
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
- `crates/kamn-sdk/tests/service_api_client.rs` only if low-level route coverage needs extension
- `specs/6598-add-sdk-artifact-expire.md`

# Error semantics
- Return `SdkError::NotFound { entity: "artifact", ... }` for unknown artifact aliases/records.
- Return `SdkError::TransportFailure` when the live expire response omits required lifecycle fields or returns malformed content identifiers.
- Preserve existing `SdkError::ServiceApiError` mappings for live auth or HTTP failures.
- Do not fabricate fallback lifecycle states when the live route fails.

# Test plan
- Add red tests for in-memory artifact expire success and unknown-artifact failure.
- Add red tests for live artifact expire using the real service route contract server.
- Add a live fail-closed test for malformed expire response payloads.
- Re-run existing signed low-level `ServiceApiClient` route contract coverage.
- Run strict `clippy` for `kamn-sdk` and targeted SDK tests.
