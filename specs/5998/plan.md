# Plan: Issue #5998

## Approach
1. Extend persisted service-api snapshot with `agents` map (serde-defaulted for backward compatibility).
2. Add message-store method to query-or-initialize agent profile (`did`, `reputation_score`).
3. Wire `GET /v1/agents/{agent_did}` in middleware to message-store method.
4. Add restart integration test with explicit durable state-file assertion.
5. Run relay durability regression test to ensure no behavioral drift.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: legacy state-file deserialization regression after snapshot schema extension.
  Mitigation: add `#[serde(default)]` for `agents` map.
- Risk: route behavior mismatch with existing response contract.
  Mitigation: preserve existing `ServiceApiAgentGetBody` fields and values.

## Interfaces / Contracts
- Route and payload shape unchanged.
- Live path becomes durable and restart-safe.
