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
- [x] `verify_kolme_live_managed_signer_backend_signature_provenance()` no longer uses `eq_ignore_ascii_case()` for signer public-key provenance matching
- [x] Case-variant-equivalent valid hex still verifies successfully
- [x] Mismatched signer public key still returns the existing `managed_signer_backend_response_provenance_mismatch` error
- [x] Malformed signer key material still fails closed with deterministic runtime/config errors
- [x] A regression test fails if implementation reverts to `eq_ignore_ascii_case()`

## Files to touch
- `crates/kamn-node/src/signer/managed_backend.rs`
- `crates/kamn-node/tests/managed_signer_provenance_constant_time.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`
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
- Verified workspace inventory parity after adding the dedicated external contract test target

## Verification actuals
- Red: `cargo test -p kamn-node --test managed_signer_provenance_constant_time -- --nocapture`
  - failed because `verify_kolme_live_managed_signer_backend_signature_provenance()` still used `eq_ignore_ascii_case()`
- Green: `cargo test -p kamn-node --test managed_signer_provenance_constant_time -- --nocapture`
- Green: `cargo test -p kamn-node --bin kamn-node main_tests::signer_tests::regression_kolme_live_managed_external_backend_response_accepts_case_variant_signer_public_key -- --exact --nocapture`
- Green: `cargo test -p kamn-node --bin kamn-node main_tests::signer_tests::regression_kolme_live_managed_external_backend_response_rejects_malformed_signer_public_key -- --exact --nocapture`
- Green: `cargo test -p kamn-node --bin kamn-node main_tests::signer_tests::regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch -- --exact --nocapture`
- Green: `cargo test -p kamn-node --bin kamn-node main_tests::signer_tests::integration_kolme_live_managed_external_adapter_provenance_consumed_by_signer_selection -- --exact --nocapture`
- Green: `cargo clippy -p kamn-node --tests -- -D warnings`
- Green: `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`

## Deviations
- Adding the dedicated external node contract test increased workspace test inventory by one file, so `fixtures/ci/test_file_size_policy_baseline.env` was refreshed from `461` to `462`.
