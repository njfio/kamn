# Spec: Issue #6423 - Extract M10 scheduler-runtime tests into dedicated cases module

## Objective

Extract scheduler-runtime scenario tests (`spec_c28`..`spec_c32`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into `scheduler_runtime_cases.rs`, preserving root test entrypoint names and behavioral assertions.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
  - split guard file `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`
- Outputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival/scheduler_runtime_cases.rs`
  - root wrappers for C28-C32 delegating to scheduler-runtime cases functions
  - split-contract guard markers for scheduler-runtime delegation

## Boundaries/Non-goals

- In scope:
  - C28-C32 test extraction only
  - split-contract guard updates for delegation markers
- Out of scope:
  - production M10 code changes
  - extracting additional sections of the root test file in this issue
  - changing scenario behavior/assertion semantics

## Failure modes

- FM-1: C28-C32 remain inline in root file.
- FM-2: root wrappers for C28-C32 are missing.
- FM-3: split guard lacks scheduler-runtime delegation checks.
- FM-4: split-contract or archival test suite failures.

## Acceptance criteria (testable booleans)

- [x] AC-1: `scheduler_runtime_cases.rs` exists and contains C28-C32 scenario bodies.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` keeps C28-C32 test entrypoint names and delegates to scheduler-runtime cases functions.
- [x] AC-3: split-contract test enforces scheduler-runtime delegation markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch

- `specs/6423-m10-scheduler-runtime-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/scheduler_runtime_cases.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics

- Test-only refactor. Fail-closed assertion behavior remains unchanged.

## Test plan

- RED:
  - add split-contract scheduler-runtime delegation markers and confirm failure before extraction.
- GREEN:
  - add scheduler-runtime cases module and move C28-C32 bodies.
  - update root C28-C32 wrappers to delegate.
  - run split-contract + archival suites.
- REFACTOR:
  - centralize scheduler-runtime marker arrays in split guard.
- INTEGRATION:
  - run required test lanes and record evidence.

## Phase 6 integration evidence

- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`3 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations

- None.
