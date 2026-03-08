# Objective
Add high-level artifact tombstone to the SDK `KamnAgent` API so callers can advance submitted artifacts into the existing tombstoned lifecycle state through the same top-level workflow surface in both in-memory and live transports.

# Inputs/Outputs
- Input: `ArtifactId` from a previously submitted artifact.
- Output: updated `ArtifactStatus` with lifecycle state `tombstoned` and redaction status `redacted`.
- Input: live transport artifact tombstone request.
- Output: service-backed `ArtifactStatus` derived from existing `POST /v1/content/{id}/tombstone`.

# Boundaries/Non-goals
- Do not add new content routes or change existing service contracts.
- Do not add artifact byte retrieval or artifact listing APIs.
- Do not change existing artifact expire behavior.
- Do not add persistence formats beyond what is required to track in-memory tombstoned lifecycle state.

# Failure modes
- Unknown artifact identifiers must fail closed as not found.
- Live transport must fail closed when the artifact alias map does not contain the requested `ArtifactId`.
- Empty or malformed live tombstone response payloads must fail closed as transport failures.
- Repeated in-memory tombstone calls must remain deterministic and return tombstoned status without corrupting state.

# Acceptance criteria
- [x] `KamnAgent` exposes an artifact tombstone method returning `ArtifactStatus`.
- [x] `InMemoryKamnClient` tombstones known artifacts, returns tombstoned status, and rejects unknown artifacts.
- [x] `LiveTransportKamnClient` resolves `ArtifactId` through the existing alias map and uses `POST /v1/content/{id}/tombstone`.
- [x] SDK tests cover in-memory and live high-level artifact tombstone, including unknown-alias and malformed-response failures.
- [x] Existing lower-level `ServiceApiClient::tombstone_content()` behavior remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
- `crates/kamn-sdk/tests/service_api_client.rs` only if low-level route coverage needs extension
- `specs/6600-add-sdk-artifact-tombstone.md`

# Error semantics
- Return `SdkError::NotFound { entity: "artifact", ... }` for unknown artifact aliases/records.
- Return `SdkError::TransportFailure` when the live tombstone response omits required lifecycle fields or returns malformed content identifiers.
- Preserve existing `SdkError::ServiceApiError` mappings for live auth or HTTP failures.
- Do not fabricate fallback lifecycle states when the live route fails.

# Test plan
- Add red tests for in-memory artifact tombstone success and unknown-artifact failure.
- Add red tests for live artifact tombstone using the real service route contract server.
- Add a live fail-closed test for malformed tombstone response payloads.
- Re-run existing signed low-level `ServiceApiClient` route contract coverage.
- Run strict `clippy` for `kamn-sdk` and targeted SDK tests.

# Deviations
- None.

# Verification
- `cargo test -p kamn-sdk --test memory_agent tombstone_artifact -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_task_escrow tombstone -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_task_escrow spec_c06_live_transport_task_and_escrow_routes_execute_network_contract -- --exact --nocapture`
- `cargo test -p kamn-sdk --test service_api_client functional_service_api_client_executes_signed_http_route_contracts -- --exact --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
