## Objective
Reject legacy `did:kamn:agent:...` values in the SDK `get_agent_profile()` route helper so the SDK
fails closed at the client boundary before emitting `/v1/agents/{did}` requests.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-sdk/src/service_client_bridge_misc_routes.rs`
  - `crates/kamn-sdk/tests/service_api_client.rs`
- Outputs:
  - client-side rejection of legacy agent-profile DID inputs
  - preserved canonical `kamn:did:agent:*` agent-profile behavior
  - regression coverage for legacy rejection and unchanged route-segment hardening

## Boundaries/Non-goals
- Do not change service API ingress behavior.
- Do not change unrelated SDK route helpers.
- Do not add compatibility rewrites from `did:kamn:...` to `kamn:did:...`.
- Do not weaken existing CRLF/route-segment validation.

## Failure modes
- `get_agent_profile()` still accepts legacy `did:kamn:agent:*` values and only fails at transport
  time.
- Canonical `kamn:did:agent:*` agent-profile requests regress.
- Existing CRLF route-segment rejection changes reason or error family.
- The SDK silently rewrites legacy DID inputs instead of failing closed.

## Acceptance criteria
- [x] `get_agent_profile()` rejects legacy `did:kamn:agent:*` values before request emission.
- [x] Canonical `kamn:did:agent:*` agent-profile requests still succeed through the existing route
      path.
- [x] Existing CRLF route-segment rejection remains unchanged.
- [x] SDK tests cover both the legacy rejection path and canonical acceptance path.

## Files to touch
- `crates/kamn-sdk/src/service_client_bridge_misc_routes.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `specs/6514-reject-legacy-sdk-agent-profile-dids.md`

## Error semantics
- Legacy `did:kamn:...` agent-profile inputs must fail closed in the SDK before request emission.
- Rejection must surface as `SdkError::InvalidInput`.
- Route-segment validation still runs first for malformed or injected values.
- No silent normalization or fallback is allowed.

## Test plan
- Red:
  - add a client test expecting legacy agent-profile DIDs to fail with `SdkError::InvalidInput`
  - confirm the existing canonical agent-profile integration test still documents the happy path
- Green:
  - `cargo test -p kamn-sdk regression_service_api_client_rejects_legacy_agent_profile_did -- --nocapture`
  - `cargo test -p kamn-sdk integration_service_api_client_service_routes_smoke -- --nocapture`
- Refactor:
  - rerun focused SDK client tests after cleanup

## Deviations
- None.

## Execution Evidence
- Red:
  - `cargo test -p kamn-sdk regression_service_api_client_rejects_legacy_agent_profile_did -- --nocapture`
- Green:
  - `cargo test -p kamn-sdk regression_service_api_client_rejects_legacy_agent_profile_did -- --nocapture`
  - `cargo test -p kamn-sdk regression_service_api_client_rejects_crlf_agent_did_route_payload -- --nocapture`
  - `cargo test -p kamn-sdk functional_service_api_client_executes_signed_http_route_contracts -- --nocapture`
- Refactor / Integration:
  - `cargo test -p kamn-sdk service_api_client -- --nocapture`
  - `cargo clippy -p kamn-sdk --tests -- -D warnings`
