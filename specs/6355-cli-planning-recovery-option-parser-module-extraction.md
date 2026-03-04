# Spec: Issue 6355 - Extract CLI planning/recovery option parsing into module

## Objective
Move planning/recovery option parsing out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` stays focused on top-level dispatch.

## Inputs/Outputs
- Inputs:
  - Inline planning/recovery match arms in `cli.rs`:
    - `--expected-state-version`
    - `--expected-state-hash`
    - `--proposal`
    - `--rejoin-attempt`
- Outputs:
  - New planning/recovery option parser module.
  - `cli.rs` wiring to delegate planning/recovery option parsing.
  - Contract assertions for module declaration and inline-arm removal.

## Boundaries/Non-goals
- Do not change CLI flags, defaults, or parse order.
- Do not change runtime-mode validation behavior.
- Do not change daemon/endpoint/Kolme Live parser semantics.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Planning/recovery parse behavior drift changes required-value or parse error outcomes.
- Inline planning/recovery match arms remain in `cli.rs` and violate extraction contract.

## Acceptance criteria (testable booleans)
- [x] `cli.rs` declares a dedicated planning/recovery option parser module.
- [x] `cli.rs` no longer contains inline planning/recovery option match arms.
- [x] Planning/recovery parse behavior and `ConfigError` outcomes remain unchanged.
- [x] Extraction contract includes planning/recovery module assertions and passes.
- [x] Existing CLI tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_planning_recovery_option_parsing.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6355-cli-planning-recovery-option-parser-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior:
  - missing value arguments continue returning `ConfigError::MissingArgumentValue(<flag>)`
  - invalid planning/recovery values continue using existing value parser errors for state version, proposal, and rejoin attempt.

## Test plan
- Red:
  - Extend `cli_module_extraction_contract` with planning/recovery parser module declaration and inline-arm removal assertions; confirm failure.
- Green/refactor/integration:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-node --test cli_module_extraction_contract` (pass)
- `cargo test -p kamn-node cli_tests` (pass)
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract` (pass)
- `timeout 30s cargo run -p kamn-node -- --role processor --runtime-mode recovery-check --expected-state-version 1 --expected-state-hash reason --rejoin-attempt 'peer-a|1|reason|accepted' --output text --diagnostics basic` (pass)
- `timeout 30s cargo run -p kamn-node -- --role processor --runtime-mode planning --expected-state-hash reason --output text --diagnostics basic --proposal` (expected hard-fail: missing `--proposal`)

## Deviations
- None.
