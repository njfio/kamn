# Spec: Issue 6437 - Extract M10 archival shared helper module

## Objective
Extract the large import + helper-builder block from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into a dedicated `shared` module while preserving root test entrypoints and all existing scenario behavior.

## Inputs/Outputs
- Inputs:
  - Root `data_layer_m10_partition_archival.rs` import block and helper functions (`partition_input`, `m8_message_input`, `project_request`, `phase6_request`, `phase6_budget`, `phase6_scheduler_policy`, `phase6_runtime_state`).
  - Existing extracted case modules depending on `super::*` symbols.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m10_partition_archival/shared.rs` containing helper functions and re-exported symbols.
  - Root `data_layer_m10_partition_archival.rs` that consumes `shared::*` and retains wrapper entrypoints.
  - Split-contract guard markers validating shared-module wiring and helper ownership.

## Boundaries/Non-goals
- No behavior changes to test scenarios.
- No changes to production code under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Case modules lose access to symbols previously provided through root scope.
- Helper functions remain duplicated between root and shared module.
- Split contract does not enforce shared-module ownership markers.

## Acceptance criteria (testable booleans)
- [x] AC-1: `shared.rs` exists with helper functions and re-exported symbols used by case modules.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` imports from `shared::*` and no longer defines the moved helpers inline.
- [x] AC-3: split-contract test enforces shared-module wiring/ownership markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch
- `specs/6437-m10-shared-helper-module-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/shared.rs` (new)
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics
- No changes to assertion behavior or reason-code checks.

## Test plan
- Red:
  - Add split-contract shared-module marker checks before moving helpers; verify failing test.
- Green:
  - Move helper/import block into `shared.rs` and wire root `use shared::*`.
- Refactor:
  - Minor literal/format cleanup in `shared.rs` without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Phase 6 integration evidence
- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`10 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations
- None.
