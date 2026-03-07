# 6554 Map Live Task Escrow Routes

## Objective
Map the Rust live transport client task and escrow methods onto the existing Service API route surface so `LiveTransportKamnClient` supports create/accept/complete task flows and create/release escrow flows instead of returning `NotImplemented`.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-sdk/src/live.rs`
  - `crates/kamn-sdk/src/live/config.rs`
  - `crates/kamn-sdk/src/live/state.rs`
  - `crates/kamn-sdk/src/live/routes.rs`
  - `crates/kamn-sdk/src/live/task_escrow.rs`
  - `crates/kamn-sdk/src/live/agent.rs`
  - `crates/kamn-sdk/src/service_client_message_task_routes.rs`
  - `crates/kamn-sdk/src/service_client_content_routes.rs`
  - `crates/kamn-sdk/src/types.rs`
  - `crates/kamn-sdk/tests/live_transport_agent.rs`
  - `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
  - `crates/kamn-sdk/tests/support/live_transport_task_escrow.rs`
  - `crates/kamn-sdk/tests/support/live_transport_contract_server.rs`
  - `crates/kamn-sdk/tests/support/live_transport_http.rs`
- Outputs:
  - live transport task methods call the real service API routes
  - live transport escrow methods call the real service API routes
  - deterministic SDK alias maps preserve service task/escrow ids behind `TaskId` and `EscrowId`

## Boundaries/Non-goals
- Do not add a balance route to the service API.
- Do not implement artifact submission, receive, register, or search route parity in this issue.
- Do not redesign `TaskDefinition`, `EscrowConfig`, `TaskId`, or `EscrowId` public types.

## Failure modes
- Live task or escrow methods still return `NotImplemented`.
- A created service task/escrow id cannot be resolved later for accept/complete/release operations.
- Unknown live aliases silently fall back to fabricated route ids instead of failing closed.
- Payload translation emits malformed JSON or the wrong auth scope for task/escrow routes.
- `balance()` behavior changes even though there is still no service route.

## Acceptance criteria
- [x] `LiveTransportKamnClient::create_task()` posts a deterministic JSON payload to `/v1/tasks/create`, uses `tasks:write`, and returns a stable `TaskId` alias for the service task id.
- [x] `accept_task()` and `complete_task()` resolve the stored service task id and call `/v1/tasks/{id}/accept` and `/v1/tasks/{id}/complete` with the expected auth scopes.
- [x] `create_escrow()` posts a deterministic JSON payload to `/v1/escrow/fund`, uses `escrow:write`, and returns a stable `EscrowId` alias for the service escrow id.
- [x] `release_escrow()` resolves the stored service escrow id and calls `/v1/escrow/{id}/release` with `escrow:write`.
- [x] Unknown task and escrow aliases fail closed with typed `SdkError::NotFound` results.
- [x] `balance()` remains `SdkError::NotImplemented("live transport balance route is not available via service api")`.

## Files to touch
- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-sdk/src/live/config.rs`
- `crates/kamn-sdk/src/live/state.rs`
- `crates/kamn-sdk/src/live/routes.rs`
- `crates/kamn-sdk/src/live/task_escrow.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
- `crates/kamn-sdk/tests/support/live_transport_task_escrow.rs`
- `crates/kamn-sdk/tests/support/live_transport_contract_server.rs`
- `crates/kamn-sdk/tests/support/live_transport_http.rs`

## Error semantics
- Missing local live alias mappings return `SdkError::NotFound { entity: "task"|"escrow", id: <numeric id> }`.
- Service route failures continue to surface through existing `ServiceApiClient` / `SdkError::ServiceApiError` behavior.
- JSON/auth payload construction failures remain hard failures via existing typed SDK errors.
- Unsupported surfaces outside this slice keep their current `SdkError::NotImplemented` behavior.

## Test plan
1. Add red live-transport contract tests proving task and escrow methods execute against a real loopback contract server.
2. Add red tests for unknown task/escrow aliases failing closed as `SdkError::NotFound`.
3. Keep `balance()` covered as still unsupported.
4. Run:
   - `cargo test -p kamn-sdk -- --nocapture`
   - `cargo test -p kamn-sdk --test live_transport_agent -- --nocapture`
   - `cargo test -p kamn-sdk --test live_transport_task_escrow -- --nocapture`
   - `cargo test -p kamn-sdk --test service_api_client -- --nocapture`
   - `rustfmt crates/kamn-sdk/src/live.rs crates/kamn-sdk/src/live/agent.rs crates/kamn-sdk/src/live/config.rs crates/kamn-sdk/src/live/state.rs crates/kamn-sdk/src/live/task_escrow.rs crates/kamn-sdk/src/live/routes.rs crates/kamn-sdk/tests/live_transport_task_escrow.rs crates/kamn-sdk/tests/support/live_transport_task_escrow.rs crates/kamn-sdk/tests/support/live_transport_contract_server.rs crates/kamn-sdk/tests/support/live_transport_http.rs crates/kamn-sdk/tests/live_transport_agent.rs`
   - `cargo fmt --all --check`
   - `cargo clippy -p kamn-sdk --tests -- -D warnings`

## Integration evidence
- `LiveTransportKamnClient` now uses the real `ServiceApiClient` route surface for task create/accept/complete and escrow fund/release instead of returning `NotImplemented`.
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs` exercises the full route set against a loopback contract server that verifies path, JSON body, sender DID, auth scope, nonce, and request signature.
- The alias map remains local-only state inside the live transport client and fails closed on unknown numeric aliases with typed `SdkError::NotFound`.

## Deviations
- The implementation split the live transport work across `crates/kamn-sdk/src/live/` submodules instead of a single helper file so each touched file stayed within the repo file-size budget.
- `cargo fmt --all --check` still fails on pre-existing formatting drift in `crates/kamn-core/tests/ci_strategy_docs.rs`; this issue formatted the touched SDK files directly and left the unrelated drift untouched.
