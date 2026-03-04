# Spec: Issue 6363 - Extract kamn-cli dispatch wiring from `lib.rs`

## Objective
Move command dispatch wiring out of `crates/kamn-cli/src/lib.rs` into a dedicated module so `lib.rs` focuses on public types and high-level API wrappers.

## Inputs/Outputs
- Inputs:
  - Inline dispatch match in `lib.rs` (`dispatch` function).
  - Existing help-output helper from `cli_args` used by `CommandKind::Help`.
- Outputs:
  - New dispatch module with the command routing implementation.
  - `lib.rs` delegation to dispatch module.
  - Contract assertions for module declaration/delegation and inline marker removal.

## Boundaries/Non-goals
- Do not change command behavior, response formatting, command names, or error semantics.
- Do not change argument/help parsing behavior.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile failures.
- Dispatch mapping drift routes a command to wrong executor.
- Inline dispatch match remains in `lib.rs`, violating extraction contract.

## Acceptance criteria (testable booleans)
- [ ] `lib.rs` declares and uses a dedicated dispatch module.
- [ ] Inline dispatch-match markers are removed from `lib.rs`.
- [ ] Existing `kamn-cli` unit/contract tests remain green.
- [ ] Contract coverage asserts dispatch module declaration/delegation and new module API markers.

## Files to touch
- `specs/6363-kamn-cli-dispatch-module-extraction.md`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/cli_dispatch.rs` (new)
- `crates/kamn-cli/tests/main_contract.rs`

## Error semantics
- Preserve hard-fail behavior for command execution errors exactly as before.
- No silent fallbacks.

## Test plan
- Red:
  - Extend `main_contract.rs` with dispatch module declaration/delegation and inline marker removal assertions; confirm failure.
- Green/Refactor/Integration:
  - `cargo test -p kamn-cli --test main_contract`
  - `cargo test -p kamn-cli --test subcommand_surface_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
