# Issue 6227 Spec

Status: Implemented
Priority: P0
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
`kamn-core` currently derives direct-message and group-message AEAD keys by hashing shared secrets with bare `SHA-256` plus ad-hoc context concatenation. This is a legacy KDF path and should be replaced with `HKDF-SHA256` while preserving decrypt compatibility for existing ciphertext.

## Scope
In scope:
- Replace SHA-256 AEAD key derivation in direct-message crypto with HKDF-SHA256.
- Replace SHA-256 AEAD key derivation in group-channel crypto with HKDF-SHA256.
- Preserve backward compatibility by allowing decrypt of legacy ciphertext derived with v1 SHA-256 KDF.
- Add deterministic tests for HKDF derivation and legacy decrypt compatibility.

Out of scope:
- Replacing SHA-256 usage for non-KDF concerns (nonce derivation, signatures, content hashing, audit chains).
- Wire-format changes requiring new ciphertext schema fields.

## Acceptance Criteria
- AC-1: Direct-message AEAD key derivation uses HKDF-SHA256 instead of ad-hoc SHA-256 digest derivation.
- AC-2: Group-channel AEAD key derivation uses HKDF-SHA256 instead of ad-hoc SHA-256 digest derivation.
- AC-3: Legacy ciphertext created with prior SHA-256 KDF remains decryptable (explicit compatibility behavior).
- AC-4: New/updated tests verify HKDF determinism and legacy compatibility for direct and group flows.
- AC-5: Targeted verification passes for `kamn-core` crypto suites.

## Conformance Cases
- C-01 (AC-1, Conformance): direct-message key derivation path calls HKDF-SHA256 implementation.
- C-02 (AC-2, Conformance): group-channel key derivation path calls HKDF-SHA256 implementation.
- C-03 (AC-3, Regression): direct-message decrypt accepts legacy v1 SHA-256-KDF ciphertext.
- C-04 (AC-3, Regression): group-message decrypt accepts legacy v1 SHA-256-KDF ciphertext.
- C-05 (AC-4, Unit): HKDF derivation tests are deterministic and fail closed.
- C-06 (AC-5, Functional): `cargo test -p kamn-core direct_message_crypto::tests` passes.
- C-07 (AC-5, Functional): `cargo test -p kamn-core group_channel_crypto::tests` passes.

## Success Metrics
- No new ad-hoc SHA-256 KDF usage remains in direct/group AEAD derivation.
- Existing direct/group decrypt behavior remains backward-compatible for legacy ciphertext.
