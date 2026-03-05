# Spec: Issue 6463 - Harden DirectMessageCryptoEngine memory hygiene and debug surface

## Objective
Harden `DirectMessageCryptoEngine` against accidental secret exposure by removing cloning, replacing derived debug output with a redacted custom formatter, and zeroizing key buffers on drop.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-crypto/src/direct_message_crypto.rs`
  - `crates/kamn-core/src/direct_message_crypto.rs` facade
- Outputs:
  - `DirectMessageCryptoEngine` no longer derives/implements `Clone`.
  - Custom `Debug` impl for engine that reports non-sensitive metadata only.
  - `Drop` impl that zeroizes `aead_key` and `legacy_aead_key`.
  - Updated tests/contracts validating no-clone/debug/drop zeroize markers.

## Boundaries/Non-goals
- No cryptographic algorithm changes.
- No ciphertext format changes.
- No legacy-v1 decrypt compatibility removal.

## Failure modes
- `Clone` still available through derive or manual impl.
- Debug output includes key material bytes.
- Drop path does not zeroize key buffers.
- Facade (`kamn-core`) still advertises clone semantics while inner engine is non-clone.

## Acceptance criteria (testable booleans)
- [x] AC-1: `DirectMessageCryptoEngine` in `kamn-crypto` does not implement `Clone`.
- [x] AC-2: `Debug` output for engine omits raw key bytes and includes only safe metadata.
- [x] AC-3: engine `Drop` impl zeroizes `aead_key` and `legacy_aead_key`.
- [x] AC-4: `cargo test -p kamn-crypto --test direct_message_crypto_integration` passes.
- [x] AC-5: `cargo test -p kamn-crypto` and `cargo test -p kamn-core --test issue_5921_production_message_crypto` pass.

## Files to touch
- `specs/6463-harden-direct-message-crypto-engine-memory-hygiene.md`
- `crates/kamn-crypto/Cargo.toml`
- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `crates/kamn-core/src/direct_message_crypto.rs`

## Error semantics
- Preserve existing construction/encrypt/decrypt error values and display messages.
- Preserve existing nonce reuse and integrity-failure behavior.

## Test plan
- Red:
  - Add/adjust direct-message contract tests asserting source/debug/drop markers and remove clone expectations.
  - Verify red failure before implementation wiring.
- Green:
  - Implement non-clone engine, custom debug, and drop zeroization.
  - Align `kamn-core` wrapper derives with non-clone inner engine.
- Refactor:
  - Keep debug output stable and intentionally minimal.
- Integration:
  - `cargo test -p kamn-crypto --test direct_message_crypto_integration`
  - `cargo test -p kamn-crypto`
  - `cargo test -p kamn-core --test issue_5921_production_message_crypto`

## Phase 6 integration evidence
- Wrapper integration (`kamn-core`) remains wired to `kamn-crypto` facade; clone derive removed on
  wrapper to match hardened inner type semantics.
- Verified commands:
  - `cargo test -p kamn-crypto spec_c09_direct_message_engine_source_contract_enforces_non_clone_redacted_debug_and_drop_zeroize -- --nocapture`
  - `cargo test -p kamn-crypto spec_c10_direct_message_engine_debug_output_redacts_sensitive_key_material -- --nocapture`
  - `cargo test -p kamn-crypto --test direct_message_crypto_integration`
  - `cargo test -p kamn-crypto`
  - `cargo test -p kamn-core --test issue_5921_production_message_crypto`

## Deviations
- None.
