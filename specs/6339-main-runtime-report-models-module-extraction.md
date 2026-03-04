# Spec: Issue 6339 - Extract main runtime/report data structs into module

## Objective
Move runtime/report data structure definitions out of `crates/kamn-node/src/main.rs` into a dedicated module so `main.rs` stays focused on runtime entrypoint orchestration.

## Inputs/Outputs
- Inputs:
  - Inline data structures in `main.rs`:
    - `NodeCli`
    - `PlanningExecution`
    - `RecoveryExecution`
    - `DaemonExecution`
    - `DaemonRuntimeOptions`
    - `KolmeLiveExecution`
    - `RuntimeExecutionBundle`
    - `NodeBootstrapReport`
- Outputs:
  - New module file `runtime_models.rs` containing extracted structs.
  - `main.rs` module wiring and imports/re-exports updated to use extracted structs.
  - Main extraction contract updates for module declaration + inline-removal assertions.

## Boundaries/Non-goals
- Do not change CLI flags or parse behavior.
- Do not change runtime execution flow.
- Do not change bootstrap report field names/semantics.
- Do not add dependencies.

## Failure modes
- Missing module wiring/visibility causes compile failures.
- Struct field drift changes runtime/report behavior.
- Contract assertions fail due to inline definitions remaining in `main.rs`.

## Acceptance criteria (testable booleans)
- [ ] `main.rs` declares `mod runtime_models;`.
- [ ] `main.rs` no longer contains inline definitions for extracted runtime/report structs.
- [ ] Extracted structs remain visible where needed and behavior/signatures are unchanged.
- [ ] `main_module_extraction_contract` includes and passes runtime models module assertions.
- [ ] Existing `kamn-node` runtime/report tests remain green.

## Files to touch
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/runtime_models.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `specs/6339-main-runtime-report-models-module-extraction.md`

## Error semantics
- Preserve existing hard-fail behavior and `ConfigError` mapping semantics in all runtime/report execution paths.

## Test plan
- Red phase:
  - Extend `main_module_extraction_contract` with runtime models module declaration and inline-removal assertions; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`
  - `cargo test -p kamn-node --test runtime_output_contract`
  - `cargo test -p kamn-node --test runtime_entrypoint_invalid_mode`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
