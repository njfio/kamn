# Objective
Add a service-backed agent search route so `LiveTransportKamnClient::search_agents()` can use real persisted registration metadata instead of returning `NotImplemented`.

# Inputs/Outputs
- Input: `POST /v1/agents/search` with authenticated `agents:read` scope and JSON body containing optional `model_family` and optional `capability` filters.
- Output: deterministic JSON array of registered agent summaries containing `did`, `agent_type`, `model_family`, and `capabilities`.
- Input: `AgentQuery` from SDK live transport.
- Output: `Vec<AgentSummary>` matching current in-memory semantics.

# Boundaries/Non-goals
- Do not add fuzzy search, text search, pagination, or sorting options.
- Do not change in-memory `search_agents()` semantics.
- Do not return synthetic default profiles for unregistered DIDs.
- Do not alter registration or profile fetch behavior beyond what search needs.

# Failure modes
- Invalid JSON body must fail closed with deterministic bad-request semantics.
- Empty or whitespace-only `model_family` filters must fail closed.
- Empty or whitespace-only `capability` filters must fail closed.
- Missing or invalid `agents:read` auth scope must fail closed at the auth boundary.
- Persistence/query failures in the service store must fail closed as internal errors.

# Acceptance criteria
- [ ] Service API adds `POST /v1/agents/search` requiring `agents:read` scope.
- [ ] Service message store exposes a deterministic search over persisted registered agent metadata.
- [ ] Search results are sorted by DID and match the current in-memory filter behavior for optional `model_family` and `capability`.
- [ ] Search excludes synthetic default profiles for unregistered DIDs.
- [ ] `ServiceApiClient` exposes a typed agent-search route returning `Vec<ServiceAgentProfile>` or equivalent typed summaries.
- [ ] `LiveTransportKamnClient::search_agents()` uses the real route and returns `Vec<AgentSummary>`.
- [ ] Existing live unsupported-method coverage stops expecting `search_agents()` to be unsupported.
- [ ] Node integration, SDK service-client, and live transport integration tests cover the real path.

# Files to touch
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/models.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/service_client_bridge_misc_routes.rs`
- `crates/kamn-sdk/src/service_models.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `docs/api/service-openapi.yaml`
- `docs/api/service-http-api.md`
- `specs/6592-add-service-backed-live-agent-search.md`
- `fixtures/ci/test_file_size_policy_baseline.env` only if inventory drift occurs

# Error semantics
- Use existing service API unauthorized responses for auth/scope failures.
- Use `400 Bad Request` with deterministic reason code for invalid search payload fields.
- Use `500 Internal Server Error` with existing persistence reason semantics for store failures.
- Preserve existing `SdkError::InvalidInput`, `SdkError::ServiceApiError`, and transport failure mappings.

# Test plan
- Add node integration coverage for unfiltered search, filtered-by-capability search, filtered-by-model search, and invalid filter rejection.
- Add auth/scope regression coverage for `POST /v1/agents/search -> agents:read`.
- Add service-client contract coverage for the typed search route.
- Add live transport coverage for `search_agents()` using the real route and remove it from unsupported-method expectations.
- Run targeted node and SDK suites plus strict clippy.
