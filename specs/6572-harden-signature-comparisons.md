# 6572 Harden Signature Comparisons

## Objective
Replace direct equality checks in selected `kamn-core` signature verification paths with one internal constant-time comparison helper, preserving all current boolean outcomes and error semantics.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/lib.rs`
  - `crates/kamn-core/src/signature_profile.rs`
  - `crates/kamn-core/src/group_channel_crypto.rs`
  - `crates/kamn-core/src/instruction_verify.rs`
- Outputs:
  - one internal constant-time byte/string equality helper inside `kamn-core`
  - verification call sites updated to use that helper in scoped signature paths
  - regression coverage that fails if those paths revert to direct equality

## Boundaries/Non-goals
- Do not add dependencies.
- Do not change public API contracts or exported types.
- Do not replace every equality check across the workspace.
- Do not modify CI/workflows or governance policy.
- Do not change non-signature comparisons such as ordinary ids, labels, or docs markers.

## Failure modes
- targeted verification paths continue using direct `==` / `!=` signature comparison.
- helper returns true for mismatched lengths instead of failing closed.
- helper changes verification outcomes for existing equal/mismatched signature inputs.
- helper is introduced but one or more scoped call sites do not adopt it.

## Acceptance criteria
- [x] `signature_matches_supported_profile_for_fields()` uses the internal constant-time helper instead of direct signature equality.
- [x] `GroupChannelCryptoEngine::decrypt()` uses the same helper for expected-vs-provided signature comparison.
- [x] instruction verification uses the same helper for record-vs-claim signature comparison.
- [x] helper fails closed on length mismatch and preserves equality behavior for matching bytes.
- [x] regression tests fail if the scoped verification paths revert to direct equality.

## Files to touch
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/src/constant_time_eq.rs`
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-core/src/group_channel_crypto.rs`
- `crates/kamn-core/src/instruction_verify.rs`
- `specs/6572-harden-signature-comparisons.md`

## Error semantics
- existing signature verification functions keep their current return types and failure outcomes.
- length mismatch in the helper evaluates to `false`; no new error type is introduced.
- caller-visible errors and rejection reasons remain unchanged.

## Test plan
- Red:
  - add source-contract tests that fail if the targeted verification paths still use direct equality
  - add helper unit tests for equal bytes, mismatched bytes, and mismatched lengths
- Green:
  - add the minimal helper and switch only the scoped signature call sites
- Refactor:
  - keep helper small and internal; avoid duplication across call sites
- Integration:
  - run targeted `kamn-core` tests for signature profile, group crypto, and instruction verification

## Integration notes
- No new runtime entrypoint was introduced. The integration boundary for this issue is the existing public verification surface in `signature_profile`, `group_channel_crypto`, and `instruction_verify`.

## Verification
- Red:
  - `cargo test -p kamn-core signature_profile -- --nocapture` (failed: unresolved `constant_time_eq_*`)
- Green / Refactor / Integration:
  - `cargo test -p kamn-core --lib signature_profile -- --nocapture`
  - `cargo test -p kamn-core --lib instruction_verify -- --nocapture`
  - `cargo test -p kamn-core --lib group_channel_crypto -- --nocapture`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/kamn-core/src/constant_time_eq.rs crates/kamn-core/src/group_channel_crypto.rs crates/kamn-core/src/instruction_verify.rs crates/kamn-core/src/lib.rs crates/kamn-core/src/signature_profile.rs`

## Deviations
- None.

## Shell-surface closure template
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: 0
- shell_to_rust_ratio_delta_actual: 0.0
- shell_surface_ratio_target_status: improved
