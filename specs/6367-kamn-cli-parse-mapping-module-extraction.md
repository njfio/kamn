# Spec: Issue 6367 - Extract kamn-cli command/format parse mapping from `lib.rs`

## Objective
Move `CommandKind` and `OutputFormat` parse mappings out of `crates/kamn-cli/src/lib.rs` into a dedicated module so root API type definitions remain decoupled from parse-table details.

## Inputs/Outputs
- Inputs:
  - Inline `impl OutputFormat { fn parse(...) }` mapping in `lib.rs`.
  - Inline `impl CommandKind { fn parse(...) }` mapping in `lib.rs`.
- Outputs:
  - New parse-mapping module with parsing entrypoints for command kind and output format.
  - `lib.rs` delegation to module parse entrypoints.
  - Contract assertions verifying module declaration/delegation and inline-marker removal.

## Boundaries/Non-goals
- Do not change command names, aliases, output format values, defaults, output text/json shapes, or error strings.
- Do not change dispatch logic or argument/help parsing flow.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile failures.
- Parse mapping drift changes accepted commands/formats or error messages.
- Inline parse mappings remain in `lib.rs`, violating extraction contract.

## Acceptance criteria (testable booleans)
- [ ] `lib.rs` declares and uses a dedicated parse-mapping module.
- [ ] Inline parse-match markers are removed from `lib.rs`.
- [ ] Existing `kamn-cli` tests remain green.
- [ ] Contract coverage asserts module API markers and delegation calls.

## Files to touch
- `specs/6367-kamn-cli-parse-mapping-module-extraction.md`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/cli_parse_mapping.rs` (new)
- `crates/kamn-cli/tests/main_contract.rs`

## Error semantics
- Preserve hard-fail parse behavior and text exactly:
  - unsupported format: `unsupported format: <raw>`
  - unsupported command: `unsupported command: <raw>`
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Extend `main_contract.rs` with parse-mapping module assertions + inline-marker removal checks; confirm failure.
- Green/Refactor/Integration:
  - `cargo test -p kamn-cli --test main_contract`
  - `cargo test -p kamn-cli --test subcommand_surface_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
