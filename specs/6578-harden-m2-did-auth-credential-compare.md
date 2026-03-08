# 6578-harden-m2-did-auth-credential-compare

## Objective
Replace direct equality in `DataLayerM2DidSessionService::authenticate()` with the existing internal constant-time helper so derived DID auth credential verification does not rely on plain string comparison.

## Inputs/Outputs
- Input: `DataLayerM2DidAuthRequest`
- Output success: `Ok(DataLayerM2SessionToken)` when the provided credential matches the derived expected credential and the request is otherwise valid
- Output failure:
  - `DataLayerM2GatewayError::InvalidSessionTtl` when the TTL is invalid
  - `DataLayerM2GatewayError::InvalidCredential("credential signature mismatch")` when the provided credential differs from the expected credential
  - existing validation errors from `DataLayerM2DidAuthRequestValidated::try_from(...)`

## Boundaries/Non-goals
- No DID format changes
- No token format changes
- No repo-wide constant-time comparison sweep
- No new dependencies
- No CI/workflow/docs policy changes

## Failure modes
- Request TTL is zero or exceeds service max
- Request validation rejects malformed DID auth fields
- Provided credential differs from the derived expected credential
- Regression to direct equality instead of constant-time helper

## Acceptance criteria
- [x] `DataLayerM2DidSessionService::authenticate()` uses `crate::constant_time_eq::constant_time_eq_bytes(...)` for credential comparison
- [x] Matching credentials still mint the same deterministic session token payload
- [x] Mismatched credentials still return `DataLayerM2GatewayError::InvalidCredential("credential signature mismatch")`
- [x] A regression test fails if the implementation reverts to direct equality
- [x] No public API or error taxonomy changes

## Files to touch
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `specs/6578-harden-m2-did-auth-credential-compare.md`

## Error semantics
- Preserve current typed errors exactly
- Do not log inside the M2 service helper
- Preserve the existing mismatch error message string exactly

## Test plan
- Add a source-contract regression test that requires the constant-time helper call and rejects the prior direct equality pattern
- Assert matching credentials still mint the expected deterministic token
- Assert mismatched credentials still return the same `InvalidCredential` payload
- Run targeted `kamn-core` M2 tests and strict clippy for the touched crate

## Integration verification
- Existing public boundary preserved: `DataLayerM2DidSessionService::authenticate(...)`
- Verified deterministic token minting still succeeds for the canonical valid request
- Verified the real mismatch path still fails closed with the existing `InvalidCredential` payload

## Verification actuals
- `cargo test -p kamn-core data_layer_m2_gateway_access::tests -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Deviations
- None
