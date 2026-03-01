# Spec: Issue #6285 - kamn-crypto integration contract + key-ref guard

## Objective

Add first-class integration coverage for `kamn-crypto` direct-message flow and make decrypt fail
closed with a typed error when ciphertext sender/recipient key references do not match engine
context.

## Inputs/Outputs

- Inputs:
  - `DirectMessageCryptoEngine` initialized with sender/recipient key refs.
  - `DirectMessageCiphertext` produced by `encrypt` or provided externally.
- Outputs:
  - Successful decrypt round-trip for matching key references.
  - Typed key-reference mismatch error for mismatched sender/recipient metadata.

## Boundaries/Non-goals

- In scope:
  - Add integration tests in `crates/kamn-crypto/tests/`.
  - Add explicit key-reference mismatch check in decrypt path.
  - Keep existing algorithm/integrity behavior unchanged.
- Out of scope:
  - New crypto algorithms or key derivation changes.
  - Cross-crate runtime wiring.

## Failure Modes

- FM-1: decrypt accepts ciphertext with mismatched sender/recipient key refs.
- FM-2: new guard alters successful round-trip behavior for valid ciphertext.
- FM-3: integration test harness depends on nondeterministic external state.

## Acceptance Criteria

- AC-1: `crates/kamn-crypto/tests/direct_message_crypto_integration.rs` exists and runs in CI.
- AC-2: decrypt returns a typed mismatch error before cryptographic verification when key refs
  differ from engine context.
- AC-3: integration tests cover:
  - successful encrypt/decrypt round-trip
  - sender key-ref mismatch rejection
  - recipient key-ref mismatch rejection
- AC-4: existing `kamn-crypto` test suite remains green.

## Files To Touch

- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `crates/kamn-crypto/tests/direct_message_crypto_integration.rs`
- `specs/6285-kamn-crypto-integration-keyref-guard.md`

## Error Semantics

- Add a typed error variant for key-reference mismatch in `DirectMessageCryptoError`.
- Decrypt returns this error deterministically when metadata refs differ from engine refs.
- Existing error semantics for algorithm mismatch, nonce validation, and integrity remain unchanged.

## Test Plan

- RED:
  - Add integration tests asserting typed key-reference mismatch error in sender/recipient mismatch
    cases.
  - Confirm tests fail before mismatch guard exists.
- GREEN:
  - Implement minimal decrypt guard + error variant.
- REFACTOR:
  - Keep mismatch validation as a small helper.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-crypto --tests -- -D warnings`
  - `cargo test -p kamn-crypto`
