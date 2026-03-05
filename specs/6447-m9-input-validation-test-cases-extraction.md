# Spec: Issue 6447 - Extract M9 input-validation test cases

## Objective
Extract invalid-input taxonomy scenarios (`spec_c14` through `spec_c16`) from `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs` into a dedicated `input_validation_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline C14-C16 tests in `data_layer_m9_realtime_delivery.rs`.
  - Existing split-contract guard `data_layer_m9_realtime_delivery_split_contract.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m9_realtime_delivery/input_validation_cases.rs` containing C14-C16 scenario bodies.
  - Root wrappers for C14-C16 delegating to `input_validation_cases`.
  - Split-contract marker checks for C14-C16 delegation and ownership.

## Boundaries/Non-goals
- No behavior changes to invalid DID field taxonomy or reason-code assertions.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline C14-C16 logic.
- Split-contract does not enforce C14-C16 extraction markers.
- Reason-code or `field` assertion behavior changes during extraction.

## Acceptance criteria (testable booleans)
- [x] AC-1: `input_validation_cases.rs` exists and contains C14-C16 scenario bodies.
- [x] AC-2: root `data_layer_m9_realtime_delivery.rs` retains C14-C16 entrypoints and delegates to `input_validation_cases` functions.
- [x] AC-3: split-contract test enforces C14-C16 delegation/ownership markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` passes.

## Files to touch
- `specs/6447-m9-input-validation-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery/input_validation_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery_split_contract.rs`

## Error semantics
- Preserve fail-closed behavior and stable reason-code checks in C14-C16.

## Test plan
- Red:
  - Add split-contract C14-C16 markers before module extraction; verify failure.
- Green:
  - Move C14-C16 bodies into `input_validation_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate repeated literals in extracted module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract`
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`

## Phase 6 integration evidence
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` -> PASS (`4 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` -> PASS (`16 passed, 0 failed`)

## Deviations
- None.
