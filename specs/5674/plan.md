# Plan: #5674 Remove Remaining agent-lib Stubs via Service/SDK Route Expansion

## Approach
1. Add RED tests in `kamn-node` service API endpoint suite for new route authz/scope and deterministic route payload contracts.
2. Implement service endpoint route constants, payload structs, path extractors, route dispatch, and method-not-allowed routing for:
   - `/v1/tasks/{id}/accept`
   - `/v1/tasks/{id}/complete`
   - `/v1/escrow/fund`
   - `/v1/escrow/{id}/release`
3. Update scope mapping and fixture matrix rows; update route authz matrix counts and fixtures assertions.
4. Add RED tests in `kamn-sdk/tests/service_api_client.rs` for new methods and extend test server contract routes.
5. Implement `ServiceApiClient` methods + response structs and export them in `kamn-sdk` public surface.
6. Add RED tests in `kamn-agent-lib` for previously unsupported operations.
7. Replace stubs in `KamnAgentHandle` and client wrapper with SDK-backed implementations.
8. Run targeted regression suites and lint/fmt for touched crates.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `crates/kamn-agent-lib/src/client.rs`
- `crates/kamn-agent-lib/src/lib.rs`

## Risks and Mitigations
- Risk: Scope-policy drift due new routes.
  - Mitigation: Update fixture + matrix counts + route-to-scope tests together.
- Risk: SDK parsing mismatch with route payload shape.
  - Mitigation: Add SDK contract tests first and keep payload fields minimal/deterministic.
- Risk: Breaking existing route matching with new path extractors.
  - Mitigation: Keep new extractors strict and order route matching to avoid prefix collisions.

## Interfaces/Contracts
- New SDK response models:
  - task-accept receipt
  - task-complete receipt
  - escrow-fund receipt
  - escrow-release receipt
- `KamnAgentHandle` signatures for four target operations will return typed receipts, not `()`.

## ADR
- Not required: no dependency/protocol architecture change, additive deterministic route expansion only.
