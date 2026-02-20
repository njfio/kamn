# Issue #5299 Spec

- Title: Task: wire Phase-6 retention scheduler runtime into kamn-node daemon tick
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 M8/M10 contracts are implemented through runtime evidence projection (`#5297`), but daemon-mode runtime in `kamn-node` does not execute them. This leaves retention/shred/archive scheduling disconnected from node execution flow.

## Scope
In:
- Integrate Phase-6 scheduler runtime contract execution into daemon runtime orchestration.
- Emit deterministic daemon runtime markers for Phase-6 applied/deferred/fail-closed reason classes.
- Project Phase-6 runtime summary fields into bootstrap report output.
- Add conformance/regression tests and ops-doc marker assertions.

Out:
- Persistence backends, object-storage uploads, or network transport changes.
- Shell/python/workflow/template changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0004
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Daemon runtime executes a deterministic Phase-6 scheduler contract path and records runtime counters.
- AC-2: Daemon runtime projects deterministic Phase-6 reason markers for applied/deferred decisions.
- AC-3: Fail-closed Phase-6 scheduler failures project stable reason markers without panicking.
- AC-4: Runtime report output includes Phase-6 scheduler runtime summary markers.
- AC-5: Unit/Functional/Integration/Regression verification commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | daemon runtime with applied scheduler fixture | runtime counters increment and applied reason marker is projected |
| C-02 | AC-2 | Functional | daemon runtime with deferred scheduler fixture | deferred reason marker projected with zero archived artifacts |
| C-03 | AC-3 | Regression | daemon runtime helper with regressed scheduler clock | fail-closed reason marker projected and no panic |
| C-04 | AC-4 | Integration | daemon execution -> bootstrap report rendering | report includes Phase-6 runtime reason taxonomy + reason code + counters |
| C-05 | AC-5 | Verification | fmt/clippy + targeted daemon/report/docs tests | all pass |

## Success Metrics
- Story `#5253` transitions from contract-only Phase-6 coverage to node runtime integration.
- Runtime output gains deterministic Phase-6 reason/counter markers suitable for operator diagnostics.
- Shell surface remains unchanged while Rust runtime integration expands.
