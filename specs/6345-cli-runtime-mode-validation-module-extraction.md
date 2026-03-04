# Spec: Issue 6345 - Extract CLI runtime-mode requirement validation into module

## Objective
Move runtime-mode requirement validation guards out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` remains focused on argument collection and wiring.

## Inputs/Outputs
- Inputs:
  - Inline runtime-mode validation guards in `cli.rs`:
    - planning mode required args checks
    - recovery-check mode required args checks
    - daemon/full runtime required args checks
    - api/full bind checks
    - kolme-live mode policy and requirement checks
- Outputs:
  - New module file for runtime-mode validation helpers, with focused Kolme-live validation submodule.
  - `cli.rs` module wiring/imports updated to call extracted validators.
  - CLI extraction contract updates for module declaration + inline-removal assertions.

## Boundaries/Non-goals
- Do not change CLI flags or defaults.
- Do not change config/env layering behavior.
- Do not change `ConfigError` mapping semantics for runtime-mode validation paths.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Runtime-mode validation drift changes error behavior.
- Inline guard branches remain in `cli.rs` and violate extraction contract.

## Acceptance criteria (testable booleans)
- [x] `cli.rs` declares a dedicated runtime-mode validation helper module.
- [x] `cli.rs` no longer contains inline runtime-mode requirement guard branches.
- [x] Existing CLI parse behavior and `ConfigError` mappings remain unchanged.
- [x] CLI extraction contract includes and passes runtime-mode validation module assertions.
- [x] Existing `kamn-node` CLI parsing tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_runtime_mode_validation.rs` (new)
- `crates/kamn-node/src/cli_runtime_mode_validation/kolme_live.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6345-cli-runtime-mode-validation-module-extraction.md`

## Error semantics
- Preserve existing hard-fail `ConfigError` behavior and messages for runtime-mode requirement/policy validation.

## Test plan
- Red phase:
  - Extend `cli_module_extraction_contract` with runtime-mode validation module declaration and inline-removal assertions; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-node --test cli_module_extraction_contract`:
  - pass (`5 passed, 0 failed`)
- `cargo test -p kamn-node cli_tests`:
  - pass (`7 passed, 0 failed`)
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`:
  - pass (`2 passed, 0 failed`)

## Deviations
- None.
