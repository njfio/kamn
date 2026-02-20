# Issue #5267 Spec

- Title: Task: implement M1 anchoring orchestrator tick and persistence-plan projection
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M1 now has scheduler thresholds and persistence APIs, but there is no deterministic orchestration contract that composes these pieces into one runtime tick projection for anchoring + persistence application.

## Scope
In:
- Add an M1 anchoring orchestrator module that composes:
  - scheduler trigger evaluation,
  - deterministic merkle batch assembly,
  - anchoring worker submission outcome,
  - persistence-plan projection for batch lifecycle writes.
- Add fail-closed validation for invalid timestamps and missing/invalid confirmation metadata for final receipts.
- Add integration coverage applying projected plans via existing PostgreSQL adapter lifecycle methods.

Out:
- Daemon job loops and thread orchestration.
- External service polling loops.
- New shell/python/workflow tooling.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 420
- shell_to_rust_ratio_delta_estimate: -0.0013
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Orchestrator returns deterministic `deferred`, `planned`, and `rejected` outcomes from the same input set.
- AC-2: Planned outcomes include deterministic persistence metadata (batch identity, assignments, submission, optional confirmation).
- AC-3: Final receipt outcomes fail closed when required confirmation metadata is missing or invalid.
- AC-4: Unit, Functional, Integration, and Regression tests for this slice pass with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | pending set below thresholds | deterministic deferred outcome |
| C-02 | AC-1/AC-2 | Functional | threshold-met pending set + submitted receipt | deterministic planned outcome with persistence plan |
| C-03 | AC-2 | Integration | projected plan applied through adapter methods | lifecycle write calls succeed deterministically |
| C-04 | AC-3 | Regression | final receipt without confirmation metadata | fail-closed error |
| C-05 | AC-4 | Verification | fmt/clippy + targeted tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test public_api_surface_policy`

## Success Metrics
- Runtime has a deterministic orchestration contract bridging scheduler, merkle anchoring, and persistence updates.
- Phase-3 implementation advances from independent components to integrated execution planning.
