# 6510 Reject Legacy WebSocket Presence DIDs

## Objective
Reject legacy `did:kamn:...` presence owner, target-owner, and target-agent header values at the websocket presence ingress boundary instead of allowing them to flow into presence projection.

## Inputs/Outputs
- Input: websocket upgrade requests using `X-KAMN-Events-Mode: presence` with presence DID headers.
- Output on canonical input: existing presence websocket projection behavior remains unchanged for canonical `kamn:did:...` owner and agent DID header values.
- Output on legacy input: fail closed at ingress with stable websocket reason codes in the JSON error envelope.

## Boundaries/Non-goals
- Only websocket presence DID header ingress is in scope.
- Do not change canonical DID parsing rules.
- Do not change non-DID websocket presence validation such as timestamps or capabilities.
- Do not add compatibility rewrites, silent normalization, or broader DID cleanup outside websocket presence ingress.

## Failure modes
- Legacy `did:kamn:...` owner DID header is accepted and forwarded into presence projection.
- Legacy `did:kamn:...` target-owner DID header is accepted when present.
- Legacy `did:kamn:...` target-agent DID header is accepted and forwarded into presence projection.
- Rejection uses unstable or generic websocket reason codes instead of explicit invalid-header codes.
- Canonical presence DID header flows regress from the current success path.

## Acceptance criteria
- [ ] A regression test proves presence mode rejects legacy owner DID header values at ingress.
- [ ] A regression test proves presence mode rejects legacy target-owner DID header values when provided.
- [ ] A regression test proves presence mode rejects legacy target-agent DID header values at ingress.
- [ ] The rejection uses stable websocket reason codes pinned in the websocket taxonomy inventory.
- [ ] A regression test proves canonical presence DID header values still succeed.
- [ ] The spec is updated with final verification evidence and any deviations before closure.

## Files to touch
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs`
- `specs/6510-reject-legacy-websocket-presence-dids.md`

## Error semantics
- Presence DID header parsing returns typed invalid-header errors rather than treating legacy DID values as valid non-empty headers.
- The websocket entrypoint returns hard-failure JSON error envelopes for invalid presence DID headers.
- Invalid owner, target-owner, and target-agent DID headers must each expose stable websocket reason codes included in the websocket taxonomy inventory.
- No silent fallback, normalization, or projection attempt is allowed after invalid DID header detection.

## Test plan
- Add red websocket regression coverage for legacy owner DID, target-owner DID, and target-agent DID header values through the real websocket upgrade path.
- Assert canonical presence DID header values still succeed.
- Assert websocket reason taxonomy tests observe the new invalid-header codes.
- Run targeted `kamn-node` websocket contract tests covering the new rejection path and an existing canonical presence success path.
