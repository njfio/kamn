# 7021 Restore Local Quality Gates

## Objective
Restore the repository's local quality gate so MVP demo work can proceed on an
honest green baseline. The gate for this issue is the existing `make check`
contract: `cargo fmt --check` plus strict workspace clippy with all warnings
denied.

## Inputs/Outputs
- Inputs:
  - current `origin/main` after `git fetch --all --prune`
  - existing `Makefile` `check` target
  - current `cargo fmt --check` failure
  - current strict workspace clippy failure
- Outputs:
  - formatted Rust workspace according to `cargo fmt`
  - strict clippy-clean workspace under all targets and all features
  - preserved tests, lint levels, and proof semantics
  - verification evidence for `cargo fmt --check`, strict clippy, and
    `make check`

## Boundaries/Non-goals
- Do not add the MVP demo command in this issue.
- Do not change the MVP proof report schema or claim taxonomy in this issue.
- Do not alter CI, workflow, script, or Makefile behavior unless the gate
  configuration itself is proven incorrect.
- Do not add broad `#[allow(...)]` attributes to hide real warnings.
- Do not weaken tests, lint levels, clippy strictness, formatting checks, or
  proof semantics.
- Do not refactor unrelated modules beyond what is needed for formatting,
  strict clippy, touched-code size limits, or behavior-preserving cleanup.

## Failure modes
- formatting drift remains and `cargo fmt --check` still fails
- strict clippy still reports warnings under `--workspace --all-targets
  --all-features -- -D warnings`
- a lint fix changes existing runtime behavior
- a warning is hidden by relaxing lint configuration instead of fixing code
- tests are deleted, weakened, or skipped to make the gate appear green
- proof or validation language is diluted while touching nearby files

## Acceptance criteria (testable booleans)
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
      passes.
- [ ] `make check` passes.
- [ ] No lint level, formatting rule, test, or proof semantic is weakened.
- [ ] Any semantic clippy fix is covered by existing or newly added focused
      tests for the touched surface.
- [ ] If the issue becomes too large to review safely, follow-up
      crate/module-specific gate-recovery issues are opened and MVP feature work
      remains blocked until all gate-recovery slices are green.

## Files to touch
- `specs/7021-restore-local-quality-gates.md`
- Rust source and test files reported by `cargo fmt --check`
- Rust source and test files reported by strict workspace clippy
- No shell, workflow, or template files are expected for this issue

## Error semantics
- Existing hard-fail behavior must be preserved.
- Interior error paths must continue returning typed/structured errors where
  they do today.
- Entrypoints must continue translating and logging failures only at their
  existing boundaries.
- Clippy fixes may simplify expressions, remove unused bindings, split complex
  types, or replace panic-prone test helpers, but they must not introduce silent
  fallbacks or swallow errors.

## Test plan
1. Red:
   - run `cargo fmt --check` and capture the formatting failure
   - run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
     and capture the strict clippy failure
2. Green:
   - run `cargo fmt`
   - fix strict clippy warnings by category without lowering strictness
   - run focused tests for any touched behavior surface
3. Refactor:
   - review touched files for repo size and responsibility rules
   - extract helpers or type aliases only where they reduce real clippy or
     readability problems
   - remove any temporary scaffolding used during gate recovery
4. Integration/proof:
   - run `cargo fmt --check`
   - run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - run `make check`
   - run `cargo test -p kamn-core`
   - run `cargo test -p kamn-node`
