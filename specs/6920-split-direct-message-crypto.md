# 6920-split-direct-message-crypto

## Objective
Split `crates/kamn-crypto/src/direct_message_crypto.rs` into bounded concern-based modules while preserving direct-message crypto behavior, public surface, and existing tests.

## Inputs/Outputs
- Input: current `crates/kamn-crypto/src/direct_message_crypto.rs` production source and its existing tests.
- Output: a thin root shell delegating to bounded sibling modules for extracted direct-message crypto concerns.
- Output: a hard-fail extraction contract that enforces the root shell budget and extracted module layout.

## Boundaries/Non-goals
- Do not change direct-message crypto behavior or public API semantics.
- Do not add dependencies.
- Do not weaken or delete tests to make the split pass.
- Do not move unrelated crypto modules in this issue.

## Failure modes
- The root file remains oversized while the extraction contract passes.
- Extracted modules drift behavior or public surface.
- Tests compile but no real crypto path still exercises the split code.
- Any touched file or function fails the touched-Rust size policy.

## Acceptance criteria
- [ ] `crates/kamn-crypto/src/direct_message_crypto.rs` becomes a thin root shell under the active file-size policy.
- [ ] Bounded sibling modules exist for the extracted direct-message crypto seams.
- [ ] Existing direct-message crypto behavior remains unchanged after the split.
- [ ] A hard-fail extraction contract exists and passes.
- [ ] Real direct-message crypto targets still pass after the split.
- [ ] Touched-Rust size policy returns `GO` on the final branch.

## Files to touch
- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `crates/kamn-crypto/src/direct_message_crypto/`
- `crates/kamn-crypto/tests/direct_message_crypto_module_extraction_contract.rs`
- `specs/6920-split-direct-message-crypto.md`

## Error semantics
- Existing hard-fail direct-message crypto behavior remains explicit and unchanged.
- No silent fallbacks or swallowed failures are introduced.
- Extracted seams preserve current error propagation and caller-visible behavior.

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing.
- Run the extraction contract target and confirm red.
- Run the real direct-message crypto target after extraction.
- Run touched-Rust size policy on the final branch.

## Evidence
- `cargo test -p kamn-crypto --test direct_message_crypto_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-crypto direct_message_crypto --lib -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6920-touched-size-post-refactor.json`

## Deviations
- None.
