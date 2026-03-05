# Spec: Issue #6401 - M10 Phase-6 runtime evidence extraction

## Objective

Extract deterministic M10 Phase-6 runtime evidence projection from `kamn-core` into `kamn-data-layer`, while preserving the existing `kamn-core` public API via compatibility wrappers.

## Inputs/Outputs

- Inputs:
  - core projector logic in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - current extraction markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - new `kamn-data-layer` runtime evidence projection contracts
  - core wrapper mappings between core phase6 types/errors and extracted contract types/errors
  - docs/test markers proving extraction surface and compatibility

## Boundaries/Non-goals

- In scope:
  - extracting runtime evidence projection shape/validation logic only
  - preserving existing applied/deferred evidence reason-code behavior
  - preserving deterministic archival ordering in evidence output
- Out of scope:
  - full phase6 orchestration/scheduler extraction
  - scheduler trigger/budget policy semantic changes
  - API-breaking changes for `kamn-core` downstream callers

## Failure modes

- FM-1: runtime evidence contract module missing from `kamn-data-layer` exports.
- FM-2: core wrapper mapping regresses reason codes or invalid-input behavior.
- FM-3: archival/object URI deterministic ordering regressions.
- FM-4: extraction docs markers/tests missing.
- FM-5: phase6 integration evidence commands not captured in spec.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `kamn-data-layer` exports a runtime evidence projection contract surface for M10 Phase-6.
- [ ] AC-2: `kamn-core::data_layer_m10_project_phase6_runtime_evidence_bundle` delegates through extracted contracts with behavior parity.
- [ ] AC-3: core M10 phase6 contract tests for applied/deferred evidence and fail-closed payload validation remain green.
- [ ] AC-4: docs + tests include runtime-evidence extraction markers.
- [ ] AC-5: Phase 6 integration evidence recorded for docs lane, new data-layer contract lane, and core M10 contract lane.

## Files to touch

- `specs/6401-m10-phase6-runtime-evidence-extraction.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_runtime_evidence.rs` (new)
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_runtime_evidence_contract.rs` (new)
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Data-layer projector must fail closed with typed invalid-input errors for malformed phase6 evidence payloads.
- Core wrapper must map extracted errors into existing `DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput` variant with stable reason codes.
- No silent fallback behavior is allowed.

## Test plan

- RED:
  - add docs marker assertions for runtime-evidence extraction slice.
  - add data-layer contract test that imports runtime-evidence surface and exercises applied/deferred/error paths.
- GREEN:
  - add `kamn-data-layer` runtime-evidence module + exports.
  - replace core projector body with compatibility wrapper and type/error mappings.
  - update docs extraction map markers.
- REFACTOR:
  - reduce mapping duplication in core wrapper helpers.
- INTEGRATION:
  - run docs extraction test.
  - run new `kamn-data-layer` runtime evidence contract test.
  - run core M10 contract lane.
  - capture pre/post telemetry timing for core M10 contract lane.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
