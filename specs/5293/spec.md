# Issue #5293 Spec

- Title: Task: add Phase-6 scheduler-cycle trigger and guarded execution contracts
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 currently has deterministic orchestration tick execution (`#5289`) and post-tick budget evaluation (`#5291`), but lacks a scheduler-cycle contract that composes trigger policy, budget admission, and guarded execution in one deterministic boundary.

## Scope
In:
- Add deterministic Phase-6 scheduler trigger policy and decision taxonomy.
- Add scheduler-cycle guarded execution contract composing trigger + budget admission + execution.
- Add fail-closed preflight budget overflow path before execution side effects.
- Add conformance tests and ops-doc marker assertions.

Out:
- Long-running tokio scheduler daemon in `kamn-node`.
- Object-storage I/O or deployment workflow changes.
- Shell/python/workflow/template changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 260
- shell_to_rust_ratio_delta_estimate: -0.0004
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Scheduler trigger policy deterministically classifies `Deferred` vs `Triggered` for due-work threshold and interval cadence.
- AC-2: Deferred cycle returns explicit deferred reason marker and does not execute retention/archival mutations.
- AC-3: Budget-preflight overflow fails closed before orchestration tick execution.
- AC-4: Triggered cycle executes orchestration and emits deterministic budget evidence.
- AC-5: Unit/Functional/Integration/Regression coverage and verification commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | scheduler signal with due count below threshold and interval not elapsed | `Deferred` with stable deferred reason |
| C-02 | AC-1 | Unit | scheduler signal with due count at/above threshold | `Triggered` with due-threshold reason |
| C-03 | AC-1 | Unit | scheduler signal with interval elapsed and due count below threshold | `Triggered` with interval reason |
| C-04 | AC-2 | Functional | scheduler cycle evaluates deferred decision | cycle report indicates no execution and no mutations |
| C-05 | AC-3 | Regression | scheduler cycle with preflight budget overflow | fail-closed preflight error with stable reason marker |
| C-06 | AC-4 | Integration | scheduler cycle trigger + execution + budget contract with M8/M10 registries | execution report present + within-budget evidence marker |
| C-07 | AC-5 | Verification | fmt/clippy + targeted tests/docs-contract tests | all checks pass |

## Success Metrics
- Story `#5253` gains runtime-facing scheduler-cycle contract composition without shell-surface growth.
- Phase-6 promotion path moves from isolated functions to explicit scheduler boundary behavior.
