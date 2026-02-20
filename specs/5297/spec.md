# Issue #5297 Spec

- Title: Task: add Phase-6 runtime evidence bundle projection contracts
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 now includes stateless scheduler-cycle contracts (`#5293`) and stateful runtime checkpoint contracts (`#5295`), but lacks one canonical evidence-bundle projection combining cycle outcomes, budget evidence, archival artifacts, and runtime counters.

## Scope
In:
- Add deterministic Phase-6 runtime evidence bundle input/output contracts.
- Add projection function composing scheduler-cycle report + runtime state snapshot.
- Add fail-closed validation for invalid applied/deferred payload combinations.
- Add conformance tests and ops-doc marker assertions.

Out:
- Persistence adapter writes or transport/API integration.
- Async daemon wiring in `kamn-node`.
- Shell/python/workflow/template changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 220
- shell_to_rust_ratio_delta_estimate: -0.0003
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Applied scheduler cycle projects canonical evidence with trigger reason, budget reason, and deterministic archival artifact lists.
- AC-2: Deferred scheduler cycle projects canonical evidence with empty execution artifacts and deferred reason.
- AC-3: Invalid cycle payload combinations fail closed with stable reason marker.
- AC-4: Unit/Functional/Integration/Regression coverage and verification commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | applied cycle report + runtime state | evidence bundle with applied markers and deterministic artifact ordering |
| C-02 | AC-1 | Unit | applied cycle report with archived entries out of order | projected artifact names/uris sorted deterministically |
| C-03 | AC-2 | Functional | deferred cycle report + runtime state | evidence bundle has deferred reason and empty execution artifacts |
| C-04 | AC-3 | Regression | applied cycle missing execution/budget payload | fail-closed invalid evidence input reason marker |
| C-05 | AC-3 | Regression | deferred cycle including execution/budget payload | fail-closed invalid evidence input reason marker |
| C-06 | AC-4 | Verification | fmt/clippy + targeted tests + docs marker tests | all pass |

## Success Metrics
- Story `#5253` gains canonical runtime evidence projection contracts ready for persistence/API adapters.
- Shell surface remains unchanged while Rust runtime integration contracts expand.
