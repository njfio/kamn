# 6576-harden-agent-did-key-binding-compare

## Objective
Replace direct equality in `AgentDid::ensure_public_key_hex_binding()` with the existing internal constant-time helper so DID-embedded key-binding fingerprint verification does not rely on plain string comparison.

## Inputs/Outputs
- Input: `&self` bound `AgentDid`, `public_key_hex: &str`
- Output success: `Ok(())` when the derived public-key fingerprint matches the DID-embedded fingerprint
- Output failure:
  - `AgentDidKeyBindingError::MissingKeyBinding` when the DID lacks a key-binding suffix
  - `AgentDidKeyBindingError::InvalidPublicKeyHex` when `public_key_hex` cannot be decoded
  - `AgentDidKeyBindingError::KeyBindingMismatch { expected, actual }` when the derived fingerprint differs

## Boundaries/Non-goals
- No DID format changes
- No repo-wide constant-time comparison sweep
- No new dependencies
- No error taxonomy or payload-shape changes
- No CI/workflow/docs policy changes

## Failure modes
- DID has no key-binding fingerprint suffix
- Provided `public_key_hex` is malformed
- Provided public key derives a fingerprint that does not match the DID binding
- Regression to direct equality instead of constant-time helper

## Acceptance criteria
- [ ] `AgentDid::ensure_public_key_hex_binding()` uses `crate::constant_time_eq::constant_time_eq_bytes(...)` for fingerprint comparison
- [ ] Matching bound DID and public key still return `Ok(())`
- [ ] Mismatched public key still returns `AgentDidKeyBindingError::KeyBindingMismatch` with unchanged `expected` and `actual` values
- [ ] Malformed public key hex still returns `AgentDidKeyBindingError::InvalidPublicKeyHex`
- [ ] A regression test fails if the implementation reverts to direct equality

## Files to touch
- `crates/kamn-core/src/did.rs`
- `specs/6576-harden-agent-did-key-binding-compare.md`

## Error semantics
- Preserve current typed errors exactly
- Do not log inside DID helpers
- Fail fast on malformed public key hex before comparison

## Test plan
- Add a source-contract regression test that requires the constant-time helper call and rejects the prior direct equality pattern
- Keep the matching-public-key success coverage green
- Assert mismatched public key returns `KeyBindingMismatch` and preserves `expected`/`actual`
- Assert malformed public key hex returns `InvalidPublicKeyHex`
- Run targeted `kamn-core` DID tests and strict clippy for the touched crate
