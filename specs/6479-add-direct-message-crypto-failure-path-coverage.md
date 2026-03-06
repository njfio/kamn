# Spec: Issue 6479 - Add direct-message crypto failure-path coverage

## Objective
Add focused `kamn-crypto` test coverage for the remaining direct-message
failure-path branches that are not already exercised, while protecting the
defensive `EncryptionFailed` and `KeyDerivationFailed` mappings with explicit
source-contract tests.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-crypto/src/direct_message_crypto.rs`
- Outputs:
  - Dedicated `kamn-crypto` failure-path test target for direct-message crypto.
  - Public API coverage for malformed ciphertext encoding and invalid UTF-8
    decrypt output branches.
  - Source-contract assertions for defensive `EncryptionFailed` and
    `KeyDerivationFailed` error mappings.

## Boundaries/Non-goals
- No direct-message crypto behavior changes.
- No HKDF implementation changes.
- No AEAD implementation changes.
- No broader crypto crate refactor.

## Failure modes
- Malformed ciphertext hex still lacks direct coverage through the public API.
- Invalid UTF-8 decrypt output is not asserted to map to
  `InvalidCiphertextEncoding`.
- Defensive `EncryptionFailed` / `KeyDerivationFailed` mappings drift or are
  removed without test protection.

## Acceptance criteria (testable booleans)
- [ ] AC-1: A dedicated `kamn-crypto` failure-path test target exists for
      direct-message crypto.
- [ ] AC-2: The target covers malformed ciphertext hex mapping to
      `InvalidCiphertextEncoding`.
- [ ] AC-3: The target covers malformed auth-tag hex mapping to
      `IntegrityCheckFailed`.
- [ ] AC-4: The target covers invalid UTF-8 decrypt output mapping to
      `InvalidCiphertextEncoding`.
- [ ] AC-5: The target enforces source-contract markers for defensive
      `EncryptionFailed` and `KeyDerivationFailed` mappings.
- [ ] AC-6: `cargo test -p kamn-crypto --test direct_message_crypto_failure_paths -- --nocapture`
      passes.

## Files to touch
- `specs/6479-add-direct-message-crypto-failure-path-coverage.md`
- `crates/kamn-crypto/tests/direct_message_crypto_failure_paths_contract.rs`
- `crates/kamn-crypto/tests/direct_message_crypto_failure_paths.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error semantics
- Preserve current `DirectMessageCryptoError` behavior exactly.
- Defensive branches that are not reproducible through the current public API
  must still be guarded by source-contract tests instead of synthetic behavior
  changes.

## Test plan
- Red:
  - Add a contract target that requires the dedicated failure-path test file and
    named test markers.
  - Confirm the contract target fails before the new test file exists.
- Green:
  - Add the dedicated failure-path test file with the required public API and
    source-contract assertions.
- Refactor:
  - Keep fixtures local and avoid duplicating full engine setup.
- Integration:
  - Run the dedicated failure-path test target.

## Phase 6 integration evidence
- Pending.

## Deviations
- Refreshed `fixtures/ci/test_file_size_policy_baseline.env` because adding the
  dedicated contract and failure-path targets increased the repository test-file
  inventory from 432 to 434 without changing oversized-file counts.
