# Spec: Issue 6369 - Extract kamn-cli public model types from `lib.rs`

## Objective
Move core CLI model type definitions out of `crates/kamn-cli/src/lib.rs` into a dedicated module while preserving all existing behavior and API semantics.

## Inputs/Outputs
- Inputs:
  - Inline definitions in `lib.rs`:
    - `OutputFormat`
    - `CommandKind`
    - `ParsedCliArgs`
    - `CommandOutput`
- Outputs:
  - New models/types module file owning these definitions.
  - `lib.rs` module declaration and type re-export/use wiring.
  - Contract assertions for module declaration and inline-marker removal.

## Boundaries/Non-goals
- Do not change parse behavior, command dispatch behavior, output shapes, or error text.
- Do not change command names/aliases or format values.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile failures.
- Type visibility drift breaks module consumers.
- Inline type definitions remain in `lib.rs`, violating extraction contract.

## Acceptance criteria (testable booleans)
- [x] `lib.rs` declares and uses a dedicated model-types module.
- [x] Inline type-definition markers are removed from `lib.rs`.
- [x] Existing `kamn-cli` tests remain green.
- [x] Contract coverage asserts module declaration and API markers in new module.

## Files to touch
- `specs/6369-kamn-cli-model-types-module-extraction.md`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/cli_models.rs` (new)
- `crates/kamn-cli/tests/main_contract.rs`

## Error semantics
- No runtime error behavior changes.
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Extend `main_contract.rs` with model-module declaration assertions and inline-marker removal checks; confirm failure.
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
