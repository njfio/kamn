# Spec: Issue #6419 - Extract M10 runtime-evidence tests into dedicated cases module

## Objective

Extract Phase-6 runtime-evidence test scenarios (`spec_c33`..`spec_c36`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into a dedicated `runtime_evidence_cases.rs` module while preserving root test entrypoints and assertion behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
  - existing Phase-6 runtime-evidence scenario tests C33-C36
- Outputs:
  - `crates/kamn-core/tests/data_layer_m10_partition_archival/runtime_evidence_cases.rs`
  - root file wrappers delegating C33-C36 to cases module
  - split-contract test guarding delegation markers

## Boundaries/Non-goals

- In scope:
  - runtime-evidence C33-C36 extraction only
  - delegation marker contract enforcement
- Out of scope:
  - production M10 code changes
  - full decomposition of the entire 1,991-line test file in one issue
  - changing scenario semantics or assertions

## Failure modes

- FM-1: C33-C36 stay inline in root file.
- FM-2: delegation wrappers are missing or incorrectly wired.
- FM-3: no split-contract guard exists, allowing regression.
- FM-4: split-contract or main M10 archival test lane fails.

## Acceptance criteria (testable booleans)

- [x] AC-1: `runtime_evidence_cases.rs` exists and contains C33-C36 scenario bodies.
- [x] AC-2: root `data_layer_m10_partition_archival.rs` retains C33-C36 test entrypoint names while delegating to cases module functions.
- [x] AC-3: split-contract test enforces runtime-evidence delegation markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch

- `specs/6419-m10-runtime-evidence-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/runtime_evidence_cases.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics

- Test-only refactor. Fail-closed behavior remains assertion-based.

## Test plan

- RED:
  - add split-contract test requiring runtime-evidence delegation markers and cases module presence.
  - confirm failure before extraction.
- GREEN:
  - add `runtime_evidence_cases.rs` and move C33-C36 bodies.
  - update root tests to delegate.
  - run split-contract + M10 archival lanes.
- REFACTOR:
  - deduplicate marker assertions in split-contract test.
- INTEGRATION:
  - run both required test lanes and record evidence.

## Phase 6 integration evidence

- `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` -> PASS (`1 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)

## Deviations

- None.
