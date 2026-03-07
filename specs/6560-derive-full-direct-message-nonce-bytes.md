# Spec: Issue #6560 - Derive full direct-message nonce bytes without raw counter prefix

## Objective

Harden direct-message XChaCha20 nonce construction so encryption no longer embeds the raw
`u64` nonce counter in the first 8 bytes of the 24-byte nonce material, while preserving
decrypt compatibility for ciphertext produced by the existing layout.

## Inputs/Outputs

- Inputs:
  - `sender_key_ref: &str`
  - `recipient_key_ref: &str`
  - `nonce: u64`
  - plaintext/ciphertext inputs already accepted by `DirectMessageCryptoEngine`
- Outputs:
  - encryption returns `DirectMessageCiphertext` with unchanged public fields
  - decryption returns plaintext for both new-layout ciphertext and ciphertext produced by
    the previous nonce-byte derivation

## Boundaries/Non-goals

- Do not change the public shape of `DirectMessageCiphertext`.
- Do not change the existing legacy SHA-256-v1 key compatibility policy.
- Do not change nonce validation (`0` invalid) or nonce reuse tracking semantics.
- Do not change DID, task, escrow, or service API behavior.

## Failure modes

- zero nonce still fails with `DirectMessageCryptoError::InvalidNonce(0)`
- repeated nonce on the same engine instance still fails with
  `DirectMessageCryptoError::NonceReuse(<nonce>)`
- decrypt still fails closed with `DirectMessageCryptoError::IntegrityCheckFailed` when both
  current and compatibility nonce layouts fail authentication
- malformed ciphertext encoding still fails with existing typed errors

## Acceptance criteria

- [ ] Encryption derives all 24 nonce bytes without copying the raw counter prefix directly
      into the output nonce bytes.
- [ ] Ciphertext encrypted after this change decrypts successfully through the normal engine
      path.
- [ ] Ciphertext encrypted with the previous nonce-byte layout still decrypts successfully.
- [ ] Zero-nonce rejection and nonce-reuse rejection remain unchanged.
- [ ] Targeted Rust tests cover new-layout encryption behavior and legacy-layout decrypt
      compatibility.

## Files to touch

- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `crates/kamn-crypto/tests/direct_message_crypto_input_nonce_failures.rs`
- `crates/kamn-crypto/tests/direct_message_crypto_failure_paths.rs`
- `specs/6560-derive-full-direct-message-nonce-bytes.md`

## Error semantics

- Preserve existing typed `DirectMessageCryptoError` variants.
- Interior crypto helpers return typed errors and do not log.
- Decrypt compatibility fallback may try multiple nonce-byte layouts internally, but it must
  surface a single `IntegrityCheckFailed` result if none authenticate successfully.

## Test plan

1. Add RED tests that prove the derived nonce bytes no longer expose `nonce.to_le_bytes()`
   as the first 8 bytes.
2. Add RED tests that decrypt legacy-layout ciphertext through the updated engine.
3. Keep explicit regression coverage for zero nonce and nonce reuse.
4. Run targeted `kamn-crypto` tests covering the new behavior and existing failure paths.

## Deviations

- The external integration coverage was initially added inline to
  `crates/kamn-crypto/tests/direct_message_crypto_integration.rs`, but that pushed the file to
  247 lines and violated the repo file-size policy. The helper bulk was moved into
  `crates/kamn-crypto/tests/support/direct_message_crypto_support.rs`, which keeps the public
  integration target under budget and does not affect the tracked test-file inventory because
  `/tests/support/` is excluded from `test_file_size_policy`.

## Verification

- RED:
  - `cargo test -p kamn-crypto direct_message_nonce_bytes_do_not_expose_raw_counter_prefix -- --nocapture`
  - `cargo test -p kamn-crypto encrypt_output_does_not_authenticate_under_legacy_raw_prefix_nonce_layout -- --nocapture`
- GREEN / refactor / integration:
  - `cargo test -p kamn-crypto -- --nocapture`
  - `cargo test -p kamn-crypto --test direct_message_crypto_integration -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
  - `cargo clippy -p kamn-crypto --tests -- -D warnings`

## Outcome

- Encryption now derives all 24 XChaCha20 nonce bytes from the v2 hash material rather than
  exposing the raw counter prefix.
- Decrypt accepts both the new v2 nonce layout and the previous raw-prefix layout, across both
  current HKDF and legacy SHA-256-v1 key compatibility paths.
- Zero nonce and nonce reuse semantics remain unchanged.
