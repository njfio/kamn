# Plan: Issue #5845

## Approach
1. Extend the service API persisted store to cover task and escrow entities plus recipient mailbox projection metadata.
2. Route task/escrow handlers in middleware through the persisted store instead of static payload rendering.
3. Add recipient-aware message retrieval semantics that transition `created` -> `delivered` for recipient readers.
4. Add targeted integration/regression tests for task/escrow persistence and recipient delivery transitions.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: Contract drift in existing route payloads.
  - Mitigation: Preserve existing required fields and only add optional fields.
- Risk: State-file write amplification during retrieval status transitions.
  - Mitigation: Persist only when status actually changes.
- Risk: Authentication/scope regressions due to route handler branching.
  - Mitigation: Keep route paths and scope mapping unchanged; only switch backend storage source.

## Interfaces / Contracts
- `ServiceApiMessageStore` gains persisted task/escrow and recipient mailbox support.
- `GET /v1/messages/{id}` remains compatible and may include optional metadata fields.
- `POST /v1/tasks/create`, `POST /v1/tasks/{id}/accept|complete`, `GET /v1/tasks/{id}` become store-backed.
- `POST /v1/escrow/fund`, `POST /v1/escrow/{id}/release` become store-backed.
