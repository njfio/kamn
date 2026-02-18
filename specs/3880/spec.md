# Issue #3880 Spec

- Title: Subtask: add invalid-profile fail-closed reason taxonomy regression checks
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Invalid profile rejection reasons must remain deterministic for operator debugging and policy gating.

## Scope
In:
- Add reason-code checks for invalid activation paths.

Out:
- Cutover evidence lanes.

## Acceptance Criteria
- AC-1:  Invalid-profile rejections emit stable reason taxonomy.
- AC-2:  Drift triggers deterministic regression failures.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Invalid-profile rejections emit stable reason taxonomy. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Drift triggers deterministic regression failures. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3880.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
