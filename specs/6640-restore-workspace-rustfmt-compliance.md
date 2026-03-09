# 6640 Restore Workspace Rustfmt Compliance

## Objective
Restore workspace-wide `rustfmt` compliance so `cargo fmt --all --check` passes in a clean checkout and Fast Gate is no longer blocked by committed formatting drift on `main`.

## Inputs/Outputs
- Inputs:
  - Current workspace Rust sources and tests on `origin/main`
  - Stable toolchain `rustfmt 1.8.0-stable`
  - CI command `cargo fmt --all --check`
- Outputs:
  - Deterministically formatted Rust files required by `rustfmt`
  - Passing local `cargo fmt --all --check`
  - PR evidence that the formatting stage is green

## Boundaries/Non-goals
- No behavioral changes
- No refactors beyond formatting emitted by `rustfmt`
- No dependency, toolchain, or CI configuration changes
- No non-Rust file edits unless required to document acceptance evidence

## Failure Modes
- `cargo fmt --all --check` still reports drift after formatting
- Formatting step introduces non-deterministic churn outside `rustfmt`
- A supposed formatting-only diff changes program behavior or test expectations

## Acceptance Criteria
- [x] In a clean checkout of the fix branch, `cargo fmt --all --check` passes
- [x] The diff contains only deterministic formatting changes in Rust files
- [x] No production or test behavior changes are introduced
- [x] A PR carrying the fix can satisfy the Fast Gate format stage

## Files To Touch
- Rust files reported by `cargo fmt --all --check`
- `specs/6640-restore-workspace-rustfmt-compliance.md`

## Error Semantics
- Treat any remaining `cargo fmt --all --check` failure as a hard failure
- Treat any non-formatting source change discovered during review as a hard failure
- Do not suppress formatter errors or narrow the command scope

## Test Plan
1. Run `cargo fmt --all --check` on a clean checkout and record failure as RED
2. Run `cargo fmt --all` to apply deterministic formatting
3. Re-run `cargo fmt --all --check` and require success as GREEN
4. Inspect the diff to confirm formatting-only changes
5. Open a PR and verify the Fast Gate format stage passes

## Phase 6 Evidence
- `cargo fmt --all --check` failed on clean `origin/main` checkout before the fix
- `cargo fmt --all` applied deterministic formatting across 61 Rust files
- `cargo fmt --all --check` passes on branch `6640-restore-workspace-rustfmt-compliance` after formatting
- Diff review confirmed formatting-only rewrites with no intended behavioral changes
