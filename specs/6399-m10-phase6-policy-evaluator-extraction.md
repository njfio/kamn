# Spec: Issue #6399 - M10 Phase-6 policy evaluator extraction

## Objective

Extract deterministic M10 Phase-6 policy evaluators (execution-budget and scheduler-trigger decisions) from `kamn-core` into `kamn-data-layer`, while preserving existing `kamn-core` public evaluator APIs through compatibility wrappers.

## Inputs/Outputs

- Inputs:
  - current core evaluator functions in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - existing M10 extraction map markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - new `kamn-data-layer` phase6 policy evaluator contracts
  - core wrappers that map core types/errors to extracted contracts
  - docs + contract tests proving extraction markers and behavior compatibility

## Boundaries/Non-goals

- In scope:
  - extracting evaluator logic for budget and scheduler trigger policies only
  - preserving current reason-code behavior and error classes
  - keeping existing public function signatures in `kamn-core`
- Out of scope:
  - full phase6 module migration
  - changing policy thresholds/precedence semantics
  - API-breaking changes for downstream callers

## Failure modes

- FM-1: data-layer evaluator contracts missing or not exported.
- FM-2: core wrappers regress reason codes or decision shapes.
- FM-3: docs markers not updated for extraction slice.
- FM-4: telemetry evidence missing for pre/post comparison.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `kamn-data-layer` exports phase6 evaluator contracts for budget and scheduler trigger.
- [ ] AC-2: `kamn-core` evaluator entrypoints delegate to extracted contracts with behavior compatibility.
- [ ] AC-3: existing M10 core contract suite remains green.
- [ ] AC-4: docs + data-layer contract tests enforce extraction markers/surface.
- [ ] AC-5: pre/post telemetry captured for `cargo test -p kamn-core --test data_layer_m10_partition_archival`.

## Files to touch

- `specs/6399-m10-phase6-policy-evaluator-extraction.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs` (new)
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_policy_contract.rs` (new)
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Data-layer evaluator errors stay typed and deterministic:
  - invalid budget field
  - invalid scheduler policy field
  - invalid scheduler signal field
- Core wrappers must map these onto existing core error variants and reason-code constants.
- No silent fallback or best-effort behavior.

## Test plan

- RED:
  - docs marker assertions for phase6 policy extraction slice.
  - data-layer contract test that imports evaluator surface.
- GREEN:
  - implement data-layer evaluator module + exports.
  - implement core wrapper delegation/mapping.
  - update docs markers.
- REFACTOR:
  - reduce duplication in wrapper conversion helpers.
- INTEGRATION:
  - run docs test, data-layer policy contract test, full core M10 contract lane.
  - capture pre/post telemetry.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
