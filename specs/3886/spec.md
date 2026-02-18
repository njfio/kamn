# Issue #3886 Spec

- Title: Subtask: add triadic native libp2p plus kolme interoperability scenario runner
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Interoperability confidence needs deterministic triadic execution across churn and recovery cases.

## Scope
In:
- Add triadic runner and scenario matrix wiring.

Out:
- Go-no-go gating.

## Acceptance Criteria
- AC-1:  Triadic runner executes scenarios with stable markers.
- AC-2:  Scenario outcomes are deterministic across repeated runs.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Triadic runner executes scenarios with stable markers. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Scenario outcomes are deterministic across repeated runs. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3886.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
