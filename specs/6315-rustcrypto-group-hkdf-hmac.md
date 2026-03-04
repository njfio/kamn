# Spec: Issue #6315 - replace manual group-channel HMAC/HKDF helpers

## Objective

Remove bespoke HMAC/HKDF helper implementation from
`crates/kamn-core/src/group_channel_crypto.rs` and route group-message v2 AEAD key derivation
through RustCrypto-backed HKDF/HMAC semantics exposed via shared crypto surface.

## Inputs/Outputs

- Inputs:
  - manual `hkdf_sha256_derive_32` and `hmac_sha256` helpers in
    `crates/kamn-core/src/group_channel_crypto.rs`.
  - existing RustCrypto-backed HKDF usage in `crates/kamn-crypto/src/direct_message_crypto.rs`.
  - existing group-channel compatibility tests for HKDF-v2 determinism and legacy-v1 decrypt fallback.
- Outputs:
  - no manual HMAC/HKDF primitive code in `group_channel_crypto.rs`.
  - reusable RustCrypto HKDF-SHA256 helper surface in `kamn-crypto` consumed by group-channel derivation.
  - contract test coverage asserting manual helper symbols are absent from group-channel source.

## Boundaries/Non-goals

- In scope:
  - key derivation path used by `derive_group_aead_key`.
  - wiring from `kamn-core` group-channel crypto to shared RustCrypto-backed helper surface.
  - deterministic and compatibility test coverage updates needed by this refactor.
- Out of scope:
  - ciphertext wire format changes.
  - signature format or nonce format changes.
  - removing legacy-v1 decrypt compatibility path.
  - shell/workflow/template/config surface changes.

## Failure modes

- FM-1: manual HMAC/HKDF helper code remains reachable in `group_channel_crypto.rs`.
- FM-2: HKDF derive failure is silently ignored instead of failing closed.
- FM-3: group HKDF-v2 derivation output drifts unexpectedly and breaks decrypt roundtrip.
- FM-4: legacy-v1 decrypt compatibility regresses.
- FM-5: helper wiring introduces compile-time visibility errors between `kamn-core` and `kamn-crypto`.

## Acceptance criteria (testable booleans)

- AC-1: `group_channel_crypto.rs` no longer defines `fn hkdf_sha256_derive_32(`.
- AC-2: `group_channel_crypto.rs` no longer defines `fn hmac_sha256(`.
- AC-3: group v2 AEAD derivation uses RustCrypto-backed HKDF helper through shared crypto surface.
- AC-4: `cargo test -p kamn-core group_message_hkdf_derivation_is_deterministic_and_distinct_from_legacy_v1 -- --exact` passes.
- AC-5: `cargo test -p kamn-core decrypt_accepts_legacy_v1_sha256_kdf_ciphertext_for_compatibility -- --exact` passes.
- AC-6: added source-surface contract test fails if manual helper symbols are reintroduced.

## Files to touch

- `specs/6315-rustcrypto-group-hkdf-hmac.md`
- `crates/kamn-crypto/src/lib.rs`
- `crates/kamn-crypto/src/hkdf_sha256.rs` (new)
- `crates/kamn-core/src/group_channel_crypto.rs`

## Error semantics

- Group-channel encryption/decryption remains fail-closed.
- If HKDF expansion fails, return `GroupChannelCryptoError::KeyDerivationFailed`.
- Do not introduce silent fallback to manual derivation.
- Preserve existing error display strings for unchanged failure paths.

## Test plan

- RED:
  - add/extend group-channel contract tests that assert manual helper symbols are absent and
    RustCrypto backend markers are present.
  - run targeted test(s) to confirm failure before implementation.
- GREEN:
  - implement shared HKDF-SHA256 helper in `kamn-crypto`.
  - rewire `derive_group_aead_key` to use shared helper and map failures to
    `GroupChannelCryptoError::KeyDerivationFailed`.
- REFACTOR:
  - remove obsolete manual helper code and keep helper APIs focused and small.
- INTEGRATION:
  - run targeted group-channel tests and full `kamn-core` tests to ensure no behavior drift.

## Phase 6 integration evidence

- `cargo clippy -p kamn-crypto --all-targets --all-features -- -D warnings`:
  - pass
- `cargo clippy -p kamn-core --all-targets --all-features -- -D warnings`:
  - pass
- `cargo test -p kamn-crypto --test direct_message_crypto_integration`:
  - pass (`4 passed, 0 failed`)
- `cargo test -p kamn-core --test issue_5921_production_message_crypto`:
  - pass (`17 passed, 0 failed`)
- `cargo test -p kamn-crypto`:
  - pass (`23 unit + 4 integration passed, 0 failed`)
- `cargo test -p kamn-core`:
  - pass (all suites completed with `0 failed`)

## Deviations

- None.
