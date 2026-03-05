# Spec: Issue #6411 - M10 Phase-6 budget overflow projector extraction

## Objective

Extract deterministic Phase-6 scheduler budget-overflow error projection (reason code + detail shaping for preflight and post-execution overflow) from `kamn-core` into `kamn-data-layer`, while preserving `kamn-core` behavior and API compatibility.

## Inputs/Outputs

- Inputs:
  - inline overflow error shaping in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - policy evaluator contracts in `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
  - extraction markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - exported data-layer overflow error projector contract
  - core wrapper delegation that preserves existing `Phase6SchedulerBudgetPreflightExceeded` semantics
  - docs/test marker updates proving extraction slice

## Boundaries/Non-goals

- In scope:
  - extracting overflow error payload shaping only
  - preserving reason-code and detail-string behavior for preflight and post-execution overflow paths
  - keeping core public signatures unchanged
- Out of scope:
  - changing budget decision or scheduler execution semantics
  - full scheduler execution extraction
  - API-breaking changes

## Failure modes

- FM-1: overflow error projector contract not exported.
- FM-2: core wrapper regresses reason code parity.
- FM-3: preflight/post-execution detail strings regress.
- FM-4: docs marker coverage missing.
- FM-5: integration evidence not captured.

## Acceptance criteria (testable booleans)

- [x] AC-1: `kamn-data-layer` exports scheduler budget overflow error projector contracts.
- [x] AC-2: `kamn-core` overflow error shaping delegates through extracted contracts.
- [x] AC-3: core M10 contract lane remains green for preflight and runtime preflight-overflow scenarios.
- [x] AC-4: docs + data-layer policy contract tests include overflow projector extraction markers/surface checks.
- [x] AC-5: Phase 6 integration evidence recorded for docs lane, data-layer policy lane, and core M10 lane.

## Files to touch

- `specs/6411-m10-phase6-budget-overflow-projector-extraction.md`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_policy_contract.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Projector contract returns deterministic overflow payload when decision is `Exceeded`; no payload for `WithinBudget`.
- Core wrapper maps projected payload to existing `DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded`.
- No silent fallback behavior.

## Test plan

- RED:
  - add docs marker assertion for budget-overflow projector extraction slice.
  - extend data-layer policy contract test to call overflow projector surface.
- GREEN:
  - implement/export data-layer overflow projector contracts.
  - replace inline core overflow payload shaping with wrapper delegation.
  - update docs markers.
- REFACTOR:
  - keep core conversion helpers DRY.
- INTEGRATION:
  - run docs extraction test.
  - run data-layer policy contract test.
  - run core M10 partition archival lane.
  - capture pre/post telemetry for core M10 lane.

## Phase 6 integration evidence

- `cargo test -p kamn-core --test data_layer_m0_m11_extraction_docs` -> PASS (`1 passed, 0 failed`)
- `cargo test -p kamn-data-layer --test data_layer_m10_phase6_policy_contract` -> PASS (`1 passed, 0 failed`)
- `cargo test -p kamn-core --test data_layer_m10_partition_archival` -> PASS (`38 passed, 0 failed`)
- Timed core lane (current branch):
  - `m10_phase6_budget_overflow_post_seconds=0.12`
- Timed core lane (baseline worktree at RED commit `2255b14e`):
  - `m10_phase6_budget_overflow_pre_seconds=16.42`

## Deviations

- None.
