# Issue #3882 Spec

- Title: Subtask: implement native cutover-rollback evidence bundle lane
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Activation readiness cannot be audited without deterministic bundle artifacts for cutover and rollback paths.

## Scope
In:
- Add cutover and rollback evidence bundle lane.

Out:
- Policy checker logic.

## Acceptance Criteria
- AC-1:  Evidence bundle lane emits stable schema markers.
- AC-2:  Bundle includes cutover and rollback outcome checkpoints.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Evidence bundle lane emits stable schema markers. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Bundle includes cutover and rollback outcome checkpoints. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3882.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
