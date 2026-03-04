# Spec: Issue 6359 - Extract CLI parse orchestration state from `cli.rs`

## Objective
Move CLI parse-state orchestration out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` focuses on control flow, validation calls, and final `NodeCli` return wiring.

## Inputs/Outputs
- Inputs:
  - Current inline parse-state initialization block in `cli.rs` (`let mut role ...` through override flags).
  - Current parse loop delegation in `parse_args` that mutates shared state.
- Outputs:
  - New module that owns parse-state data shape and initialization.
  - `cli.rs` wiring that consumes the extracted parse-state object.
  - Contract assertions proving module declaration/entrypoint presence and inline marker removal.

## Boundaries/Non-goals
- Do not change CLI flags, defaults, parse order, or parsing semantics.
- Do not change runtime-mode validation or endpoint guard semantics.
- Do not redesign existing option parser modules (`core_common`, `daemon`, `endpoint`, `kolme_live`, `planning_recovery`).
- Do not add new dependencies.

## Failure modes
- Missing module wiring causes build failures.
- State field mapping drift causes behavior regression in parsed `NodeCli` values.
- Initialization/default drift changes role/profile/default-chain/runtime behavior.
- Inline parse-state initialization remains in `cli.rs`, violating extraction contract.

## Acceptance criteria (testable booleans)
- [x] `cli.rs` declares and uses a dedicated parse-state orchestration module.
- [x] Inline bulk parse-state initialization markers are removed from `cli.rs`.
- [x] Existing CLI extraction contract and `cli_tests` remain green.
- [x] Contract coverage asserts the new module declaration and new module API markers.

## Files to touch
- `specs/6359-cli-parse-orchestration-state-module-extraction.md`
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_parse_state.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`

## Error semantics
- Preserve hard-fail behavior exactly:
  - unknown flags continue returning `ConfigError::UnknownArgument(...)`.
  - missing required values continue returning `ConfigError::MissingArgumentValue(...)`.
  - enum parse failures continue using existing parse paths.
- No silent fallback behavior may be introduced.

## Test plan
- Red:
  - Extend `cli_module_extraction_contract` with parse-state module declaration/entrypoint and inline-marker removal assertions; verify failure before implementation.
- Green/Refactor/Integration:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-node --test cli_module_extraction_contract` (pass)
- `cargo test -p kamn-node cli_tests` (pass)
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract` (pass)
- `timeout 30s cargo run -p kamn-node -- --role processor --runtime-mode bootstrap --output text --diagnostics basic` (pass)
- `timeout 30s cargo run -p kamn-node -- --runtime-mode bootstrap --output text --diagnostics basic` (expected hard-fail: missing `--role`)

## Deviations
- None.
