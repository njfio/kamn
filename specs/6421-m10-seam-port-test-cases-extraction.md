# Spec: Issue #6421 - Extract M10 seam-port tests into dedicated cases module

## Objective

Extract seam-port contract tests (`spec_c37`, `spec_c38`) and their local fake-port scaffolding from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into `seam_port_cases.rs`, while preserving root entrypoint names and behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
  - existing split guard `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`
- Outputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival/seam_port_cases.rs`
  - root wrappers for C37/C38 delegating into seam-port module
  - split-contract marker checks for seam-port extraction

## Boundaries/Non-goals

- In scope:
  - C37/C38 test extraction only
  - moving seam-specific fake test doubles used by those tests
  - split-contract guard update for delegation
- Out of scope:
  - production M10 logic changes
  - extracting other sections from the root test file
  - changing assertion semantics in C37/C38

## Failure modes

- FM-1: seam-port scenarios remain inline in root file.
- FM-2: fake seam test doubles remain in root despite extraction.
- FM-3: split-contract guard does not enforce seam-port delegation markers.
- FM-4: split-contract or main archival suite fails.

## Acceptance criteria (testable booleans)

- [x] AC-1: `seam_port_cases.rs` exists and owns C37/C38 scenario bodies plus seam-local fake ports.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` keeps C37/C38 test entrypoint names, delegating to `seam_port_cases` functions.
- [x] AC-3: split-contract test enforces seam-port delegation markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch

- `specs/6421-m10-seam-port-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/seam_port_cases.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics

- Test-only refactor. Fail-closed behavior remains assertion-driven.

## Test plan

- RED:
  - extend split-contract test with seam-port delegation markers.
  - verify split-contract lane fails before extraction.
- GREEN:
  - extract C37/C38 + seam-local fake ports into `seam_port_cases.rs`.
  - keep root wrapper tests delegating to new module.
  - run split-contract + archival lanes.
- REFACTOR:
  - centralize seam-port marker arrays in split-contract test.
- INTEGRATION:
  - run both test lanes and record outcomes.

## Phase 6 integration evidence

- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`2 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations

- None.
