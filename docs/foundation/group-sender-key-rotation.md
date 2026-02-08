# Group Sender-Key Distribution and Rotation (Issues #226, #227)

This document captures the first implementation slice for group-channel sender-key distribution and key rotation behavior.

## Scope Delivered
- Added `crates/kamn-core/src/group_channel_crypto.rs` with:
  - `GroupChannelCryptoEngine` for sender-key distribution, rotation, and group encrypt/decrypt.
  - `SenderKeyDistributionRecord` with per-sender key generations and active generation tracking.
  - `GroupMessageCiphertext` with deterministic auth-tag and sender-signature fields.
  - typed errors via `GroupChannelCryptoError` for authorization, generation, integrity, and validation failures.
- Added integration tests in `crates/kamn-core/tests/group_sender_keys.rs`.

## Behavior Rules
- Sender-key distribution:
  - sender DID and recipient DIDs must be valid `kamn:did:agent:*`.
  - sender key ref must include `#sender-key-`.
  - recipient allowlist must be non-empty.
- Rotation:
  - each distribution for the same sender increments `key_generation`.
  - previous sender generation is marked inactive.
- Encryption/decryption:
  - only allowlisted recipients can decrypt.
  - algorithm, channel id, signature, and auth-tag are validated.
  - nonce reuse for `(sender, generation, nonce)` is blocked.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test group_sender_keys
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
