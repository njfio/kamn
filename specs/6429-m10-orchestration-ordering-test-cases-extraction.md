# Spec: Issue #6429 - Extract M10 orchestration-ordering tests into dedicated cases module

## Objective

Extract orchestration ordering scenario tests (`spec_c16`, `spec_c17`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into `orchestration_ordering_cases.rs`, preserving root entrypoint names and existing assertion behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
  - split guard `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`
- Outputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival/orchestration_ordering_cases.rs`
  - root wrappers for C16/C17 delegating to orchestration-ordering cases functions
  - split-contract guard markers for C16/C17 delegation

## Boundaries/Non-goals

- In scope:
  - C16/C17 extraction only
  - split-contract updates enforcing delegation markers
- Out of scope:
  - production M10 changes
  - extracting additional sections in this issue
  - changing scenario semantics/assertions

## Failure modes

- FM-1: C16/C17 remain inline in root file.
- FM-2: root C16/C17 wrappers are missing.
- FM-3: split guard lacks orchestration-ordering delegation checks.
- FM-4: split-contract or archival suite fails.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `orchestration_ordering_cases.rs` exists and contains C16/C17 scenario bodies.
- [ ] AC-2: root `data_layer_m10_partition_archival.rs` keeps C16/C17 entrypoint names and delegates to orchestration-ordering cases functions.
- [ ] AC-3: split-contract test enforces C16/C17 delegation markers.
- [ ] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch

- `specs/6429-m10-orchestration-ordering-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/orchestration_ordering_cases.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics

- Test-only refactor. Fail-closed assertion behavior remains unchanged.

## Test plan

- RED:
  - add split-contract C16/C17 delegation markers and confirm failure before extraction.
- GREEN:
  - add orchestration-ordering cases module and move C16/C17 bodies.
  - convert root C16/C17 tests to wrappers.
  - run split-contract + archival suites.
- REFACTOR:
  - centralize orchestration-ordering marker arrays in split guard.
- INTEGRATION:
  - run required test lanes and record evidence.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
