# Issue #5285 Spec

- Title: Task: start Phase-6 retention-to-archival gate execution contracts
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-5 realtime delivery is complete, but Phase-6 (`#5253`) has not yet started runtime execution bridging for retention-to-archival decisions. We need deterministic integration between M8 lifecycle eligibility and M10 archival gate preconditions.

## Scope
In:
- Add deterministic retention-to-archival gate projection coverage for legal-hold and crypto-shred preconditions.
- Add fail-closed reason mapping coverage for archival-denied scenarios.
- Update execution trackers to move current wave from Phase-5 closure to Phase-6 kickoff.

Out:
- Object storage export implementation.
- Partition movement orchestration and scheduler rollout.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 360
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Retention eligibility and legal-hold/crypto-shred preconditions project deterministically into archival gate decisions.
- AC-2: Archival-denied paths preserve stable fail-closed reason markers.
- AC-3: Unit/Functional/Integration/Regression coverage is added and passing for this Phase-6 slice.
- AC-4: `cargo fmt --check`, strict `clippy`, and targeted tests pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | M8 eligible + M10 candidate partition inputs | deterministic archival-eligible projection |
| C-02 | AC-1 | Integration | legal-hold active lifecycle input | archival projection denied with hold-preserving contract |
| C-03 | AC-1 | Integration | crypto-shred preconditions not satisfied | archival projection denied until lifecycle prerequisites complete |
| C-04 | AC-2 | Regression | denied archival scenarios | stable reason-code markers across runs |
| C-05 | AC-3 | Functional | runtime-facing policy projection helper invocation | expected status/outcome mapping |
| C-06 | AC-4 | Verification | fmt/clippy + targeted test commands | all checks pass |

## Success Metrics
- Story `#5253` moves from planned to active execution with deterministic contract-to-runtime coverage.
- No shell-surface growth while expanding Rust-side Phase-6 execution coverage.
