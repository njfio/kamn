# 6504 Reject Legacy Agent Profile Path DIDs

## Objective
Reject legacy `did:kamn:agent:...` path values at the service API agent-profile ingress boundary instead of accepting them as valid agent identifiers.

## Inputs/Outputs
- Input: `GET /v1/agents/{agent_did}` requests where `{agent_did}` is supplied through the route path.
- Output on canonical input: existing `200 OK` `ServiceApiAgentGetBody` behavior remains unchanged for canonical `kamn:did:agent:...` values.
- Output on legacy input: fail closed at ingress with a stable JSON error envelope and reason code.

## Boundaries/Non-goals
- Only the service API agent-profile route ingress is in scope.
- Do not change canonical DID parsing rules.
- Do not add compatibility rewrites, silent normalization, or broad DID cleanup outside this ingress.
- Do not alter unrelated service API route matching behavior.

## Failure modes
- Legacy `did:kamn:agent:...` path value is accepted and persisted as a new agent record.
- Legacy path value reaches storage lookup/create paths instead of being rejected at ingress.
- Rejection is untyped or unstable at the HTTP boundary.
- Canonical `kamn:did:agent:...` path values regress from the current success path.

## Acceptance criteria
- [ ] A regression test proves `GET /v1/agents/did:kamn:agent:legacy-alpha` is rejected at service API ingress.
- [ ] The rejection uses a stable JSON error envelope with a pinned reason code.
- [ ] A regression test proves canonical `GET /v1/agents/kamn:did:agent:alpha` still succeeds.
- [ ] No agent record is created for rejected legacy path values.
- [ ] The spec is updated with final verification evidence and any deviations before closure.

## Files to touch
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/6504-reject-legacy-agent-profile-path-dids.md`

## Error semantics
- Interior path parsing returns a typed invalid-path result rather than silently treating legacy DID values as valid route parameters.
- The service API entrypoint returns a hard failure JSON error envelope for legacy `did:kamn:...` agent path values.
- The error envelope must expose a stable reason code so callers can distinguish invalid agent DID input from persistence failures.
- No silent fallback, normalization, or route coercion is allowed.

## Test plan
- Add a red regression test covering `GET /v1/agents/did:kamn:agent:legacy-alpha` through the real service API rendering path and assert non-200 plus pinned reason code.
- Assert rejected legacy path input does not create an agent record in persisted state.
- Keep or add a canonical route regression proving `GET /v1/agents/kamn:did:agent:alpha` still returns `200 OK`.
- Run targeted `kamn-node` service API tests covering the new rejection and canonical success path.
