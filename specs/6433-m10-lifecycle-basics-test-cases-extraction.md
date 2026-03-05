# Spec: Issue 6433 - Extract M10 lifecycle baseline test cases

## Objective
Extract lifecycle baseline scenario tests (`spec_c01` through `spec_c05`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into a dedicated `lifecycle_basics_cases` module while keeping root entrypoints and behavior unchanged.

## Inputs/Outputs
- Inputs:
  - Existing inline C01-C05 tests in `data_layer_m10_partition_archival.rs`.
  - Existing split-contract guard in `data_layer_m10_partition_archival_split_contract.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m10_partition_archival/lifecycle_basics_cases.rs` holding C01-C05 scenario bodies.
  - Root C01-C05 tests delegating to the extracted module.
  - Split-contract markers for lifecycle-basics delegation/ownership.

## Boundaries/Non-goals
- No behavior changes to lifecycle naming, due selection, archive indexing, transition validation, or duplicate handling.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still contains inline C01-C05 scenario logic.
- Split-contract does not enforce lifecycle-basics delegation.
- Assertions or reason-code checks change during extraction.

## Acceptance criteria (testable booleans)
- [x] AC-1: `lifecycle_basics_cases.rs` exists and contains C01-C05 scenario bodies.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` retains C01-C05 entrypoints and delegates to `lifecycle_basics_cases` functions.
- [x] AC-3: split-contract test enforces C01-C05 delegation markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch
- `specs/6433-m10-lifecycle-basics-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/lifecycle_basics_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics
- Preserve existing fail-closed assertions and reason-code checks.
- Preserve existing panic/expect messages unless only formatting changes are required.

## Test plan
- Red:
  - Add split-contract lifecycle-basics markers before wiring module extraction; verify failure.
- Green:
  - Move C01-C05 bodies into `lifecycle_basics_cases.rs` and delegate root wrappers.
- Refactor:
  - Apply small no-behavior deduplication/constant cleanup in extracted module.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Phase 6 integration evidence
- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`8 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations
- None.
