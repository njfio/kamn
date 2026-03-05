# Spec: Issue #6409 - M10 Phase-6 scheduler cycle report extraction

## Objective

Extract deterministic Phase-6 scheduler cycle report projection (deferred/applied reason-code shaping + payload envelope) from `kamn-core` into `kamn-data-layer`, while preserving existing `kamn-core` public APIs and behavior.

## Inputs/Outputs

- Inputs:
  - scheduler cycle report assembly in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - existing phase6 policy contracts in `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
  - extraction markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - exported data-layer scheduler cycle report projector contract
  - core wrapper mappings between core trigger decision/report types and extracted contract types
  - docs/tests covering extraction markers and surface

## Boundaries/Non-goals

- In scope:
  - extracting deferred/applied scheduler cycle report projection with stable reason codes
  - preserving current core return structures and trigger decision behavior
  - keeping all `kamn-core` public signatures unchanged
- Out of scope:
  - full scheduler cycle execution extraction
  - trigger/budget policy semantic changes
  - API-breaking downstream changes

## Failure modes

- FM-1: scheduler cycle report projector contract not exported from data-layer.
- FM-2: core wrappers regress deferred/applied reason-code parity.
- FM-3: docs marker coverage missing for this extraction slice.
- FM-4: integration evidence not captured in spec.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `kamn-data-layer` exposes scheduler cycle report projector contracts.
- [ ] AC-2: `kamn-core` scheduler cycle path delegates report assembly through extracted contracts.
- [ ] AC-3: core M10 contract lane remains green, including deferred/applied scheduler-cycle assertions.
- [ ] AC-4: docs + data-layer policy contract tests include scheduler-cycle report extraction markers/surface checks.
- [ ] AC-5: Phase 6 integration evidence recorded for docs lane, data-layer policy lane, and core M10 lane.

## Files to touch

- `specs/6409-m10-phase6-scheduler-cycle-report-extraction.md`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_policy_contract.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Projector contract is deterministic and non-fallible for shaping outputs.
- Core wrappers must preserve existing error behavior for scheduler execution and budget checks.
- No fallback behavior.

## Test plan

- RED:
  - add docs marker assertions for scheduler-cycle report extraction slice.
  - extend data-layer policy contract test to call scheduler-cycle report projector surface.
- GREEN:
  - implement data-layer projector contracts.
  - replace inline core report assembly with wrapper delegation.
  - update docs markers.
- REFACTOR:
  - remove duplicated trigger-decision mapping helpers where possible.
- INTEGRATION:
  - run docs extraction test.
  - run data-layer policy contract test.
  - run core M10 partition archival lane.
  - capture pre/post telemetry for core M10 lane.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
