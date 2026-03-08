# Objective
Add a service-backed agent registration route so `LiveTransportKamnClient::register()` can use a real authenticated HTTP path and `resolve()` can return persisted agent metadata from the service.

# Inputs/Outputs
- Input: `POST /v1/agents/register` with authenticated sender DID and JSON body containing agent metadata.
- Output: persisted agent profile record and SDK `AgentDid` for the sender.
- Input: `GET /v1/agents/{did}`
- Output: persisted metadata fields plus existing profile fields.

# Boundaries/Non-goals
- Do not implement `search_agents()` in this issue.
- Do not change in-memory client behavior.
- Do not add artifact or balance semantics to registration beyond existing default balance initialization.

# Failure modes
- Empty or invalid `agent_type`, `model_family`, or capability entries must fail closed.
- Sender DID in the auth envelope must match the DID persisted for registration.
- Duplicate registration for an existing DID must be idempotent for identical metadata and conflict on mismatched metadata.
- Missing or unknown `agents:write` scope must fail closed at auth boundary.

# Acceptance criteria
- [ ] Service API adds `POST /v1/agents/register` requiring `agents:write` scope.
- [ ] `ServiceApiScope` and route auth mapping support canonical `agents:write`.
- [ ] Agent records persist `agent_type`, `model_family`, and `capabilities`.
- [ ] `GET /v1/agents/{did}` returns persisted metadata fields in addition to `did` and `reputation_score`.
- [ ] `ServiceApiClient` adds a typed registration route.
- [ ] `LiveTransportKamnClient::register()` uses the real route and returns the sender DID.
- [ ] `LiveTransportKamnClient::resolve()` returns stored metadata rather than synthetic defaults.
- [ ] Duplicate identical registration is stable; duplicate mismatched registration fails closed.

# Files to touch
- `crates/kamn-kolme/src/service_api_scope.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-sdk/src/service_models.rs`
- `crates/kamn-sdk/src/service_client_bridge_misc_routes.rs`
- `crates/kamn-sdk/src/live/config.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/agent_mutations.rs`
- `crates/kamn-sdk/src/live/routes.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `docs/api/service-openapi.yaml`
- `docs/api/service-http-api.md`
- `specs/6590-add-service-backed-live-agent-registration.md`
- `fixtures/ci/test_file_size_policy_baseline.env` only if inventory drift occurs

# Error semantics
- Use existing service API unauthorized responses for scope/auth failures.
- Use `400 Bad Request` with deterministic reason code for invalid registration payload fields.
- Use `409 Conflict` with deterministic reason code for mismatched duplicate registration.
- Preserve existing `SdkError::InvalidInput`, `SdkError::Conflict`, and `SdkError::ServiceApiError` mappings.

# Test plan
- Add node integration coverage for successful registration, idempotent duplicate registration, and mismatched duplicate rejection.
- Add auth/scope regression coverage for `agents:write` routing.
- Add service client contract coverage for `POST /v1/agents/register`.
- Add live transport contract coverage for `register()` and `resolve()` round-trip with persisted metadata.
- Run targeted node and sdk suites plus strict clippy.
