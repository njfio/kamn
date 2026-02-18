# Issue #3893 Spec

- Title: Subtask: add docs-contract and milestone-summary parity checks for activation closure
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Milestone closure must remain synchronized with documented activation markers and summary outputs.

## Scope
In:
- Add docs-contract and milestone-summary parity checks.

Out:
- Gate marker evaluation logic changes.

## Acceptance Criteria
- AC-1:  Docs or summary marker drift fails closed.
- AC-2:  Closure summary marker set remains deterministic.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Docs or summary marker drift fails closed. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Closure summary marker set remains deterministic. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3893.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
