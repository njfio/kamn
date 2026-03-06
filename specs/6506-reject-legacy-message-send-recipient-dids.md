# 6506 Reject Legacy Message-Send Recipient DIDs

## Objective
Reject legacy `did:kamn:agent:...` recipient DID values at the service API `/v1/messages/send` ingress boundary instead of accepting them for message persistence or relay routing.

## Inputs/Outputs
- Input: `POST /v1/messages/send` JSON payloads carrying a recipient DID through `recipient_did`, `to`, or `to_did`.
- Output on canonical input: existing accepted message creation behavior remains unchanged for canonical `kamn:did:agent:...` recipient DID values.
- Output on legacy input: fail closed at ingress with a stable JSON error envelope and reason code.

## Boundaries/Non-goals
- Only service API message-send recipient DID ingress is in scope.
- Do not change canonical DID parsing rules.
- Do not change unrelated message-send payload semantics.
- Do not add compatibility rewrites, silent normalization, or broader DID cleanup outside this ingress.

## Failure modes
- Legacy `did:kamn:agent:...` recipient DID is accepted and persisted as part of a created message.
- Legacy recipient DID is accepted into relay spool side effects.
- Rejection is untyped or unstable at the HTTP boundary.
- Canonical recipient DID message-send behavior regresses from the current accepted path.

## Acceptance criteria
- [ ] A regression test proves `/v1/messages/send` rejects legacy `did:kamn:...` recipient DID values at ingress.
- [ ] The rejection uses a stable JSON error envelope with a pinned reason code.
- [ ] A regression test proves canonical `kamn:did:agent:...` recipient DID values still succeed.
- [ ] Rejected legacy recipient DID values do not create persisted messages or relay spool entries.
- [ ] The spec is updated with final verification evidence and any deviations before closure.

## Files to touch
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/6506-reject-legacy-message-send-recipient-dids.md`

## Error semantics
- Interior payload parsing returns a typed invalid-recipient-DID result instead of silently treating legacy DID values as valid recipient inputs.
- The service API entrypoint returns a hard-failure JSON error envelope for legacy `did:kamn:...` recipient DID values.
- The error envelope must expose a stable reason code so callers can distinguish invalid recipient DID input from persistence failures.
- No silent fallback, normalization, or partial persistence is allowed.

## Test plan
- Add a red regression test covering `POST /v1/messages/send` with a legacy `did:kamn:...` recipient DID through the real service API endpoint and assert non-202 plus pinned reason code.
- Assert rejected legacy recipient DID input does not create persisted message records or relay spool entries.
- Keep or add a canonical send regression proving `POST /v1/messages/send` with canonical recipient DID still succeeds.
- Run targeted `kamn-node` service API tests covering the new rejection and canonical success path.

## Integration notes
- Real ingress wiring remains in `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` through `handle_service_api_http_route`.
- Message-send recipient extraction now validates canonical DID shape before `create_message` or relay spool append can run.
- Rejected legacy recipient DID values fail before any message persistence or recipient relay side effects occur.

## Verification evidence
- Red:
  - `cargo test -p kamn-node integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids -- --nocapture`
  - Observed failure before implementation: legacy recipient send returned `status_code=202 outcome=handled`
- Green/Integration:
  - `cargo clippy -p kamn-node --tests -- -D warnings`
  - `cargo test -p kamn-node integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids -- --nocapture`
  - `cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --nocapture`

## Deviations
- None
