# Issue #3883 Spec

- Title: Subtask: add policy checker and CI exclusion tests for native cutover lane
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Cutover policy must fail closed and remain out of fast CI to preserve cost and determinism.

## Scope
In:
- Add policy checker and CI-fast exclusion checks.

Out:
- Additional cutover scenario types.

## Acceptance Criteria
- AC-1:  Policy checker fails on missing or drifted cutover markers.
- AC-2:  CI selection excludes heavy cutover lane from fast gate.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Policy checker fails on missing or drifted cutover markers. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  CI selection excludes heavy cutover lane from fast gate. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3883.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
