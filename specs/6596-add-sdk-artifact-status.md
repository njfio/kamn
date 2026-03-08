# Objective
Add high-level artifact status retrieval to the SDK `KamnAgent` API so callers can query submitted artifact lifecycle state through the same top-level workflow surface in both in-memory and live transports.

# Inputs/Outputs
- Input: `ArtifactId` from a previously submitted artifact.
- Output: SDK artifact status metadata with artifact identifier, lifecycle state, and redaction status.
- Input: live transport artifact status query.
- Output: service-backed artifact status derived from existing `GET /v1/content/{id}`.

# Boundaries/Non-goals
- Do not add artifact byte retrieval; the current live service contract does not expose bytes.
- Do not add artifact listing or search APIs.
- Do not change existing content registration, expire, or tombstone route contracts.
- Do not add new persistence formats beyond mapping existing in-memory and live content status surfaces.

# Failure modes
- Unknown artifact identifiers must fail closed as not found.
- Live transport must fail closed when the artifact alias map does not contain the requested `ArtifactId`.
- Empty or malformed live `content_id` / lifecycle payload fields must fail closed as transport failures.
- In-memory artifact status must remain deterministic for repeated reads.

# Acceptance criteria
- [ ] `KamnAgent` exposes an artifact status retrieval method.
- [ ] The SDK exposes a dedicated artifact status type.
- [ ] `InMemoryKamnClient` returns retained/non-redacted status for known artifacts and `NotFound` for unknown ones.
- [ ] `LiveTransportKamnClient` resolves `ArtifactId` through the existing alias map and uses `GET /v1/content/{id}`.
- [ ] SDK tests cover both in-memory and live high-level artifact status retrieval, including fail-closed alias/status errors.
- [ ] Existing lower-level `ServiceApiClient::get_content()` behavior remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/types.rs`
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/state.rs` only if alias/state helpers need extraction
- `crates/kamn-sdk/src/live/task_escrow/aliases.rs` if artifact alias lookup helpers need extraction
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs` or `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/service_api_client.rs` only if low-level route harness coverage needs extension
- `specs/6596-add-sdk-artifact-status.md`

# Error semantics
- Return `SdkError::NotFound { entity: "artifact", ... }` for unknown artifact aliases/records.
- Return `SdkError::TransportFailure` when the live service response omits required lifecycle fields or returns empty content identifiers.
- Preserve existing `SdkError::ServiceApiError` mappings for live auth or HTTP failures.
- Do not fabricate fallback lifecycle states when the live route fails.

# Test plan
- Add red tests for in-memory artifact status success and unknown-artifact failure.
- Add red tests for live artifact status retrieval using the real service route contract server.
- Add a live fail-closed test for missing artifact alias or malformed service status payload.
- Re-run existing `ServiceApiClient::get_content()` contract coverage.
- Run strict `clippy` for `kamn-sdk` and targeted SDK tests.
