# Spec: Issue #6297 - kamn-crypto HKDF/HMAC crate migration

## Objective

Replace hand-rolled HKDF/HMAC logic in `kamn-crypto` direct-message key derivation with
crate-backed `hkdf` + `hmac` primitives while preserving existing behavior.

## Inputs/Outputs

- Inputs:
  - HKDF salt (`DIRECT_MESSAGE_AEAD_KDF_SALT_V2`)
  - IKM (`shared_secret`)
  - HKDF info (`DIRECT_MESSAGE_AEAD_KDF_INFO_V2`)
- Outputs:
  - same deterministic 32-byte AEAD key output as current logic for existing fixtures/tests.

## Boundaries/Non-goals

- In scope:
  - `kamn-crypto` direct-message derivation internals.
  - Add explicit backend marker constants and tests.
  - Remove local hand-rolled HKDF/HMAC helpers.
- Out of scope:
  - `kamn-core` group-channel migration.
  - nonce derivation changes.
  - wire format changes.

## Failure Modes

- FM-1: derivation output changes and breaks decrypt compatibility tests.
- FM-2: migration leaves dead manual HKDF/HMAC helpers in source.
- FM-3: backend provenance is unclear for future audits.

## Acceptance Criteria

- AC-1: direct-message key derivation path uses `hkdf::Hkdf::<sha2::Sha256>`.
- AC-2: no local `hkdf_sha256_derive_32` / `hmac_sha256` helper remains in
  `direct_message_crypto.rs`.
- AC-3: backend marker constants exist and are asserted in tests.
- AC-4: existing direct-message unit + integration tests remain green.

## Files To Touch

- `crates/kamn-crypto/Cargo.toml`
- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `specs/6297-crypto-hkdf-hmac-crate-migration.md`

## Error Semantics

- HKDF expansion failure maps deterministically to existing
  `DirectMessageCryptoError::EncryptionFailed` in derivation path (should be unreachable for fixed
  32-byte output; still fail-closed).
- No silent fallback to hand-rolled derivation.

## Test Plan

- RED:
  - Add tests expecting backend marker constants and absence of manual helper symbols in source.
- GREEN:
  - Add `hkdf`/`hmac` dependencies and migrate derivation implementation.
  - remove manual helper functions.
- REFACTOR:
  - keep derivation helper concise with explicit failure mapping.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-crypto --tests -- -D warnings`
  - `cargo test -p kamn-crypto`
