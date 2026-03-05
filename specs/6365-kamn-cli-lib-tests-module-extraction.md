# Spec: Issue 6365 - Extract kamn-cli lib tests into dedicated module

## Objective
Move inline `#[cfg(test)]` coverage out of `crates/kamn-cli/src/lib.rs` into a dedicated test module file so `lib.rs` remains focused on production API surface.

## Inputs/Outputs
- Inputs:
  - Current inline `#[cfg(test)] mod tests` block in `crates/kamn-cli/src/lib.rs`.
- Outputs:
  - New `crates/kamn-cli/src/lib_tests.rs` module (or equivalent dedicated module path).
  - `lib.rs` test-module delegation via module declaration.
  - Contract assertions for delegation marker and inline test-marker removal.

## Boundaries/Non-goals
- Do not change command behavior, parser behavior, outputs, or error semantics.
- Do not change production dispatch/parser module logic.
- Do not add dependencies.

## Failure modes
- Missing test module wiring causes compile/test failures.
- Test behavior drift due to move mistakes.
- Inline test module remains in `lib.rs`, violating extraction contract.

## Acceptance criteria (testable booleans)
- [x] `lib.rs` delegates test coverage to dedicated test module file.
- [x] Inline test-function markers are removed from `lib.rs`.
- [x] Existing `kamn-cli` test suites remain green.
- [x] Contract coverage asserts delegation marker and module API file existence.

## Files to touch
- `specs/6365-kamn-cli-lib-tests-module-extraction.md`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/lib_tests.rs` (new)
- `crates/kamn-cli/tests/main_contract.rs`

## Error semantics
- No runtime error behavior changes.
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Extend `main_contract.rs` with test-module delegation and inline-marker removal assertions; confirm failure.
- Green/Refactor/Integration:
  - `cargo test -p kamn-cli --test main_contract`
  - `cargo test -p kamn-cli --test subcommand_surface_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-cli --test main_contract` (pass)
- `cargo test -p kamn-cli --test subcommand_surface_contract` (pass)
- `cargo test -p kamn-cli --test command_activation_contract` (pass)
- `timeout 30s cargo run -p kamn-cli -- --help` (pass)
- `timeout 30s cargo run -p kamn-cli --` (expected parse-error hard-fail: `missing command`)

## Deviations
- None.
