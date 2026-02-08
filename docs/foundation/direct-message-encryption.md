# Direct-Message Encryption Path (Issue #124)

This document defines the first implementation slice for 1:1 direct-message
encryption controls aligned to `X25519` + `XChaCha20-Poly1305` protocol labels.

## Core Types
- `DirectMessageCryptoEngine`: sender/recipient key-agreement context with nonce tracking.
- `DirectMessageCiphertext`: sealed payload envelope (`nonce`, `ciphertext`, `auth_tag`, algorithm labels, key refs).
- `DirectMessageCryptoError`: typed error model for validation, integrity, and nonce replay failures.

## Behavior
- `new(sender_key_ref, recipient_key_ref)`
  - validates non-empty key references containing `#key-agreement`.
- `encrypt(plaintext, nonce)`
  - rejects empty payloads.
  - enforces positive, non-reused nonce values.
  - emits deterministic ciphertext + auth tag.
- `decrypt(ciphertext)`
  - verifies algorithm labels.
  - verifies integrity via deterministic auth tag.
  - rejects malformed/tampered payloads.

## Validation and Safety Rules
- Nonce reuse is blocked with `DirectMessageCryptoError::NonceReuse`.
- Tampered ciphertext or auth tag returns `IntegrityCheckFailed`.
- Algorithm label mismatch returns `AlgorithmMismatch`.
- Invalid ciphertext encoding returns `InvalidCiphertextEncoding`.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test direct_message_encryption
cargo test -p kamn-core
```

## Notes
This slice intentionally uses deterministic, dependency-free authenticated
encryption scaffolding for fast and low-cost CI, while preserving the
protocol-level `X25519` / `XChaCha20-Poly1305` interface contract.
