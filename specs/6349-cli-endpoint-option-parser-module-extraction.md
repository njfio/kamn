# Spec: Issue 6349 - Extract CLI endpoint option parsing into module

## Objective
Move service API and observability endpoint option parsing out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` stays focused on top-level argument dispatch.

## Inputs/Outputs
- Inputs:
  - Inline endpoint-related match arms in `cli.rs`:
    - `--api-bind`
    - `--api-max-requests`
    - `--api-idle-timeout-ms`
    - `--api-body-limit-bytes`
    - `--api-concurrency-limit`
    - `--api-rate-limit-per-second`
    - `--observability-endpoint-bind`
    - `--observability-endpoint-metrics-path`
    - `--observability-endpoint-health-path`
    - `--observability-endpoint-max-requests`
    - `--observability-endpoint-idle-timeout-ms`
- Outputs:
  - New module that parses endpoint options and updates endpoint option state.
  - `cli.rs` wiring that delegates endpoint option parsing to the new module.
  - Contract test assertions for module declaration and inline endpoint-arm removal markers.

## Boundaries/Non-goals
- Do not change CLI flags, defaults, or parse order.
- Do not change runtime-mode or post-parse endpoint guard behavior.
- Do not change `ConfigError` variants/messages for endpoint option parse failures.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Endpoint parse behavior drift changes required-value or numeric parse errors.
- Endpoint override markers are not set correctly, causing guard regressions.
- Inline endpoint parsing remains in `cli.rs` and violates extraction contract.

## Acceptance criteria (testable booleans)
- [ ] `cli.rs` declares a dedicated endpoint option parser module.
- [ ] `cli.rs` no longer contains inline API/observability endpoint match arms.
- [ ] Endpoint parse behavior and `ConfigError` outcomes remain unchanged.
- [ ] Endpoint override marker behavior remains unchanged.
- [ ] Extraction contract includes endpoint parser module assertions and passes.
- [ ] Existing CLI tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_endpoint_option_parsing.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6349-cli-endpoint-option-parser-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior:
  - missing values continue returning `ConfigError::MissingArgumentValue(<flag>)`
  - invalid numeric endpoint values continue returning existing invalid daemon control argument errors via shared parser helper.

## Test plan
- Red:
  - Extend `cli_module_extraction_contract` with endpoint parser module declaration and inline endpoint-arm removal assertions; confirm failure.
- Green/refactor/integration:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
