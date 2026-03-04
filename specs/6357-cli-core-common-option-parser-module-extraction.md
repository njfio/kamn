# Spec: Issue 6357 - Extract CLI core/common option parsing into module

## Objective
Move core/common option parsing out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` remains focused on orchestration and module delegation.

## Inputs/Outputs
- Inputs:
  - Inline core/common match arms in `cli.rs`:
    - `--role`
    - `--profile`
    - `--chain-id`
    - `--chain-version`
    - `--storage-dir`
    - `--disable-gossip`
    - `--sync-mode`
    - `--runtime-mode`
    - `--output`
    - `--diagnostics`
- Outputs:
  - New core/common option parser module.
  - `cli.rs` wiring to delegate core/common parsing.
  - Contract assertions for module declaration and inline-arm removal.

## Boundaries/Non-goals
- Do not change CLI flags, defaults, or parse order.
- Do not change override marker behavior used by profile defaults.
- Do not change planning/recovery, daemon, endpoint, Kolme Live parsers.
- Do not change runtime-mode validation or post-parse guard behavior.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Core/common parse behavior drift changes required-value or enum parse errors.
- Override marker drift causes profile default application regressions.
- Inline core/common match arms remain in `cli.rs` and violate extraction contract.

## Acceptance criteria (testable booleans)
- [x] `cli.rs` declares a dedicated core/common option parser module.
- [x] `cli.rs` no longer contains inline core/common option match arms.
- [x] Core/common parse behavior and `ConfigError` outcomes remain unchanged.
- [x] Override marker behavior for profile defaults remains unchanged.
- [x] Extraction contract includes core/common module assertions and passes.
- [x] Existing CLI tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_core_common_option_parsing.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6357-cli-core-common-option-parser-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior:
  - missing values continue returning `ConfigError::MissingArgumentValue(<flag>)`
  - enum parsing continues using existing parse errors for role, profile, sync mode, runtime mode, output mode, and diagnostics mode.

## Test plan
- Red:
  - Extend `cli_module_extraction_contract` with core/common parser module declaration and inline-arm removal assertions; confirm failure.
- Green/refactor/integration:
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
