# 6568 Harden Group Channel Crypto Engine

## Objective
Bring `GroupChannelCryptoEngine` up to the same security-hardening baseline as the direct-message crypto engine by removing `Clone`, adding redacted `Debug`, zeroizing/clearing in-memory key material state on drop, and deriving all 24 group nonce bytes without exposing the raw counter prefix.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/group_channel_crypto.rs`
  - `crates/kamn-core/Cargo.toml`
- Outputs:
  - hardened `GroupChannelCryptoEngine` source contract
  - compatibility-preserving group decrypt path for legacy raw-prefix nonce ciphertext
  - targeted regression coverage for nonce derivation and source hardening markers

## Boundaries/Non-goals
- Do not change the public `GroupMessageCiphertext` fields.
- Do not change direct-message crypto code.
- Do not change non-group encryption behavior outside `group_channel_crypto.rs`.
- Do not introduce a new external dependency beyond adding existing workspace `zeroize` to `kamn-core`.

## Failure modes
- `GroupChannelCryptoEngine` remains cloneable.
- `Debug` exposes internal state instead of a redacted summary.
- sensitive engine state is retained on drop.
- new group encryptions continue to expose `nonce.to_le_bytes()` in the XChaCha20 nonce prefix.
- decrypt stops accepting legacy raw-prefix group ciphertext.
- source contracts fail to detect a regression back to the insecure nonce layout or missing `Drop`/custom `Debug`.

## Acceptance criteria
- [ ] `GroupChannelCryptoEngine` no longer derives `Clone`.
- [ ] `GroupChannelCryptoEngine` defines a custom redacted `Debug` implementation.
- [ ] `GroupChannelCryptoEngine` defines `Drop` that zeroizes/clears sensitive in-memory state.
- [ ] group encryption derives all 24 nonce bytes and no longer exposes `nonce.to_le_bytes()` as the prefix.
- [ ] decrypt continues to accept legacy raw-prefix nonce ciphertext generated under the old group layout.
- [ ] targeted tests fail closed if the old raw-prefix nonce layout, `Clone`, or missing `Drop`/redacted `Debug` return.

## Files to touch
- `crates/kamn-core/src/group_channel_crypto.rs`
- `crates/kamn-core/Cargo.toml`
- `specs/6568-harden-group-channel-crypto-engine.md`

## Error semantics
- encryption still fails with `InvalidNonce`, `NonceReuse`, `EncryptionFailed`, and `KeyDerivationFailed` under the same conditions
- decrypt still fails closed with `IntegrityCheckFailed`, `SignatureMismatch`, `InvalidCiphertextEncoding`, and `AlgorithmMismatch`
- legacy raw-prefix compatibility applies only at decrypt time; encrypt must emit only the hardened nonce layout

## Test plan
- Red:
  - extend `group_channel_crypto.rs` tests to require non-`Clone`, redacted `Debug`, `Drop`, and derived-only nonce bytes
  - add a regression that decrypt accepts legacy raw-prefix group ciphertext while current encrypt output does not authenticate under that legacy layout
- Green:
  - implement the minimum module changes to satisfy the new tests
- Integration:
  - rerun targeted `kamn-core` tests for `group_channel_crypto`
  - run strict `clippy` for `kamn-core` tests

## Shell-surface closure template
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: 0
- shell_to_rust_ratio_delta_actual: 0.0
- shell_surface_ratio_target_status: neutral
