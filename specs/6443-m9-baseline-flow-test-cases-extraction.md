# Spec: Issue 6443 - Extract M9 baseline flow test cases

## Objective
Extract baseline delivery/presence/queue scenarios (`spec_c01` through `spec_c05`) from `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs` into a dedicated `baseline_flow_cases` module while preserving root entrypoints and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline C01-C05 tests in `data_layer_m9_realtime_delivery.rs`.
  - Existing split-contract guard in `data_layer_m9_realtime_delivery_split_contract.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m9_realtime_delivery/baseline_flow_cases.rs` containing C01-C05 scenario bodies.
  - Root wrappers for C01-C05 delegating to `baseline_flow_cases`.
  - Split-contract marker checks for C01-C05 delegation and ownership.

## Boundaries/Non-goals
- No behavior changes to ACK projection, presence visibility gating, owner scope validation, or queue full deferral semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline C01-C05 logic.
- Split-contract test fails to enforce C01-C05 extraction markers.
- Extraction alters reason-code assertions or expected error variants.

## Acceptance criteria (testable booleans)
- [x] AC-1: `baseline_flow_cases.rs` exists and contains C01-C05 scenario bodies.
- [x] AC-2: root `data_layer_m9_realtime_delivery.rs` retains C01-C05 entrypoints and delegates to `baseline_flow_cases` functions.
- [x] AC-3: split-contract test enforces C01-C05 delegation/ownership markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` passes.

## Files to touch
- `specs/6443-m9-baseline-flow-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery/baseline_flow_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery_split_contract.rs`

## Error semantics
- Preserve all fail-closed assertions and stable reason-code checks in C01-C05.

## Test plan
- Red:
  - Add split-contract C01-C05 markers before module extraction; verify failure.
- Green:
  - Move C01-C05 bodies into `baseline_flow_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate repeated literals in extracted module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract`
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`

## Phase 6 integration evidence
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` -> PASS (`2 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` -> PASS (`16 passed, 0 failed`)

## Deviations
- None.
