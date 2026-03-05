# Spec: Issue #6393 - M10 archival retry extraction initial slice

## Objective

Ship a behavior-preserving initial extraction slice for M10 by moving archival retry decision projection implementation into `kamn-data-layer`, while keeping existing `kamn-core` API behavior stable through compatibility wrappers.

## Inputs/Outputs

- Inputs:
  - `kamn-core` M10 partition archival contracts in `data_layer_m10_partition_archival`
  - `kamn-data-layer` extracted-contract crate
  - architecture map doc `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - new `kamn-data-layer` exported M10 retry projection module
  - `kamn-core` retry compatibility wrapper delegating to `kamn-data-layer`
  - extraction markers + dependency blocker markers in architecture docs
  - pre/post telemetry for M10 retry contract lane

## Boundaries/Non-goals

- In scope:
  - extract retry decision algorithm and its dedicated retry reason taxonomy into `kamn-data-layer`
  - preserve existing `kamn-core` retry function behavior and error outcomes
  - document blocker preventing full M10 extraction in this issue (`M8` + `KamnDid` dependency seam)
- Out of scope:
  - full M10 registry/phase6 extraction
  - API-breaking changes to `kamn-core` exports
  - redesign of retry policy semantics

## Failure modes

- FM-1: `kamn-data-layer` does not expose extracted M10 retry projection surface.
- FM-2: `kamn-core` retry behavior or error mapping regresses after delegation.
- FM-3: extraction docs omit dependency blocker and follow-up requirement.
- FM-4: telemetry evidence is missing for pre/post retry lane execution.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `kamn-data-layer` exports M10 archival retry projection function and retry reason constants.
- [ ] AC-2: `kamn-core` retry contract tests remain green with compatibility wrapper delegation.
- [ ] AC-3: docs contract enforces M10 retry extraction markers and blocker marker.
- [ ] AC-4: pre/post telemetry is captured for `kamn-core` M10 retry contract lane.

## Files to touch

- `specs/6393-m10-archival-retry-extraction-slice.md`
- `docs/architecture/data-layer-runtime-wiring.md`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_archival_retry.rs` (new)
- `crates/kamn-data-layer/tests/data_layer_m10_archival_retry_integration.rs` (new)
- `crates/kamn-core/src/data_layer_m10_partition_archival/retry.rs`

## Error semantics

- `kamn-core` entrypoint must continue returning `DataLayerM10PartitionLifecycleError` for invalid retry policy/attempt inputs.
- Retry classification and reason codes must remain deterministic and fail closed.
- `kamn-data-layer` extracted module should use explicit typed errors with stable reason codes.

## Test plan

- RED:
  - add docs-contract markers for M10 retry extraction and dependency blocker note.
  - add `kamn-data-layer` integration test that imports and exercises extracted M10 retry surface.
- GREEN:
  - implement extracted M10 retry module in `kamn-data-layer`.
  - wire `kamn-core` retry wrapper to delegate and map errors without behavior drift.
  - update architecture markers for extraction + blocker.
- REFACTOR:
  - keep wrapper tiny and remove duplication in tests/helpers.
- INTEGRATION:
  - run `kamn-core` M10 retry/partition contract lanes.
  - run `kamn-data-layer` M10 retry integration lane.
  - capture pre/post telemetry for retry lane.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
