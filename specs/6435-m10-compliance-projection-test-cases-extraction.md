# Spec: Issue 6435 - Extract M10 compliance projection test cases

## Objective
Extract compliance projection and legal-hold scenario tests (`spec_c06` through `spec_c11`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into a dedicated `compliance_projection_cases` module while preserving root test entrypoints and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline C06-C11 tests in `data_layer_m10_partition_archival.rs`.
  - Existing split-contract guard in `data_layer_m10_partition_archival_split_contract.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m10_partition_archival/compliance_projection_cases.rs` for C06-C11 scenario bodies.
  - Root C06-C11 entrypoints delegating to `compliance_projection_cases` functions.
  - Split-contract markers that enforce C06-C11 delegation and ownership.

## Boundaries/Non-goals
- No behavior changes to compliance projection, owner DID scope checks, message lookup handling, or legal-hold semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still contains inline C06-C11 logic after extraction.
- Split contract does not enforce C06-C11 delegation markers.
- Extraction alters reason-code assertions or expected failure matches.

## Acceptance criteria (testable booleans)
- [x] AC-1: `compliance_projection_cases.rs` exists and contains C06-C11 scenario bodies.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` retains C06-C11 entrypoints and delegates to `compliance_projection_cases` functions.
- [x] AC-3: split-contract test enforces C06-C11 delegation markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch
- `specs/6435-m10-compliance-projection-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/compliance_projection_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics
- Preserve all fail-closed checks and stable reason-code assertions in C06-C11.
- Preserve existing expect/panic semantics where behavior is unchanged.

## Test plan
- Red:
  - Add split-contract marker enforcement for C06-C11 before adding module wiring; verify failing test.
- Green:
  - Move C06-C11 test bodies into `compliance_projection_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate repeated literals in extracted module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Phase 6 integration evidence
- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`9 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations
- None.
