# Spec: Issue 6351 - Extract CLI daemon option parsing into module

## Objective
Move daemon runtime control and lifecycle option parsing out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` remains focused on top-level dispatch.

## Inputs/Outputs
- Inputs:
  - Inline daemon-related match arms in `cli.rs`:
    - `--daemon-max-ticks`
    - `--daemon-tick-interval-ms`
    - `--daemon-shutdown-signal-tick`
    - `--daemon-shutdown-os-signals`
    - `--daemon-shutdown-drain-ticks`
    - `--daemon-shutdown-timeout-ticks`
    - `--daemon-peer-id`
    - `--daemon-lifecycle-event`
- Outputs:
  - New module for daemon option parsing/state updates.
  - `cli.rs` wiring that delegates daemon option parsing to the new module.
  - Contract test assertions for daemon module declaration and inline-arm removal markers.

## Boundaries/Non-goals
- Do not change CLI flags, defaults, or parse order.
- Do not change runtime-mode validation behavior.
- Do not change endpoint option parsing behavior.
- Do not add dependencies.

## Failure modes
- Missing daemon parser module wiring causes compile/test failures.
- Daemon parse behavior drift changes required-value or numeric parse errors.
- Daemon lifecycle event parser mapping drift changes failure outcomes.
- Inline daemon parsing remains in `cli.rs` and violates extraction contract.

## Acceptance criteria (testable booleans)
- [ ] `cli.rs` declares a dedicated daemon option parser module.
- [ ] `cli.rs` no longer contains inline daemon option match arms.
- [ ] Daemon option parse behavior and `ConfigError` outcomes remain unchanged.
- [ ] Daemon lifecycle event parse behavior remains unchanged.
- [ ] Extraction contract includes daemon parser module assertions and passes.
- [ ] Existing CLI tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_daemon_option_parsing.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6351-cli-daemon-option-parser-module-extraction.md`

## Error semantics
- Preserve existing hard-fail behavior:
  - missing daemon option values continue returning `ConfigError::MissingArgumentValue(<flag>)`
  - invalid daemon numeric values continue returning existing invalid daemon control argument errors via shared parser helper
  - invalid daemon lifecycle values continue returning existing daemon lifecycle parse errors.

## Test plan
- Red:
  - Extend `cli_module_extraction_contract` with daemon parser module declaration and inline daemon-arm removal assertions; confirm failure.
- Green/refactor/integration:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
