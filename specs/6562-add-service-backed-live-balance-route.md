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
