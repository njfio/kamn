# 6508 Reject Legacy Relay Ingest DIDs

## Objective
Reject legacy `did:kamn:agent:...` sender or recipient DID values at the service API `/v1/messages/relay` ingress boundary instead of accepting them for relayed message persistence.

## Inputs/Outputs
- Input: `POST /v1/messages/relay` JSON payloads carrying a required `recipient_did`, optional `sender_did`, `message_id`, and `body`.
- Output on canonical input: existing relay-ingest acceptance behavior remains unchanged for canonical `kamn:did:agent:...` DID values.
- Output on legacy input: fail closed at ingress with a stable JSON error envelope and reason code.

## Boundaries/Non-goals
- Only service API relay-ingest DID ingress is in scope.
- Do not change canonical DID parsing rules.
- Do not change unrelated relay payload semantics.
- Do not add compatibility rewrites, silent normalization, or broader DID cleanup outside this ingress.

## Failure modes
- Legacy `did:kamn:agent:...` recipient DID is accepted and persisted as a relayed message.
- Legacy `did:kamn:agent:...` sender DID is accepted and persisted when present.
- Rejection is untyped or unstable at the HTTP boundary.
- Canonical relay payload DID values regress from the current accepted path.

## Acceptance criteria
- [ ] A regression test proves `/v1/messages/relay` rejects legacy `did:kamn:...` recipient DID values at ingress.
- [ ] A regression test proves `/v1/messages/relay` rejects legacy `did:kamn:...` sender DID values when present.
- [ ] The rejection uses a stable JSON error envelope with a pinned reason code.
- [ ] A regression test proves canonical relay payload DID values still succeed.
- [ ] Rejected legacy relay payloads do not create persisted message records.
- [ ] The spec is updated with final verification evidence and any deviations before closure.

## Files to touch
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/6508-reject-legacy-relay-ingest-dids.md`

## Error semantics
- Interior relay payload parsing returns a typed invalid-relay-DID result instead of silently treating legacy DID values as valid relay inputs.
- The service API entrypoint returns a hard-failure JSON error envelope for legacy relay DID values.
- The error envelope must expose a stable reason code so callers can distinguish invalid relay DID input from generic relay payload shape failures.
- No silent fallback, normalization, or partial persistence is allowed.

## Test plan
- Add a red regression test covering legacy recipient and sender DID values through the real `/v1/messages/relay` endpoint and assert non-202 plus pinned reason code.
- Assert canonical relay payload DID values still succeed through the same endpoint.
- Assert rejected legacy relay payloads do not create persisted relayed messages.
- Run targeted `kamn-node` service API tests covering the new rejection and canonical success path.
