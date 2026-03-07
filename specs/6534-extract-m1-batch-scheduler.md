# 6534 Extract M1 Batch Scheduler

## Objective
Extract the deterministic M1 batch scheduler policy surface from `kamn-core` into
`kamn-data-layer` while preserving the current `kamn-core` public API and behavior through
compatibility re-exports.

## Inputs/Outputs
- Inputs:
  - `DataLayerM1BatchSchedulerPolicy`
  - `&[DataLayerM1PendingBatchMessage]`
  - `now_unix_seconds`
- Outputs:
  - `DataLayerM1BatchTriggerDecision::{Deferred,Triggered}` with stable reason codes
  - typed fail-closed threshold and pending-message validation errors

## Boundaries/Non-goals
- In scope:
  - move the deterministic M1 scheduler types, reason codes, and evaluator into
    `crates/kamn-data-layer`
  - preserve `kamn-core` public imports and orchestrator behavior through compatibility shims
  - add dedicated `kamn-data-layer` scheduler coverage for happy-path and fail-closed inputs
  - record the new non-M10 extraction slice in docs/contracts
- Out of scope:
  - M1 anchoring orchestrator behavior changes
  - Merkle batch assembly changes
  - Kolme anchoring worker changes
  - persistence adapter changes
  - adding dependencies

## Failure Modes
- zero `max_messages_per_batch` fails closed
- zero `max_batch_window_seconds` fails closed
- empty pending `message_id` fails closed
- empty pending `content_hash` fails closed
- pending timestamp in the future fails closed

## Acceptance Criteria
- [ ] `kamn-data-layer` exports the deterministic M1 batch scheduler surface
- [ ] `kamn-core` preserves current public API and reason-code behavior through compatibility
      re-exports/shims
- [ ] dedicated `kamn-data-layer` tests cover deferred, count-threshold, window-threshold, and
      invalid-input paths
- [ ] existing `kamn-core` M1 scheduler and anchoring orchestrator tests remain green without
      public-contract changes
- [ ] extraction docs/contracts record the new M1 slice and CI remains green

## Files to touch
- `specs/6534-extract-m1-batch-scheduler.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m1_batch_scheduler.rs`
- `crates/kamn-data-layer/tests/data_layer_m1_batch_scheduler_integration.rs`
- `crates/kamn-core/src/data_layer_m1_batch_scheduler.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` must expose typed scheduler errors for invalid thresholds and invalid pending
  message inputs.
- `kamn-core` compatibility exports must preserve the current
  `DataLayerM1BatchSchedulerError::{InvalidThreshold,InvalidPendingMessage}` shapes and stable
  reason-code constants.
- No silent fallback or normalization is allowed.

## Test Plan
- Red:
  - add a `kamn-data-layer` scheduler test file importing the extracted surface before it exists
  - update the extraction docs contract with required M1 markers before docs are updated
- Green:
  - implement the extracted scheduler module in `kamn-data-layer`
  - replace the `kamn-core` file body with compatibility re-exports
- Refactor:
  - keep the extracted module within file-size limits and remove any duplicated scheduler logic
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, extraction-docs, and test-file-inventory lanes
