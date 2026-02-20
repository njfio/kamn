# Issue #5287 Spec

- Title: Task: add deterministic archival failure-retry policy contracts for Phase-6
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 now has retention-to-archival gating from M8 into M10, but archival failure/retry behavior is not modeled explicitly. We need deterministic retry policy projection to keep archival execution recoverable and fail closed.

## Scope
In:
- Add deterministic M10 archival failure/retry projection with bounded backoff.
- Add stable reason markers for transient retry, retry-budget exhaustion, and permanent failure branches.
- Add conformance tests for retry scheduling and fail-closed terminal outcomes.
- Update Phase-6 trackers to current execution wave.

Out:
- Live object-storage upload integration.
- Distributed scheduler coordination.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 260
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Transient archival failures project deterministic retry windows under bounded exponential backoff.
- AC-2: Retry budget exhaustion and permanent failures fail closed with stable reason markers.
- AC-3: Unit/Functional/Regression tests for archival retry policy projection are present and passing.
- AC-4: `cargo fmt --check`, strict `clippy`, and targeted tests pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | transient failure attempt within retry budget | retry decision with deterministic bounded backoff and next-attempt projection |
| C-02 | AC-1 | Regression | repeated transient failures near cap | retry delay clamps to max-backoff without overflow/drift |
| C-03 | AC-2 | Functional | transient failure at/exceeding max attempts | fail-closed exhausted reason marker, no retry schedule |
| C-04 | AC-2 | Functional | permanent failure classification | immediate fail-closed permanent reason marker, no retry schedule |
| C-05 | AC-3 | Regression | invalid retry policy or attempt inputs | deterministic fail-closed input validation errors |
| C-06 | AC-4 | Verification | fmt/clippy + targeted M10 tests | all checks pass |

## Success Metrics
- Story `#5253` advances from gating-only to explicit archival retry/recovery contract behavior.
- No shell-surface growth while extending Phase-6 runtime contracts.
