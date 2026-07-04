# Restore CI-Stable Strict Clippy Compatibility

## Objective

Restore PR #7022 Fast Gate `Lint (strict)` compatibility with the CI stable
clippy toolchain without weakening lint flags, workflow behavior, or runtime
request identity semantics.

## Inputs/Outputs

Input:

- CI Fast Gate log for PR #7022 head
  `d6ecc87d3130deebbe19fc2c4ed85e8dac272983`.
- Local source in
  `crates/kamn-kolme/src/runtime_request_identity_policy.rs`.

Output:

- CI-stable-compatible Rust formatting that preserves deterministic runtime
  commit payload/id output.
- Green local `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Green local `make check`.

## Boundaries/Non-goals

- Do not change clippy flags or workflow strictness.
- Do not add `#[allow(...)]` for the reported lint.
- Do not change runtime request identity output.
- Do not add MVP demo, proof, report, settlement, or devnet behavior.
- Do not refactor unrelated `kamn-kolme` code.

## Failure modes

- CI-stable clippy still emits `clippy::uninlined_format_args`.
- Local strict clippy or `make check` regresses.
- Runtime request identity output changes while only lint compatibility was
  intended.

## Acceptance criteria

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `make check` passes.
- CI Fast Gate `Lint (strict)` passes on PR #7022.
- No lint, test, or workflow gate is weakened.
- Existing runtime request identity tests remain green.

## Files to touch

- `crates/kamn-kolme/src/runtime_request_identity_policy.rs`

## Error semantics

This issue does not alter runtime error semantics. Any failure remains surfaced by
the existing strict clippy or test commands.

## Test plan

Red evidence:

- CI Fast Gate `Lint (strict)` failed at PR #7022 head
  `d6ecc87d3130deebbe19fc2c4ed85e8dac272983` with
  `clippy::uninlined_format_args` in
  `crates/kamn-kolme/src/runtime_request_identity_policy.rs`.

Green verification:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
```
