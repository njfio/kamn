# Spec: #5674 Remove Remaining Agent-Lib Stubs via Service/SDK Route Expansion

- Issue: #5674
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-agent-lib` still has four `UnsupportedOperation` stubs due missing service API + SDK route support for task accept/complete and escrow fund/release.

## Scope
### In Scope
- Add deterministic service API routes:
  - `POST /v1/tasks/{id}/accept`
  - `POST /v1/tasks/{id}/complete`
  - `POST /v1/escrow/fund`
  - `POST /v1/escrow/{id}/release`
- Extend route parsing helpers and method-not-allowed/route-exists detection.
- Update route authz matrix and scope-policy fixture/mapping coverage for new routes.
- Add SDK response models + methods for all four routes.
- Replace remaining agent-lib stubs with SDK-backed implementations.
- Add conformance tests across node payload, SDK client, and agent-lib.

### Out of Scope
- Non-deterministic runtime task/escrow business logic.
- External chain/economics integration changes.

## Acceptance Criteria
### AC-1 Service route exposure
Given valid authenticated requests,
When service API receives accept/complete/fund/release routes,
Then it returns deterministic route-contract payloads with expected status codes.

### AC-2 Route governance coherence
Given expanded protected route surface,
When route authz matrix and scope-policy checks run,
Then all governance fixtures/metrics remain consistent with the new route inventory.

### AC-3 SDK method support
Given valid request inputs,
When `ServiceApiClient` calls accept/complete/fund/release methods,
Then it targets the correct routes and decodes typed response models.

### AC-4 Agent-lib stub removal
Given an initialized `KamnAgentHandle`,
When `accept_task`, `complete_task`, `fund_escrow`, or `release_escrow` is called,
Then each operation executes via SDK client and no longer returns `UnsupportedOperation`.

### AC-5 Regression preservation
Given existing route behavior for previously supported operations,
When the new routes are added,
Then prior route and client contracts remain passing.

## Conformance Cases
- C-01 (AC-1): Node payload tests verify deterministic response contracts for all four new routes.
- C-02 (AC-2): Route authz matrix and scope-policy fixture tests include new routes/scopes with consistent counts.
- C-03 (AC-3): SDK service client tests verify route composition/status decode for accept/complete/fund/release.
- C-04 (AC-4): Agent-lib tests verify former stub operations now return typed success receipts.
- C-05 (AC-5): Existing SDK/service/agent-lib contract tests remain green.

## Success Metrics
- `cargo test -p kamn-node --test service_api_endpoint_tests` passes.
- `cargo test -p kamn-sdk --test service_api_client` passes.
- `cargo test -p kamn-agent-lib` passes with new operation coverage.
- `cargo fmt --all --check` and targeted clippy passes for touched crates.
