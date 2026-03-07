# Spec: Issue #6564 - Map live transport mailbox receive routes

## Objective

Replace the live SDK's `receive()` and `receive_stream()` hard `NotImplemented` failures with a
real mailbox-backed implementation that composes the existing service routes:
`GET /v1/channels/recipient:{did}/messages` and `GET /v1/messages/{id}`.

## Inputs/Outputs

- Inputs:
  - authenticated live transport receive request for a canonical recipient DID
  - service mailbox channel response containing zero or more service message ids
  - service message query responses containing `message_id`, `status`, `sender_did`,
    `recipient_did`, and `body`
- Outputs:
  - `receive()` returns `Vec<MessageRecord>` with deterministic SDK alias ids derived from service
    message ids
  - `receive_stream()` returns a `MessageStream` built from the same real records
  - invalid message query payloads fail closed with `SdkError`

## Boundaries/Non-goals

- Do not add or change service API routes.
- Do not implement live agent registration, agent search, or artifact submission.
- Do not change service mailbox persistence or delivery-state mutation semantics.
- Do not change public OpenAPI/docs for the service surface in this issue.

## Failure modes

- malformed or legacy recipient DID input fails closed through the existing DID parser boundary
- mailbox query connection/auth failures surface as existing `SdkError` transport/request errors
- message query responses missing `sender_did`, `recipient_did`, or `body` fail closed with typed
  invalid-response errors
- message query responses containing invalid DID strings fail closed through `AgentDid::parse`
- message query responses with non-delivery body shape fail closed when translating back into SDK
  `Message`

## Acceptance criteria

- [ ] `LiveTransportKamnClient::receive()` calls the recipient mailbox route for the requested DID.
- [ ] `LiveTransportKamnClient::receive()` fetches every listed message id through the existing
      message query route.
- [ ] `LiveTransportKamnClient::receive()` returns `MessageRecord` values with parsed sender,
      recipient, and body fields plus deterministic `MessageId` aliases.
- [ ] `LiveTransportKamnClient::receive_stream()` is backed by the real receive path.
- [ ] Empty mailbox responses return no records and no stream items without error.
- [ ] Missing message fields and malformed message payloads fail closed and are covered by tests.
- [ ] Unrelated unsupported live methods remain unsupported.

## Files to touch

- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/routes.rs`
- `crates/kamn-sdk/src/live/state.rs`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_models.rs`
- `crates/kamn-sdk/src/service_client_message_task_routes.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/live_transport_receive.rs`
- `specs/6564-map-live-mailbox-receive-routes.md`

## Error semantics

- Interior translation helpers return `SdkError` and do not log.
- Mailbox/message route failures propagate the existing `ServiceApiClient` error mapping.
- Translation from service message payloads to SDK `MessageRecord` fails fast on any missing or
  invalid field; there are no silent defaults.

## Test plan

1. Add RED live transport contract coverage for mailbox listing + per-message fetch.
2. Add RED failure-path coverage for empty mailbox, missing message fields, and malformed payloads.
3. Add the minimum internal service-model/client plumbing to parse full message query payloads.
4. Run targeted `kamn-sdk` tests and clippy.

## Deviations

- `crates/kamn-sdk/tests/support/live_transport_contract_server.rs` and
  `crates/kamn-sdk/tests/support/live_transport_http.rs` were not changed; the existing harness
  already covered the mailbox/message route contract shape needed for this issue.
- No service API route or OpenAPI change was required. The live SDK composes the existing
  mailbox-list and message-query routes and keeps the richer message-delivery model crate-private.

## Verification

- `cargo test -p kamn-sdk --test live_transport_receive -- --nocapture`
- `cargo test -p kamn-sdk --test live_transport_agent spec_c05_live_transport_remaining_unsupported_methods_fail_closed -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
- `cargo test -p kamn-sdk -- --nocapture`
