# Spec: Issue #6562 - Add service-backed live balance route for SDK parity

## Objective

Expose a read-only service API balance route and wire the Rust live transport SDK to it so
`LiveTransportKamnClient::balance()` reaches the real service surface instead of returning
`NotImplemented`.

## Inputs/Outputs

- Inputs:
  - authenticated `GET /v1/agents/{did}/balance` requests
  - canonical `kamn:did:agent:...` path DID
  - existing service state files, including older snapshots without persisted agent balance
- Outputs:
  - service returns deterministic JSON containing `did` and `balance`
  - SDK service client parses the response into a typed balance result
  - live transport `balance()` returns `TokenAmount`

## Boundaries/Non-goals

- Do not add a balance mutation/write route.
- Do not implement live `search_agents()`.
- Do not change escrow/task settlement behavior beyond exposing the current read balance.
- Do not change Python/TypeScript SDKs in this issue.

## Failure modes

- invalid or legacy DID path fails closed with the existing typed bad-request route behavior
- unauthorized or wrong-scope requests fail closed under the existing `agents:read` auth policy
- older service state files missing the new balance field still load successfully and default
  persisted/lazy agent balance to `100`
- state persistence failures still return loud service errors

## Acceptance criteria

- [ ] Service API exposes `GET /v1/agents/{did}/balance` under `agents:read`.
- [ ] The route returns a deterministic body with canonical `did` and numeric `balance`.
- [ ] Agent state persists balance and remains backward-compatible with older state snapshots
      that do not yet contain the new field.
- [ ] Lazy-created agent records default balance to `100`, matching the in-memory transport.
- [ ] `ServiceApiClient` supports the balance route.
- [ ] `LiveTransportKamnClient::balance()` uses the real route instead of returning
      `NotImplemented`.
- [ ] Targeted service, SDK, and live transport tests cover auth, persistence, and parity.

## Files to touch

- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/models.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-sdk/src/service_models.rs`
- `crates/kamn-sdk/src/service_client_bridge_misc_routes.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `specs/6562-add-service-backed-live-balance-route.md`

## Error semantics

- Reuse existing service route error envelopes and auth failures.
- Interior storage/client helpers return typed Rust errors and do not log.
- Live SDK balance lookup returns `SdkError` variants through the current service client boundary.

## Test plan

1. Add service endpoint RED tests for:
   - route rendering/body shape
   - scope mapping
   - persistence/restart compatibility with missing-balance state
2. Add service client RED tests for route parsing and scope enforcement.
3. Add live transport RED tests replacing the current unsupported-balance assertion with a real
   loopback contract call.
4. Run targeted `kamn-node`, `kamn-sdk`, and `test_file_size_policy` checks.

## Deviations

- `crates/kamn-node/src/service_api_endpoint/models.rs` was not needed; the route uses existing
  root response types plus a local persisted-balance payload to avoid growing the root module
  surface.
- `crates/kamn-sdk/tests/service_api_client.rs` was left unchanged because the dedicated
  `crates/kamn-sdk/tests/service_api_balance.rs` contract test covers the new route directly.
- Additional integration files were required:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/balance_contract_tests.rs`
  - `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
  - `docs/api/service-http-api.md`
  - `docs/api/service-openapi.yaml`
  - `fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
  - `fixtures/ci/test_file_size_policy_baseline.env`

## Verification

- `cargo test -p kamn-node unit_service_api_endpoint_balance_route_returns_did_and_balance_contract -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_balance_route_backfills_legacy_state_and_persists_restart -- --nocapture`
- `cargo test -p kamn-node unit_service_api_route_authz_matrix_matches_protected_and_public_paths -- --nocapture`
- `cargo test -p kamn-node functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping -- --nocapture`
- `cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-sdk --test service_api_balance -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_balance -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_agent spec_c05_live_transport_unsupported_methods_fail_closed -- --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`
- `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
- `rustfmt --edition 2021 crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs crates/kamn-node/src/service_api_endpoint.rs crates/kamn-node/src/service_api_endpoint/auth.rs crates/kamn-node/src/service_api_endpoint/message_store.rs crates/kamn-node/src/service_api_endpoint/middleware_impl.rs crates/kamn-node/src/service_api_endpoint/payload.rs crates/kamn-sdk/src/lib.rs crates/kamn-sdk/src/live/agent.rs crates/kamn-sdk/src/service.rs crates/kamn-sdk/src/service_client_bridge_misc_routes.rs crates/kamn-sdk/src/service_models.rs crates/kamn-sdk/tests/live_transport_agent.rs crates/kamn-sdk/tests/live_transport_task_escrow.rs crates/kamn-sdk/tests/service_api_balance.rs`

Repo-wide note:

- `cargo fmt --all --check` still reports unrelated pre-existing formatting drift outside issue
  `#6562`; this change set was formatted only on the touched Rust files.
