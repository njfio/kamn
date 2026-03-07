# 6558 Add External Direct-Message Crypto Input and Nonce Failure Coverage

## Objective
Add dedicated external `kamn-crypto` regression coverage for direct-message crypto input and nonce failure paths so callers are protected by stable `tests/` targets rather than only inline source tests.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-crypto/src/direct_message_crypto.rs`
  - existing external tests in `crates/kamn-crypto/tests/`
  - issue `#6558`
- Outputs:
  - a dedicated external integration test target covering empty payload, zero nonce, and nonce reuse failures
  - a contract test pinning the dedicated target

## Boundaries/Non-goals
- No crypto algorithm changes
- No public API changes
- No compatibility-policy changes for legacy decrypt support
- No weakening of existing inline unit tests

## Failure modes
- external tests fail to pin `EmptyPayload` on encrypt
- external tests fail to pin zero-nonce rejection on encrypt
- external tests fail to pin zero-nonce rejection on decrypt
- external tests fail to pin nonce reuse rejection on encrypt
- the dedicated external target can disappear without a contract failure

## Acceptance criteria
- [ ] external test target covers `encrypt("", nonce)` returning `DirectMessageCryptoError::EmptyPayload`
- [ ] external test target covers `encrypt(payload, 0)` returning `DirectMessageCryptoError::InvalidNonce(0)`
- [ ] external test target covers `decrypt(sealed_with_nonce_0)` returning `DirectMessageCryptoError::InvalidNonce(0)`
- [ ] external test target covers encrypting twice with the same nonce returning `DirectMessageCryptoError::NonceReuse(nonce)` on the second call
- [ ] contract test pins the dedicated external failure-path target by name/markers
- [ ] `cargo test -p kamn-crypto --test direct_message_crypto_input_nonce_failures -- --nocapture` passes
- [ ] `cargo test -p kamn-crypto --test direct_message_crypto_input_nonce_failures_contract -- --nocapture` passes

## Files to touch
- `specs/6558-add-external-direct-message-crypto-input-and-nonce-failure-coverage.md`
- `crates/kamn-crypto/tests/direct_message_crypto_input_nonce_failures.rs`
- `crates/kamn-crypto/tests/direct_message_crypto_input_nonce_failures_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error semantics
- coverage must assert exact typed `DirectMessageCryptoError` variants; no substring-only assertions
- failures remain hard-fail test assertions
- no silent fallback or normalization is allowed in test expectations

## Test plan
1. Add a red contract test that requires the dedicated external target and marker test names.
2. Add the external failure-path target with the required input/nonce scenarios.
3. Run the new contract target and failure-path target.
4. Refresh the workspace test-file inventory baseline if the new targets change the tracked count.
5. Re-run the targeted tests and the file-inventory test.

## Integration evidence
- The dedicated external target is now wired into the real workspace test surface under `crates/kamn-crypto/tests/`.
- The workspace inventory gate was refreshed to account for the two new test targets.
- Verified green commands:
  - `cargo test -p kamn-crypto --test direct_message_crypto_input_nonce_failures_contract -- --nocapture`
  - `cargo test -p kamn-crypto --test direct_message_crypto_input_nonce_failures -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`

## Deviations
- None.
