# Spec: Issue #6425 - Extract M10 scheduler-cycle tests into dedicated cases module

## Objective

Extract scheduler decision/cycle scenario tests (`spec_c23`..`spec_c27`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into `scheduler_cycle_cases.rs`, preserving root test entrypoint names and assertion behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
  - split guard `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`
- Outputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival/scheduler_cycle_cases.rs`
  - root wrappers for C23-C27 delegating to scheduler-cycle cases module
  - split-contract guard markers for scheduler-cycle delegation

## Boundaries/Non-goals

- In scope:
  - C23-C27 extraction only
  - split-contract updates enforcing delegation markers
- Out of scope:
  - production M10 changes
  - extracting additional sections in this issue
  - changing scenario semantics/assertions

## Failure modes

- FM-1: C23-C27 remain inline in root file.
- FM-2: root C23-C27 wrappers are missing.
- FM-3: split guard lacks scheduler-cycle delegation checks.
- FM-4: split-contract or archival suite fails.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `scheduler_cycle_cases.rs` exists and contains C23-C27 scenario bodies.
- [ ] AC-2: root `data_layer_m10_partition_archival.rs` keeps C23-C27 entrypoint names and delegates to scheduler-cycle cases functions.
- [ ] AC-3: split-contract test enforces scheduler-cycle delegation markers.
- [ ] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch

- `specs/6425-m10-scheduler-cycle-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/scheduler_cycle_cases.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics

- Test-only refactor. Fail-closed assertion behavior remains unchanged.

## Test plan

- RED:
  - add split-contract scheduler-cycle delegation markers and confirm failure before extraction.
- GREEN:
  - add scheduler-cycle cases module and move C23-C27 bodies.
  - convert root C23-C27 tests to wrappers.
  - run split-contract + archival suites.
- REFACTOR:
  - centralize scheduler-cycle marker arrays in split guard.
- INTEGRATION:
  - run required test lanes and record evidence.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
