# Issue 6227 Plan

## Approach
1. Add internal HKDF-SHA256 helper logic (extract+expand via HMAC-SHA256) in crypto modules using existing dependencies.
2. Switch direct-message AEAD key derivation to HKDF-SHA256 (`v2`) and retain explicit legacy `v1` derivation for decrypt fallback only.
3. Switch group-channel AEAD key derivation to HKDF-SHA256 (`v2`) and retain legacy `v1` derivation for decrypt fallback only.
4. Keep wire format stable; compatibility is handled by dual-key decrypt attempt (v2 first, legacy v1 fallback).
5. Add red/green regression tests to verify legacy ciphertext still decrypts and HKDF is deterministic.

## Affected Modules
- `crates/kamn-core/src/direct_message_crypto.rs`
- `crates/kamn-core/src/group_channel_crypto.rs`
- `specs/6227/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: Incorrect manual HKDF/HMAC implementation could introduce cryptographic defects.
  - Mitigation: Add deterministic tests, known-shape assertions, and keep implementation minimal RFC5869 semantics.
- Risk: Compatibility regression for existing ciphertext.
  - Mitigation: Explicit legacy fallback decrypt tests for both direct and group modules.
- Risk: Behavior ambiguity without wire-format versioning.
  - Mitigation: Document deterministic policy: encrypt with HKDF-v2; decrypt attempts v2 then v1 fallback.

## Interfaces
- Direct-message: `derive_direct_message_aead_key` switches to HKDF-v2; legacy helper retained for fallback.
- Group-channel: `derive_group_aead_key` switches to HKDF-v2; legacy helper retained for fallback.
