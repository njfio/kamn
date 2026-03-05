# Spec: Issue 6441 - Extract M9 controls/backpressure test cases

## Objective
Extract controls/backpressure scenarios (`spec_c10` through `spec_c13`) from `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs` into a dedicated `controls_backpressure_cases` module while preserving root test entrypoints and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline C10-C13 tests in `data_layer_m9_realtime_delivery.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m9_realtime_delivery/controls_backpressure_cases.rs` containing C10-C13 scenario bodies.
  - Root C10-C13 wrappers delegating to `controls_backpressure_cases`.
  - New split-contract guard test enforcing delegation and marker ownership.

## Boundaries/Non-goals
- No behavior changes to anti-spam control outcomes, channel membership authorization semantics, or backpressure projection outcomes.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root contract still owns inline C10-C13 scenario logic.
- Split-contract guard fails to enforce delegated ownership markers.
- Extraction changes reason-code assertions or expected error variants.

## Acceptance criteria (testable booleans)
- [x] AC-1: `controls_backpressure_cases.rs` exists and contains C10-C13 scenario bodies.
- [x] AC-2: root `data_layer_m9_realtime_delivery.rs` retains C10-C13 entrypoints and delegates to `controls_backpressure_cases` functions.
- [x] AC-3: `data_layer_m9_realtime_delivery_split_contract.rs` enforces C10-C13 delegation/ownership markers.
- [x] AC-4: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` passes.

## Files to touch
- `specs/6441-m9-controls-backpressure-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery/controls_backpressure_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery_split_contract.rs` (new)

## Error semantics
- Preserve all existing fail-closed assertions and stable reason-code checks in C10-C13.

## Test plan
- Red:
  - Add split-contract guard markers for C10-C13 extraction before module wiring; verify failure.
- Green:
  - Move C10-C13 bodies into `controls_backpressure_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate literals/config setup in extracted module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract`
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`

## Phase 6 integration evidence
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` -> PASS (`1 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` -> PASS (`16 passed, 0 failed`)

## Deviations
- None.
