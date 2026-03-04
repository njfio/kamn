# Spec: Issue 6347 - Extract CLI profile default and endpoint guard validation into module

## Objective
Move post-parse profile-default application and endpoint guard validations out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` remains focused on argument dispatching.

## Inputs/Outputs
- Inputs:
  - Inline blocks in `cli.rs`:
    - profile default application based on `LocalProfile`
    - API endpoint override guard checks
    - observability endpoint override guard checks
    - observability path format validation
- Outputs:
  - New module file for post-parse profile/endpoint guard helpers.
  - `cli.rs` module wiring/imports updated to call extracted helpers.
  - CLI extraction contract updates for module declaration + inline-removal assertions.

## Boundaries/Non-goals
- Do not change CLI flags or defaults.
- Do not change runtime-mode validation behavior.
- Do not change `ConfigError` mapping semantics.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Guard behavior drift changes parse outcomes.
- Inline guard blocks remain in `cli.rs` and violate extraction contract.

## Acceptance criteria (testable booleans)
- [x] `cli.rs` declares a dedicated post-parse profile/endpoint guard module.
- [x] `cli.rs` no longer contains inline profile-default and endpoint guard validation blocks.
- [x] Existing CLI parse behavior and `ConfigError` mappings remain unchanged.
- [x] CLI extraction contract includes and passes profile/endpoint guard module assertions.
- [x] Existing `kamn-node` CLI parsing tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_post_parse_guards.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6347-cli-profile-endpoint-guard-module-extraction.md`

## Error semantics
- Preserve existing hard-fail `ConfigError` behavior for missing endpoint bindings and invalid observability path formats.

## Test plan
- Red phase:
  - Extend `cli_module_extraction_contract` with post-parse guard module declaration and inline-removal assertions; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-node --test cli_module_extraction_contract` (pass)
- `cargo test -p kamn-node cli_tests` (pass)
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract` (pass)
- `timeout 30s cargo run -p kamn-node -- --role processor --runtime-mode planning --expected-state-hash reason --proposal 'alpha|beta|1|reason' --output text --diagnostics basic` (pass; runtime entrypoint exercised)

## Deviations
- None.
