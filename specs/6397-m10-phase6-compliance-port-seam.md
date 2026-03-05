# Spec: Issue #6397 - M10 Phase-6 compliance port seam

## Objective

Unblock remaining M10 extraction by introducing a data-layer-owned Phase-6 compliance seam and routing core Phase-6 orchestration/scheduler compatibility entrypoints through that seam, while preserving existing `kamn-core` APIs.

## Inputs/Outputs

- Inputs:
  - `kamn-core` M10 Phase-6 orchestration/scheduler code in `data_layer_m10_partition_archival/phase6.rs`
  - existing M10 projection seam and extraction map docs markers
- Outputs:
  - new `kamn-data-layer` Phase-6 compliance port contracts
  - additive `*_with_port` M10 Phase-6 seam entrypoints in `kamn-core`
  - `kamn-core` M8-backed compatibility adapter routing legacy M8-argument entrypoints through seam
  - docs markers and test contracts for seam coverage

## Boundaries/Non-goals

- In scope:
  - Phase-6 seam for due-candidate lookup, crypto-shred mutation, owner-scope authorization, and projection message lookup
  - compatibility-preserving wrappers for existing orchestration/scheduler entrypoints
- Out of scope:
  - full M10 module migration to `kamn-data-layer`
  - changing M8 business semantics
  - API-breaking signature changes for existing public functions

## Failure modes

- FM-1: new Phase-6 seam trait is not exported by `kamn-data-layer`.
- FM-2: legacy entrypoints bypass seam and keep direct dependency logic inline.
- FM-3: reason-code/error behavior drifts on owner-scope, lookup, or invalid-input failures.
- FM-4: docs extraction markers do not record seam mitigation.
- FM-5: telemetry evidence missing for pre/post lane timing.

## Acceptance criteria (testable booleans)

- [x] AC-1: `kamn-data-layer` exports Phase-6 seam trait/models/errors.
- [x] AC-2: `kamn-core` adds additive `data_layer_m10_execute_phase6_orchestration_tick_with_port(...)` and `data_layer_m10_execute_phase6_scheduler_cycle_with_port(...)`.
- [x] AC-3: legacy entrypoints remain compatible and green by delegating to M8-backed adapter.
- [x] AC-4: docs/test contracts enforce new seam markers and seam behavior.
- [x] AC-5: pre/post telemetry captured for `cargo test -p kamn-core --test data_layer_m10_partition_archival`.

## Files to touch

- `specs/6397-m10-phase6-compliance-port-seam.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_compliance_port.rs` (new)
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_compliance_port_contract.rs` (new)
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Port errors are typed and fail-closed (`OwnerScopeViolation`, `LookupFailed`, `InvalidInput`).
- Core mapper preserves deterministic M10 reason code classes:
  - owner scope -> `DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE`
  - invalid/lookup path -> `DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE` (unless legal hold is explicitly detected in message state checks)
- No silent fallback paths.

## Test plan

- RED:
  - extend docs contract with Phase-6 seam markers.
  - add data-layer seam contract test for trait surface.
  - add core M10 seam test for `*_with_port` orchestration path.
- GREEN:
  - implement data-layer seam module + exports.
  - implement core M8 adapter + additive seam entrypoints.
  - route legacy entrypoints through seam adapter.
  - update docs markers.
- REFACTOR:
  - extract helper functions for request validation and due-candidate shredding/projection loops.
- INTEGRATION:
  - run docs contract, data-layer seam contract, full M10 core lane.
  - capture pre/post telemetry.

## Phase 6 integration evidence

- 2026-03-05: `cargo test -p kamn-core --test data_layer_m0_m11_extraction_docs` (pass)
- 2026-03-05: `cargo test -p kamn-data-layer --test data_layer_m10_phase6_compliance_port_contract` (pass)
- 2026-03-05: `cargo test -p kamn-core --test data_layer_m10_partition_archival` (pass)
- 2026-03-05 telemetry:
  - pre seam lane timing: `m10_phase6_port_pre_seconds=16.57`
    - command: `/usr/bin/time -f 'm10_phase6_port_pre_seconds=%e' -o /tmp/m10_phase6_port_pre.time cargo test -p kamn-core --test data_layer_m10_partition_archival --manifest-path /tmp/kamn-6397-pre/Cargo.toml`
  - post seam lane timing: `m10_phase6_port_post_seconds=0.12`
    - command: `/usr/bin/time -f 'm10_phase6_port_post_seconds=%e' -o /tmp/m10_phase6_port_post.time cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Deviations

- None.
