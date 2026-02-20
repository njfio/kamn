# Issue #5301 Spec

- Title: Task: add deterministic convergence evidence projection and promotion markers
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Story `#5254` requires promotion-time convergence evidence across schema drift, failure-path drift, concurrency drift, and budget gates. Existing contracts exist in isolation, but `kamn-node` report output does not project one deterministic convergence marker set for release gating.

## Scope
In:
- Add a deterministic convergence projection helper with fail-closed decision semantics.
- Project convergence taxonomy/version/reason markers into daemon bootstrap report output.
- Add conformance/regression tests and ops-doc marker coverage for promotion evidence.

Out:
- New CI workflow lanes.
- Shell/python/workflow/template changes.
- New persistence backends or protocol/wire-format changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 240
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Convergence projection emits deterministic taxonomy/version and reason-code markers for schema/error-path/concurrency/performance/cost drift classes.
- AC-2: Any failed convergence class yields fail-closed `NO-GO` projection markers.
- AC-3: Daemon bootstrap report output includes convergence projection markers.
- AC-4: Convergence fail-closed markers are regression-stable across repeated evaluation.
- AC-5: Unit/Functional/Integration/Regression verification commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | all convergence classes pass | decision marker is `go` with deterministic taxonomy/version/reason codes |
| C-02 | AC-2 | Functional | schema drift class fails | decision marker is `no_go` and includes schema fail reason code |
| C-03 | AC-2 | Functional | performance or cost budget class fails | decision marker is `no_go` and includes budget fail reason code |
| C-04 | AC-3 | Integration | daemon execution -> bootstrap report rendering | report includes convergence taxonomy/reason/decision markers |
| C-05 | AC-4 | Regression | repeated fail-closed projection for same failing input | deterministic marker set remains stable |
| C-06 | AC-5 | Verification | fmt/clippy + targeted convergence/report/docs tests | all pass |

## Success Metrics
- Story `#5254` gains concrete promotion-evidence markers wired into node report output.
- Release owners can read deterministic convergence status without shell-side ad hoc parsing.
- Shell surface remains unchanged while Rust runtime/report evidence expands.
