# Spec: Issue 6341 - Extract CLI config/env layering helpers into module

## Objective
Move layered config/env preprocessing helpers out of `crates/kamn-node/src/cli.rs` into a dedicated module so CLI parse orchestration remains focused and maintainable.

## Inputs/Outputs
- Inputs:
  - Inline helper functions in `cli.rs`:
    - `read_env_var_trimmed`
    - `parse_bool_override`
    - `push_key_value_flag`
    - `map_config_entry_to_args`
    - `parse_config_file_args`
    - `append_env_override`
    - `collect_env_override_args`
    - `extract_config_file_path`
    - `build_layered_cli_args`
- Outputs:
  - New module file for config/env layering helpers.
  - `cli.rs` module wiring updated to call extracted helpers.
  - CLI extraction contract test for module declaration and inline-removal invariants.

## Boundaries/Non-goals
- Do not change CLI flags or defaults.
- Do not change config-key to flag mapping behavior.
- Do not change environment-variable precedence behavior.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Behavior drift in config/env layering changes parsed argument outcomes.
- Inline helpers left in `cli.rs` violate extraction contract.

## Acceptance criteria (testable booleans)
- [ ] `cli.rs` declares a dedicated config-layering helper module.
- [ ] `cli.rs` no longer contains inline definitions for extracted config/env layering helpers.
- [ ] Existing CLI parse behavior remains unchanged.
- [ ] CLI extraction contract includes and passes module declaration + inline-removal assertions.
- [ ] Existing `kamn-node` CLI parsing tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_config_layering.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs` (new)
- `specs/6341-cli-config-layering-module-extraction.md`

## Error semantics
- Preserve current hard-fail `ConfigError::InvalidNodeConfig` behavior and message semantics for config/env preprocessing paths.

## Test plan
- Red phase:
  - Add `cli_module_extraction_contract` with module declaration + inline-removal assertions; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
