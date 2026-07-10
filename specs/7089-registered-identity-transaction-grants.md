# Enforce Registered Identity And Transaction-Scoped Grants

## Objective

Separate request authentication from business authorization for the KAMN
transaction rail. A valid signature proves who sent a request; it does not prove
that the signer is registered or allowed to act on a task or escrow resource.

## Inputs/Outputs

Inputs:

- Existing DID/signature, nonce, replay, and route-scope checks.
- Existing durable agent registrations and service API state file.
- Signed request sender DID from verified middleware context.
- Server-owned grant records provisioned through trusted runtime/store setup.

Outputs:

- A durable grant record containing actor DID, resource selector, role, allowed
  action, status, and idempotency key.
- A durable authorization decision receipt containing correlation ID, actor DID,
  resource, action, decision, and stable reason code.
- A middleware authorization gate for transaction-rail task and escrow routes.
- Structured forbidden responses for unregistered and ungranted actors.

## Authorization Contract

The order for transaction-rail requests is:

1. Parse request.
2. Verify DID/key binding, signature, and nonce freshness.
3. Enforce signed route-scope intent.
4. Resolve actor, action, and resource from verified request context.
5. Require a durable registered agent record.
6. Require one active server-owned grant matching actor, action, and resource.
7. Persist an allow or deny authorization receipt.
8. Persist nonce high-watermark and continue to the handler only on allow.

`POST /v1/agents/register` remains authentication-protected but is exempt from
the registration and grant checks so an actor can register itself. Client
registration capabilities are descriptive discovery metadata and never create
authorization.

The first slice covers these actions:

- `POST /v1/tasks/create`: `task:create`, resource `transaction:new`.
- `GET /v1/tasks/{id}`: `task:read`, resource `task:{id}`.
- `POST /v1/tasks/{id}/accept`: `task:accept`, resource `task:{id}`.
- `POST /v1/tasks/{id}/complete`: `task:complete`, resource `task:{id}`.
- `POST /v1/escrow/fund`: `escrow:fund`, resource `task:{task_id}`.
- `POST /v1/escrow/{id}/release`: `escrow:release`, resource `escrow:{id}`.

An exact resource grant wins. The only wildcard allowed in this issue is the
explicit bootstrap selector `transaction:new` for `task:create`; generic `*`
resource grants are invalid. Later issues mint exact task and escrow grants as
resources are created and assigned.

## Boundaries/Non-goals

- Do not modify signature, nonce, replay, or request-signing semantics.
- Do not treat the signed scope header or self-declared capabilities as grants.
- Do not add a grant administration HTTP, SDK, CLI, or MCP route in this issue.
- Trusted store/runtime setup may provision and revoke grants; actor-facing
  issuance is owned by later task/escrow lifecycle issues.
- Do not implement a general RBAC/ABAC engine, organization policy, OAuth, or
  external identity provider.
- Do not change task ownership, escrow terms, or Solana settlement behavior.
- Do not apply the new grant gate to unrelated message, content, bridge,
  directory, or websocket routes in this slice.
- Do not log or persist signatures, auth headers, public keys, nonces, raw task
  bodies, completion data, or private payloads in authorization receipts.

## Failure Modes

- Validly signed unregistered actor: deny with `AGENT_NOT_REGISTERED`.
- Registered actor without a matching active grant: deny with
  `ACTION_NOT_GRANTED`.
- Matching actor/action but wrong resource or role: deny with
  `RESOURCE_ROLE_MISMATCH`.
- Revoked grant: deny with `ACTION_NOT_GRANTED` and a revoked-grant context.
- Invalid grant fields or conflicting idempotency-key reuse: hard persistence
  error; do not replace the existing grant.
- Authorization receipt persistence failure: return an internal persistence
  error and do not reach the domain handler.
- Denied request: persist the deny receipt but do not persist a consumed nonce,
  mutate domain state, or call external settlement.
- State reload failure: fail loud; never authorize from stale in-memory grants.

## Acceptance Criteria

- [x] A signed unregistered actor cannot call any covered task/escrow route.
- [x] A registered actor without a matching grant cannot call a covered route.
- [x] `tasks:write` or `escrow:write` request scope alone never authorizes.
- [x] Active grants match actor DID, exact action, exact resource selector, role,
      and active status.
- [x] Registration and grant decisions reload from the service state file.
- [x] Identical grant provisioning/revocation is idempotent.
- [x] Conflicting idempotency-key reuse fails without replacing durable state.
- [x] Allow and deny decisions persist secret-free authorization receipts.
- [x] Every receipt identifies correlation ID, actor, resource, action, decision,
      role, and reason code.
- [x] Registration remains callable by a valid unregistered signer.
- [x] Existing signature, route-scope, replay, and nonce-restart tests remain
      green.
- [x] Real service API integration tests exercise the shared middleware and
      durable store without mocking either layer.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/auth/`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/auth_flow/`
- `crates/kamn-node/src/service_api_endpoint/message_store/models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/agent_ops.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract/`

No SDK signing change is expected. The SDK continues to send signed scope
intent and receives the existing structured service error envelope.

## Persistence And Compatibility

- Add serde-defaulted grant and authorization-receipt collections to the
  service API snapshot.
- Bump newly created snapshots to a new schema version.
- Existing snapshots without the collections load as empty and therefore fail
  closed for covered transaction-rail actions until trusted grants are
  provisioned.
- Grant provisioning uses an explicit idempotency key and rejects conflicting
  reuse.
- Authorization receipts use deterministic sequence IDs within one snapshot and
  are append-only.

## Error Semantics

Authentication failures retain their existing status and reason codes.
Business authorization failures return HTTP `403 Forbidden` in the standard
JSON error envelope:

- `AGENT_NOT_REGISTERED`
- `ACTION_NOT_GRANTED`
- `RESOURCE_ROLE_MISMATCH`

Persistence and receipt-write failures return HTTP `500` with the existing
state-persistence reason code. Interior code returns typed decisions/errors;
middleware maps each result once and records the decision once.

## Test Plan

RED:

- Signed unregistered task creation is currently accepted.
- Registered actor with no grant can currently reach escrow release.
- Registered actor can self-assert `tasks:write` and reach task creation.
- Active and revoked grant behavior does not survive restart because no grant
  model exists.
- No durable secret-free authorization decision receipt exists.
- Conflicting grant idempotency-key reuse is not rejected.

GREEN:

- Add the minimal persisted grant and receipt models/store operations.
- Resolve covered route actions/resources from method, path, and required body
  fields.
- Insert the business authorization stage after authn/scope and before nonce
  persistence/handler dispatch.
- Exempt self-registration only.

REFACTOR:

- Keep authentication, scope intent, grant evaluation, response mapping, and
  persistence in separate small modules.
- Centralize action/resource parsing and stable reason codes.
- Verify file/function size limits, naming, duplication, and fail-closed paths.

INTEGRATION:

- Provision trusted grants in real state-file fixtures.
- Exercise registration, allowed action, denied action, revocation, restart,
  receipt inspection, and unchanged domain state through the live HTTP server.
- Run targeted tests, formatting, strict clippy, and `make check`.

## Completion Evidence

- RED commit `7938c347`: four authorization contract tests failed before the
  grant model and middleware gate existed.
- GREEN commit `c3c3e8af`: persisted grants, receipts, and middleware policy
  made the authorization contract tests pass.
- REFACTOR commit `15495b92`: protected handlers revalidate grants while
  holding the message-store lock; escrow records persist task ownership;
  denied requests do not consume a nonce.
- `cargo fmt --check` passed.
- `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 cargo clippy -p
  kamn-node --all-targets -- -D warnings` passed.
- `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 cargo test -p
  kamn-node` passed, including 639 binary tests and extraction contracts.
- `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 cargo test -p
  kamn-node service_api_endpoint_tests` passed with 161 service API tests.
- `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 make check` passed
  workspace formatting and strict all-target, all-feature clippy.

## Deviations And Follow-up Boundaries

- No public grant administration surface was added. Production authorization
  consumes trusted preseeded state; test-only helpers provision fixture grants.
- Task and escrow lifecycle code does not issue actor-facing grants in this
  issue. Exact lifecycle issuance remains owned by #7090 and #7091.
- Solana settlement behavior is unchanged and was not claimed as completion
  evidence for this identity/grant slice.
