# Direct-Message Encryption Path (Issue #124)

This document defines the direct-message cryptography contract used by
`DirectMessageCryptoEngine`.

## Core Types
- `DirectMessageCryptoEngine`: sender/recipient key-agreement context with nonce tracking.
- `DirectMessageCiphertext`: sealed payload envelope (`nonce`, `ciphertext`, `auth_tag`, algorithm labels, key refs).
- `DirectMessageCryptoError`: typed error model for validation, key-provisioning, integrity, and nonce replay failures.

## Required Configuration
- `KAMN_KEY_AGREEMENT_MASTER_SEED_HEX` must be set to 32-byte hex.
- The engine derives per-key-reference X25519 private keys from the master seed and key reference.

## Behavior
- `new(sender_key_ref, recipient_key_ref)`
  - validates non-empty key references containing `#key-agreement`.
  - derives an X25519 shared secret.
  - derives an XChaCha20-Poly1305 key from the shared secret.
- `encrypt(plaintext, nonce)`
  - rejects empty payloads.
  - enforces positive, non-reused nonce values.
  - encrypts via XChaCha20-Poly1305 and emits ciphertext + auth tag.
- `decrypt(ciphertext)`
  - verifies algorithm labels.
  - verifies integrity via XChaCha20-Poly1305 authentication.
  - rejects malformed/tampered payloads.

## Validation and Safety Rules
- Nonce reuse is blocked with `DirectMessageCryptoError::NonceReuse`.
- Missing/invalid key-agreement seed fails closed.
- Tampered ciphertext or auth tag returns `IntegrityCheckFailed`.
- Algorithm label mismatch returns `AlgorithmMismatch`.
- Invalid ciphertext encoding returns `InvalidCiphertextEncoding`.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
cargo test -p kamn-core --test issue_5921_production_message_crypto
cargo test -p kamn-core --test direct_message_encryption
```
