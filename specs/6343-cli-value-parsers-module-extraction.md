# Spec: Issue 6343 - Extract CLI value parser helpers into module

## Objective
Move low-level CLI value parser helper functions out of `crates/kamn-node/src/cli.rs` into a dedicated module so top-level CLI flow remains focused on argument orchestration.

## Inputs/Outputs
- Inputs:
  - Inline helper functions in `cli.rs`:
    - `parse_state_version_arg`
    - `parse_proposal_candidate`
    - `parse_rejoin_attempt`
    - `parse_daemon_control_arg`
    - `parse_daemon_lifecycle_event`
- Outputs:
  - New module file for CLI value parser helpers.
  - `cli.rs` module wiring and imports updated to call extracted helpers.
  - CLI extraction contract updates for module declaration + inline-removal assertions.

## Boundaries/Non-goals
- Do not change CLI flags or defaults.
- Do not change parser error mapping semantics.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Parser behavior drift changes `ConfigError` outcomes.
- Inline helper definitions remain in `cli.rs` and violate extraction contract.

## Acceptance criteria (testable booleans)
- [ ] `cli.rs` declares a dedicated value-parsers helper module.
- [ ] `cli.rs` no longer contains inline definitions for extracted value parser helpers.
- [ ] Existing CLI parse behavior remains unchanged.
- [ ] CLI extraction contract includes and passes value-parser module assertions.
- [ ] Existing `kamn-node` CLI parsing tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_value_parsers.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6343-cli-value-parsers-module-extraction.md`

## Error semantics
- Preserve existing hard-fail `ConfigError` mapping semantics for state-version, proposal, rejoin,
  daemon-control, and daemon-lifecycle value parsing.

## Test plan
- Red phase:
  - Extend `cli_module_extraction_contract` with value-parsers module declaration and inline-removal assertions; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
