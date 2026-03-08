# 6574 Harden Service-Auth Recovered-Key Comparison

## Objective
Replace the final expected-vs-recovered secp256k1 public key equality check in `service_auth_verify_with_public_key_hex()` with the existing internal constant-time helper, preserving all current verification outcomes and error semantics.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/signature_profile.rs`
  - `crates/kamn-core/src/constant_time_eq.rs`
  - `specs/6574-harden-service-auth-recovered-key-compare.md`
- Outputs:
  - `service_auth_verify_with_public_key_hex()` uses the internal constant-time helper for recovered-key verification
  - regression coverage that fails if the recovered-key check reverts to direct equality

## Boundaries/Non-goals
- Do not add dependencies.
- Do not change public API signatures or exported types.
- Do not replace unrelated equality checks in `signature_profile.rs`.
- Do not modify CI/workflows or governance policy.
- Do not change service-auth payload rendering, recovery-id parsing, or signature decoding behavior.

## Failure modes
- recovered-key verification continues using direct equality.
- helper adoption changes valid verification outcomes.
- malformed signature/public-key inputs change error mapping.
- regression tests fail to pin the recovered-key comparison path.

## Acceptance criteria
- [ ] `service_auth_verify_with_public_key_hex()` uses the internal constant-time helper for expected-vs-recovered compressed public key comparison.
- [ ] valid signatures still verify with the matching public key.
- [ ] tampered payloads and wrong public keys still fail with the existing verification error.
- [ ] malformed signature/public-key inputs keep their current error mapping.
- [ ] regression tests fail if the recovered-key comparison reverts to direct equality.

## Files to touch
- `crates/kamn-core/src/signature_profile.rs`
- `specs/6574-harden-service-auth-recovered-key-compare.md`

## Error semantics
- `ServiceAuthSignatureError` variants remain unchanged.
- wrong recovered-key match continues to return `ServiceAuthSignatureError::VerificationFailure`.
- malformed signature/public-key inputs continue to return their current typed parse/format errors.

## Test plan
- Red:
  - add regression tests that fail if `service_auth_verify_with_public_key_hex()` uses direct recovered-key equality
  - add behavior coverage for wrong-public-key and malformed-input paths to pin current outcomes
- Green:
  - switch only the recovered-key comparison to the existing internal constant-time helper
- Refactor:
  - keep the comparison local and readable; no new helper duplication
- Integration:
  - run targeted `kamn-core` library tests covering `signature_profile`
