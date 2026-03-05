# Spec: Issue #6405 - M10 Phase-6 validator extraction

## Objective

Extract deterministic M10 Phase-6 configuration validators (execution budget and scheduler policy) from `kamn-core` into `kamn-data-layer`, while preserving existing `kamn-core` API behavior through compatibility wrappers.

## Inputs/Outputs

- Inputs:
  - local validation helpers in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - existing policy evaluator contracts in `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
  - extraction markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - exported data-layer validator entrypoints for budget and scheduler policy config
  - core wrapper delegation mapping to existing core error variants
  - docs/test marker coverage proving extraction slice

## Boundaries/Non-goals

- In scope:
  - extracting validation ownership for budget and scheduler policy field checks
  - preserving reason-code/error parity for invalid config fields
  - keeping existing core public signatures unchanged
- Out of scope:
  - full scheduler cycle extraction
  - changing budget or trigger policy semantics
  - API-breaking changes for downstream callers

## Failure modes

- FM-1: validator entrypoints missing from data-layer exports.
- FM-2: core wrappers regress invalid-field mapping or reason codes.
- FM-3: docs marker coverage missing for validator extraction slice.
- FM-4: integration evidence commands not captured in spec.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `kamn-data-layer` exports budget and scheduler policy validator entrypoints.
- [ ] AC-2: `kamn-core` validator helpers delegate through extracted contracts with parity.
- [ ] AC-3: core M10 contract lane remains green, including invalid-budget/policy tests.
- [ ] AC-4: docs + data-layer policy contract tests include validator extraction markers/surface checks.
- [ ] AC-5: Phase 6 integration evidence recorded for docs lane, data-layer policy lane, and core M10 lane.

## Files to touch

- `specs/6405-m10-phase6-validator-extraction.md`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_policy_contract.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Data-layer validator entrypoints must fail closed with typed invalid-field errors.
- Core wrappers must map these onto existing `InvalidPhase6ExecutionBudget` and `InvalidPhase6SchedulerPolicy` variants with current reason codes.
- No silent fallbacks.

## Test plan

- RED:
  - add docs marker assertion for validator extraction slice.
  - extend data-layer policy contract test to call validator entrypoints.
- GREEN:
  - implement/export data-layer validator entrypoints.
  - update core validator helpers to delegate via extracted contracts.
  - update docs extraction marker rows.
- REFACTOR:
  - keep mapping/conversion helpers DRY.
- INTEGRATION:
  - run docs extraction test.
  - run data-layer policy contract test.
  - run core M10 partition archival contract lane.
  - capture pre/post telemetry timing for core M10 lane.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
