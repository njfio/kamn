# Objective
Replace the self-certifying sender-DID signer public-key compare in `crates/kamn-node/src/service_api_endpoint/auth.rs` with a constant-time normalized comparison while preserving the existing auth-boundary behavior.

# Inputs/Outputs
- Input: `sender_did_matches_signer_public_key(sender_did, signer_public_key_hex, allow_legacy_sender_binding)`
- Output: `true` when the sender DID is bound to the same signer public key under current policy, otherwise `false`

# Boundaries/Non-goals
- Do not change header-name case handling.
- Do not change legacy sender-binding policy for non-self-certifying sender DIDs.
- Do not change service API signature verification flow outside sender-DID key binding.

# Failure modes
- Self-certifying sender DIDs with mismatched signer public keys must return `false`.
- Malformed or differently encoded signer public-key values must not bypass the binding check.
- Reintroducing direct `eq_ignore_ascii_case()` in this compare must fail a source-contract regression test.

# Acceptance criteria
- [ ] `sender_did_matches_signer_public_key()` does not rely on `eq_ignore_ascii_case()` for self-certifying DID signer-key matching.
- [ ] Matching self-certifying sender DIDs still accept uppercase/lowercase hex variants representing the same key.
- [ ] Mismatched self-certifying sender DIDs still return `false`.
- [ ] Legacy sender-binding behavior remains unchanged.
- [ ] A dedicated source-contract regression test pins the auth-boundary compare away from `eq_ignore_ascii_case()`.

# Files to touch
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/tests/service_api_sender_did_constant_time.rs`
- `specs/6586-harden-service-api-sender-did-signer-compare.md`
- `fixtures/ci/test_file_size_policy_baseline.env` only if adding the new test target causes inventory drift

# Error semantics
- No new error types.
- Existing auth-boundary mismatch behavior remains a boolean `false` from `sender_did_matches_signer_public_key()` and continues surfacing through existing request-auth failures.

# Test plan
- Add a source-contract test that inspects `sender_did_matches_signer_public_key()` and fails if `eq_ignore_ascii_case()` appears there.
- Add runtime tests covering case-variant acceptance for the same self-certifying key.
- Add runtime tests covering mismatched self-certifying key rejection.
- Re-run existing legacy sender-binding tests to confirm unchanged behavior.
