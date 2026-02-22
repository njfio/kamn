# Spec: #5674 Remove Remaining agent-lib Stubs via Service/SDK Route Expansion

- Issue: #5674
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
`kamn-agent-lib` still returns `UnsupportedOperation` for `accept_task`, `complete_task`, `fund_escrow`, and `release_escrow` because service API + SDK route coverage is incomplete for those operations. This blocks MCP/CLI parity and violates the PRD portable-agent goal.

## Scope
### In Scope
- Add deterministic service API route contracts for:
  - `POST /v1/tasks/{id}/accept`
  - `POST /v1/tasks/{id}/complete`
  - `POST /v1/escrow/fund`
  - `POST /v1/escrow/{id}/release`
- Extend service route authz/scope policy fixtures and matrix counts for new protected routes.
- Extend `kamn-sdk::ServiceApiClient` with typed methods + response models for the new routes.
- Wire `kamn-agent-lib::ServiceApiHttpClient` and `KamnAgentHandle` to call those SDK methods.
- Replace stubbed `UnsupportedOperation` behavior for the four operations.

### Out of Scope
- MCP/CLI activation for these operations (handled in #5676).
- New escrow economics rules or settlement logic.
- Non-deterministic business state transitions beyond route contracts.

## Acceptance Criteria
### AC-1 Service API route contracts
Given service API route rendering receives signed requests for task accept/complete and escrow fund/release,
When routes are invoked,
Then deterministic JSON payloads are returned with stable status codes and required fields.

### AC-2 Scope/authz coherence
Given route authz matrix and scope fixture contracts,
When new routes are introduced,
Then protected/public counts, fixture rows, and required scopes remain coherent and pass.

### AC-3 SDK typed coverage
Given `ServiceApiClient`,
When callers invoke accept/complete/fund/release methods,
Then methods enforce input validation, issue correct routes, and decode deterministic response bodies.

### AC-4 agent-lib stub removal
Given `KamnAgentHandle`,
When `accept_task`, `complete_task`, `fund_escrow`, and `release_escrow` are called,
Then they no longer return `UnsupportedOperation` and instead return typed service responses via SDK-backed calls.

### AC-5 Regression stability
Given existing service API and SDK suites,
When all relevant tests run,
Then prior route contracts remain green alongside the new route coverage.

## Conformance Cases
- C-01 (AC-1): Service route tests validate status + payload fields for all four new routes.
- C-02 (AC-2): Route authz matrix total/protected/public counts and scope fixture rows reflect new routes/scopes.
- C-03 (AC-3): SDK contract tests validate request/response behavior for new methods and route paths.
- C-04 (AC-4): agent-lib tests confirm former stubbed operations succeed with deterministic responses.
- C-05 (AC-5): Regression suites across `kamn-node`, `kamn-sdk`, and `kamn-agent-lib` remain green.

## Success Metrics
- No `UnsupportedOperation` returned from the four target `KamnAgentHandle` operations.
- New route and SDK tests pass in targeted suites.
- Existing affected contract suites pass unchanged in behavior outside added routes.
