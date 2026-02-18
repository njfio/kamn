# Issue #5010 Spec

- Title: Story: M7 time-series telemetry, aggregates, and billing metrics
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M7 telemetry contracts for deterministic owner/agent metric ingestion,
aggregate rollups, and billing projections. Story delivery is completed through
child task `#5023`.

## Acceptance Criteria
- AC-1: Telemetry ingest contracts are implemented with deterministic bucket
  indexing and fail-closed validation.
- AC-2: Aggregate and billing projection contracts produce deterministic outputs
  with owner-scope enforcement.
- AC-3: Story maps to PRD M7 requirements with reproducible conformance
  evidence and shell/workflow/python/template LOC unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5023`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented lifecycle status.
- PRD M7 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M7 expansion beyond accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m7_timeseries_telemetry` ingest tests | Deterministic owner/agent ingest indexing behavior passes |
| C-02 | AC-2 | Conformance | Run aggregate + billing projection tests | Deterministic rollups/projections and owner-scope denies pass |
| C-03 | AC-3 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5010` closes with child task `#5023` merged and ACs mapped to passing deterministic tests.
- M7 telemetry contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
