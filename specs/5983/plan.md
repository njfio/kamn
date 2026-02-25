# Plan: Issue #5983

## Approach
- Introduce a daemon relay routing table sourced from environment and keyed by recipient DID.
- Extend daemon relay tick loop to:
  - drain relay spool entries,
  - resolve recipient route,
  - forward relay payload to recipient node,
  - project sender state only after successful forward,
  - retain failed entries in spool for retry.
- Add recipient-side ingestion path that persists forwarded relay payload into existing message store/mailbox structures in `relayed` state.
- Reuse existing `GET /v1/messages/{id}` and mailbox query behavior for recipient retrieval and `delivered` transition.
- Add end-to-end integration tests with two live Service API endpoints and daemon ticks to verify delivery + restart durability + idempotency.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/state_io.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/daemon_relay_projection_tests.rs`

## Risks / Mitigations
- Routing misconfiguration can stall delivery.
  Mitigation: deterministic parser + explicit reason-code logging + retry-preserving spool behavior.
- Duplicate relay attempts can duplicate recipient records.
  Mitigation: recipient-side idempotent upsert keyed by message identity.
- New forwarding path could weaken existing auth assumptions.
  Mitigation: dedicated internal relay route contract and regression tests for existing external auth flow.

## Interfaces / Contracts
- New env contract for relay routing map (recipient DID -> host:port or base URL).
- Relay-forward payload contract between daemon and recipient ingestion route.
- Sender projection contract: only mark sender message `relayed` after successful recipient persistence.
