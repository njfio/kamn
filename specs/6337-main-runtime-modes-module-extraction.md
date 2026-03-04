# Spec: Issue 6337 - Extract main runtime mode/profile parsing types into module

## Objective
Extract runtime mode/profile parsing types from `crates/kamn-node/src/main.rs` into a dedicated module so main orchestration remains focused on execution flow.

## Inputs/Outputs
- Inputs:
  - Inline types in `main.rs`:
    - `OutputMode` + `OutputModeKind`
    - `RuntimeMode` + `RuntimeModeKind`
    - `DiagnosticsMode`
    - `LocalProfile`
  - Existing `ConfigError` mapping semantics in parse/default helpers.
- Outputs:
  - New `runtime_modes.rs` module with extracted types/impls.
  - `main.rs` module wiring/imports updated to use extracted types.
  - Main extraction contract updates for module declaration + inline removal.

## Boundaries/Non-goals
- Do not change CLI flags or accepted values.
- Do not change runtime execution flow.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Parser behavior drift changes `ConfigError` field/reason semantics.
- CLI parse defaults regress.

## Acceptance criteria (testable booleans)
- [ ] `main.rs` declares `mod runtime_modes;`.
- [ ] `main.rs` no longer contains inline definitions for extracted mode/profile types and impls.
- [ ] Existing runtime mode/profile parse behavior remains unchanged.
- [ ] `main_module_extraction_contract` includes and passes runtime mode module assertions.
- [ ] Existing `kamn-node` tests for command surface/runtime mode paths remain green.

## Files to touch
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/runtime_modes.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `specs/6337-main-runtime-modes-module-extraction.md`

## Error semantics
- Preserve current hard-fail `ConfigError::InvalidNodeConfig` field/reason mapping in mode/profile parse helpers.

## Test plan
- Red phase:
  - Extend `main_module_extraction_contract` with runtime mode module declaration and inline-removal assertions; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`
  - `cargo test -p kamn-node --test runtime_output_contract`
  - `cargo test -p kamn-node --test runtime_entrypoint_invalid_mode`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
