# Plan: Issue #5996

## Approach
1. Add `create_channel` persistence method in `ServiceApiMessageStore` that allocates deterministic channel IDs and persists `channel_messages` entry.
2. Wire `POST /v1/channels/create` in middleware to call message-store method and return persisted response.
3. Add restart integration test validating:
   - channel create success
   - state-file persistence marker for created channel
   - restart query by channel id
4. Re-run relay durability integration regression lane.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: changing channel create route could alter response shape.
  Mitigation: preserve existing `ServiceApiChannelCreateBody` contract and status code.
- Risk: restart test may pass without persistence if assertions are weak.
  Mitigation: assert durable state file contains created channel ID.

## Interfaces / Contracts
- Route path and response contract unchanged.
- Durable behavior added in live middleware path.
