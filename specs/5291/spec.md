# Issue #5291 Spec

- Title: Task: add Phase-6 execution-tick budget guardrail contracts
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 orchestration tick contracts exist (`#5289`) but there is no deterministic per-tick budget guardrail contract to classify workload size and fail closed on oversized execution reports.

## Scope
In:
- Add deterministic Phase-6 execution-tick budget input/report contracts.
- Add explicit reason-marker taxonomy for within-budget and exceeded branches.
- Add fail-closed invalid-budget validation.
- Add conformance tests and ops-doc contract markers.

Out:
- Runtime scheduler daemon integration in `kamn-node`.
- Shell/python/workflow/template changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 220
- shell_to_rust_ratio_delta_estimate: -0.0003
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Budget evaluator deterministically reports within-budget vs exceeded for Phase-6 orchestration report counters.
- AC-2: Exceeded branches are explicit for due candidates, shred operations, partition projections, and archive entries.
- AC-3: Invalid budget limits fail closed with stable reason marker.
- AC-4: Unit/Functional/Integration/Regression coverage and required verification commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | orchestration report under all limits | `WithinBudget` decision with stable within-budget reason marker |
| C-02 | AC-2 | Unit | due candidates over limit | `Exceeded` decision with due-candidates reason marker |
| C-03 | AC-2 | Unit | shred/projection/archive counts over limits | `Exceeded` decision with deterministic first-matched reason marker |
| C-04 | AC-3 | Regression | zero-valued budget limit fields | fail-closed invalid-budget error with stable reason marker |
| C-05 | AC-4 | Integration | orchestration tick output fed to budget evaluator | composed evaluation succeeds deterministically |
| C-06 | AC-4 | Verification | fmt/clippy + targeted test suites | all checks pass |

## Success Metrics
- Story `#5253` gains explicit bounded tick-budget guardrail contracts for scheduler integration.
- Shell-surface remains unchanged while extending Rust runtime contract coverage.
