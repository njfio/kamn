# 6895-split-group-channel-crypto

## Objective
Split `crates/kamn-core/src/group_channel_crypto.rs` into bounded concern-based modules while preserving current sender-key distribution, rotation, encryption, decryption, validation, and test behavior.

## Inputs/Outputs
- Input: current `group_channel_crypto.rs` production logic and inline tests
- Output: a thin root module plus bounded sibling modules for models/errors, key distribution lifecycle, crypto/derivation helpers, validation/helpers, and tests
- Output: a hard-fail extraction contract that enforces the new layout

## Boundaries/Non-goals
- Do not change cryptographic algorithms, wire formats, or persistence semantics
- Do not redesign adjacent crypto modules outside `group_channel_crypto.rs`
- Do not weaken or delete existing behavioral tests to satisfy the split

## Failure modes
- Extraction contract does not fail when the root file remains oversized
- Module split changes encryption/decryption behavior
- Sender-key generation/rotation semantics drift
- DID validation or recipient allowlist validation changes silently
- Final branch still fails touched-Rust size policy

## Acceptance criteria
- [ ] `crates/kamn-core/src/group_channel_crypto.rs` becomes a thin root shell under the active file-size policy
- [ ] concern-based sibling modules are introduced for the real seams in the file
- [ ] no touched extracted file exceeds the active touched-Rust size policy
- [ ] a hard-fail extraction contract exists and passes
- [ ] existing `group_channel_crypto` tests still pass
- [ ] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/group_channel_crypto.rs`
- `crates/kamn-core/src/group_channel_crypto/`
- `crates/kamn-core/tests/group_channel_crypto_module_extraction_contract.rs`
- `specs/6895-split-group-channel-crypto.md`

## Error semantics
- Preserve existing `GroupChannelCryptoError` behavior and error messages unless extraction makes message text clearer without changing meaning
- No silent fallback paths may be introduced during extraction
- Validation remains fail-closed at function boundaries

## Test plan
- Add a red extraction contract that fails while the root file is still oversized and the expected module layout is missing
- Run the extraction contract target
- Run the existing `group_channel_crypto` tests after the split
- Run touched-Rust size policy on the final branch
