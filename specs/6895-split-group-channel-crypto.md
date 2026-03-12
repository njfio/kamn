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
- [x] `crates/kamn-core/src/group_channel_crypto.rs` becomes a thin root shell under the active file-size policy
- [x] concern-based sibling modules are introduced for the real seams in the file
- [x] no touched extracted file exceeds the active touched-Rust size policy
- [x] a hard-fail extraction contract exists and passes
- [x] existing `group_channel_crypto` tests still pass
- [x] touched-Rust size policy returns `GO` on the final branch

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

## Final evidence
- `cargo test -p kamn-core --test group_channel_crypto_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core group_channel_crypto::tests:: --lib -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6895-touched-size-refactor.json`
- touched-Rust result: `policy_decision=GO`

## Deviations
- Clean-clone verification was not needed because `/home/n/Code/kamn` is now on a clean current `main` baseline.
