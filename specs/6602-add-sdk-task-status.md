# Objective
Add high-level task status to the SDK `KamnAgent` API so callers can read task lifecycle through the same top-level workflow surface in both in-memory and live transports.

# Inputs/Outputs
- Input: `TaskId` from a previously created task.
- Output: SDK `TaskStatus` containing the task identifier and lifecycle state.
- Input: live transport task-status lookup.
- Output: service-backed `TaskStatus` derived from existing `GET /v1/tasks/{id}`.

# Boundaries/Non-goals
- Do not add new task routes or change existing service contracts.
- Do not add task listing APIs.
- Do not change create/accept/complete task behavior.
- Do not add persistence formats beyond what is required to expose in-memory task lifecycle state.

# Failure modes
- Unknown task identifiers must fail closed as not found.
- Live transport must fail closed when the task alias map does not contain the requested `TaskId`.
- Empty or malformed live task-status response payloads must fail closed as transport failures.
- In-memory task status must remain deterministic across submitted, accepted, and completed transitions.

# Acceptance criteria
- [x] `KamnAgent` exposes a task status method returning a stable SDK `TaskStatus` type.
- [x] `InMemoryKamnClient` reports submitted, accepted, and completed task states for known tasks and rejects unknown tasks.
- [x] `LiveTransportKamnClient` resolves `TaskId` through the existing alias map and uses `GET /v1/tasks/{id}`.
- [x] SDK tests cover in-memory and live high-level task status, including unknown-alias and malformed-response failures.
- [x] Existing lower-level `ServiceApiClient::get_task()` behavior remains green.

# Files to touch
- `crates/kamn-sdk/src/agent.rs`
- `crates/kamn-sdk/src/types.rs`
- `crates/kamn-sdk/src/lib.rs` only if the new type needs re-export
- `crates/kamn-sdk/src/memory.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
- `specs/6602-add-sdk-task-status.md`

# Error semantics
- Return `SdkError::NotFound { entity: "task", ... }` for unknown task aliases/records.
- Return `SdkError::TransportFailure` when the live task-status response omits required fields or returns malformed task identifiers.
- Preserve existing `SdkError::ServiceApiError` mappings for live auth or HTTP failures.
- Do not fabricate fallback task states when the live route fails.

# Test plan
- Add red tests for in-memory task-status transitions and unknown-task failure.
- Add red tests for live task status using the real service route contract server.
- Add a live fail-closed test for malformed task-status response payloads.
- Re-run existing signed low-level `ServiceApiClient` route contract coverage.
- Run strict `clippy` for `kamn-sdk` and targeted SDK tests.

# Deviations
- None.

# Verification
- `cargo test -p kamn-sdk --test memory_agent get_task_status -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_task_escrow task_status -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_task_escrow spec_c06_live_transport_task_and_escrow_routes_execute_network_contract -- --exact --nocapture`
- `cargo test -p kamn-sdk --test service_api_client functional_service_api_client_executes_signed_http_route_contracts -- --exact --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
