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
- [x] Service API adds `POST /v1/agents/register` requiring `agents:write` scope.
- [x] `ServiceApiScope` and route auth mapping support canonical `agents:write`.
- [x] Agent records persist `agent_type`, `model_family`, and `capabilities`.
- [x] `GET /v1/agents/{did}` returns persisted metadata fields in addition to `did` and `reputation_score`.
- [x] `ServiceApiClient` adds a typed registration route.
- [x] `LiveTransportKamnClient::register()` uses the real route and returns the sender DID.
- [x] `LiveTransportKamnClient::resolve()` returns stored metadata rather than synthetic defaults.
- [x] Duplicate identical registration is stable; duplicate mismatched registration fails closed.

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

# Phase 6 Evidence
- `POST /v1/agents/register` is handled in the real node service API path and persists sender DID metadata through `ServiceApiMessageStore`.
- `GET /v1/agents/{did}` now returns persisted metadata for registered agents and preserves the existing synthetic defaults for unregistered agents.
- `LiveTransportKamnClient::register()` signs and emits the real registration route and `resolve()` consumes the stored metadata through the typed service client.

# Verification
- `rustfmt --edition 2024 --check crates/kamn-kolme/src/service_api_scope.rs crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs crates/kamn-node/src/service_api_endpoint.rs crates/kamn-node/src/service_api_endpoint/auth.rs crates/kamn-node/src/service_api_endpoint/message_store.rs crates/kamn-node/src/service_api_endpoint/middleware_impl.rs crates/kamn-node/src/service_api_endpoint/models.rs crates/kamn-node/src/service_api_endpoint/payload.rs crates/kamn-sdk/src/live/agent.rs crates/kamn-sdk/src/live/agent_mutations.rs crates/kamn-sdk/src/live/config.rs crates/kamn-sdk/src/live/routes.rs crates/kamn-sdk/src/service_client_bridge_misc_routes.rs crates/kamn-sdk/src/service_models.rs crates/kamn-sdk/tests/live_transport_agent.rs crates/kamn-sdk/tests/service_api_client.rs`
- `cargo clippy -p kamn-node --tests -- -D warnings`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
- `cargo test -p kamn-node --bin kamn-node service_api_endpoint::auth::tests::regression_required_scope_for_route_maps_agent_registration_to_agents_write -- --exact --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_endpoint_serde_payload_roundtrip_contracts -- --exact --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::unit_service_api_route_authz_matrix_matches_protected_and_public_paths -- --exact --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping -- --exact --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch -- --exact --nocapture`
- `cargo test -p kamn-sdk --test service_api_client regression_service_api_client_registration_surface_contract_exists -- --exact --nocapture`
- `cargo test -p kamn-sdk --test service_api_client functional_service_api_client_executes_signed_http_route_contracts -- --exact --nocapture`
- `cargo test -p kamn-sdk --test live_transport_agent -- --nocapture`

# Deviations
- `cargo fmt --all --check` was not used as the final formatting gate because `main` already contains unrelated formatting drift outside `#6590`. The touched Rust files were instead formatted and checked explicitly with `rustfmt --edition 2024 --check`.
