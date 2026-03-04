# Spec: Issue 6361 - Extract kamn-cli argument/help parsing from `lib.rs`

## Objective
Move CLI argument/help parsing responsibilities out of `crates/kamn-cli/src/lib.rs` into a dedicated module so root `lib.rs` stays focused on public types and command dispatch orchestration.

## Inputs/Outputs
- Inputs:
  - Inline parsing/help surface in `lib.rs`:
    - command/flag constants
    - `env_var_or_default`
    - `is_help_request`
    - `parse_cli_args`
    - `render_help_text`
- Outputs:
  - New argument/help parsing module in `kamn-cli`.
  - `lib.rs` delegations to the module APIs.
  - Contract assertions proving module declaration/delegation and inline-marker removal.

## Boundaries/Non-goals
- Do not change command names, supported flags, defaults, or parse/error semantics.
- Do not change command dispatch behavior or command module execution paths.
- Do not change output JSON/text shapes.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile failures.
- Parsing behavior drift changes accepted args or deterministic error text.
- Help output drift changes usage/commands/flags formatting.
- Inline parser/help logic remains in `lib.rs`, violating extraction contract.

## Acceptance criteria (testable booleans)
- [x] `lib.rs` declares and uses a dedicated argument/help module.
- [x] Inline parser/help implementation markers are removed from `lib.rs`.
- [x] Existing `kamn-cli` unit/contract tests remain green.
- [x] Contract coverage asserts module declaration/delegation and new module API markers.

## Files to touch
- `specs/6361-kamn-cli-arg-help-module-extraction.md`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/cli_args.rs` (new)
- `crates/kamn-cli/tests/main_contract.rs`

## Error semantics
- Preserve hard-fail parse behavior:
  - unknown command remains `unsupported command: <value>`
  - unsupported flag remains `unsupported flag: <value>`
  - missing value markers for `--format`/`--endpoint` stay unchanged
  - missing command remains `missing command`
- No silent fallbacks.

## Test plan
- Red:
  - extend `main_contract.rs` with module declaration/delegation + inline-marker removal assertions and confirm failure.
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
