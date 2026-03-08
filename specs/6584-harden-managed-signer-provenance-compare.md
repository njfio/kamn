# 6584-harden-managed-signer-provenance-compare

## Objective
Replace `eq_ignore_ascii_case()` in `verify_kolme_live_managed_signer_backend_signature_provenance()` with a fail-closed constant-time comparison over normalized signer public-key bytes while preserving current case-insensitive acceptance semantics for valid hex inputs.

## Inputs/Outputs
- Input: `canonical_message`, `expected_signer_public_key_hex`, `ManagedExternalBackendSignature`
- Output success: `Ok(())` when the backend signer public key matches the expected runtime signer key and the backend signature proves against that key
- Output failure:
  - existing `managed_signer_backend_response_provenance_mismatch` errors for signer key mismatch and signature/key mismatch
  - existing `managed_signer_backend_response_provenance_malformed` errors for invalid backend signer key material
  - existing runtime error when expected signer public key is empty

## Boundaries/Non-goals
- No signer protocol changes
- No command invocation changes
- No repo-wide constant-time sweep
- No CI/workflow changes

## Failure modes
- Uppercase/lowercase equivalent valid hex no longer matches
- Mismatched signer key stops preserving `managed_signer_backend_response_provenance_mismatch`
- Malformed expected or backend signer key hex stops failing closed deterministically
- Regression to `eq_ignore_ascii_case()` in the provenance compare

## Acceptance criteria
- [ ] `verify_kolme_live_managed_signer_backend_signature_provenance()` no longer uses `eq_ignore_ascii_case()` for signer public-key provenance matching
- [ ] Case-variant-equivalent valid hex still verifies successfully
- [ ] Mismatched signer public key still returns the existing `managed_signer_backend_response_provenance_mismatch` error
- [ ] Malformed signer key material still fails closed with deterministic runtime/config errors
- [ ] A regression test fails if implementation reverts to `eq_ignore_ascii_case()`

## Files to touch
- `crates/kamn-node/src/signer/managed_backend.rs`
- `crates/kamn-node/tests/managed_signer_provenance_constant_time.rs`
- `specs/6584-harden-managed-signer-provenance-compare.md`

## Error semantics
- Preserve current mismatch and malformed reason-code markers exactly
- Do not log inside provenance helper internals
- Fail closed on malformed or mismatched signer key material

## Test plan
- Add an external source-contract regression that rejects `eq_ignore_ascii_case()` in the provenance compare
- Add runtime tests covering equivalent-case acceptance, signer-key mismatch, and malformed signer-key failure semantics
- Run `cargo test -p kamn-node managed_signer_provenance -- --nocapture`
- Run `cargo clippy -p kamn-node --tests -- -D warnings`

## Integration verification
- Real integrated path remains `sign_kolme_live_managed_external_message(...)` via managed signer command execution and provenance verification
- Verified through public runtime-facing signing path, not a mock-only bypass

## Verification actuals
- Pending

## Deviations
- None
