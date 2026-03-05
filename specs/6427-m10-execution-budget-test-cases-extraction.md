# Spec: Issue #6427 - Extract M10 execution-budget tests into dedicated cases module

## Objective

Extract orchestration/execution-budget scenario tests (`spec_c18`..`spec_c22`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into `execution_budget_cases.rs`, preserving root entrypoint names and assertion behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
  - split guard `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`
- Outputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival/execution_budget_cases.rs`
  - root wrappers for C18-C22 delegating to execution-budget cases functions
  - split-contract guard markers for execution-budget delegation

## Boundaries/Non-goals

- In scope:
  - C18-C22 extraction only
  - split-contract updates for delegation markers
- Out of scope:
  - production M10 code changes
  - extracting additional sections in this issue
  - changing scenario semantics/assertions

## Failure modes

- FM-1: C18-C22 remain inline in root file.
- FM-2: root C18-C22 wrappers are missing.
- FM-3: split guard lacks execution-budget delegation checks.
- FM-4: split-contract or archival suite fails.

## Acceptance criteria (testable booleans)

- [x] AC-1: `execution_budget_cases.rs` exists and contains C18-C22 scenario bodies.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` keeps C18-C22 entrypoint names and delegates to execution-budget cases functions.
- [x] AC-3: split-contract test enforces execution-budget delegation markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch

- `specs/6427-m10-execution-budget-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/execution_budget_cases.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics

- Test-only refactor. Fail-closed assertion behavior remains unchanged.

## Test plan

- RED:
  - add split-contract execution-budget delegation markers and confirm failure before extraction.
- GREEN:
  - add execution-budget cases module and move C18-C22 bodies.
  - convert root C18-C22 tests into wrappers.
  - run split-contract + archival suites.
- REFACTOR:
  - centralize execution-budget marker arrays in split guard.
- INTEGRATION:
  - run required test lanes and record evidence.

## Phase 6 integration evidence

- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`5 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations

- None.
