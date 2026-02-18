# Issue #3877 Spec

- Title: Story: activate native libp2p runtime profiles with deterministic rollback controls
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Activation without deterministic profile checks and rollback evidence increases operational risk.

## Scope
In:
- Native profile selector validation and fail-closed guards.
- Cutover and rollback evidence lane contracts.

Out:
- Non-native runtime modes.

## Acceptance Criteria
- AC-1:  Profile selector and guardrails fail closed on invalid activation states.
- AC-2:  Rollback evidence lanes and policy checks are deterministic.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing.
- AC-4:  Performance governance remains bounded.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Profile selector and guardrails fail closed on invalid activation states. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Rollback evidence lanes and policy checks are deterministic. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing. |
| C-04 | AC-4 | Unit/Functional/Integration/Regression | TBD in implementation task |  Performance governance remains bounded. |

## Test Mapping
- To be completed in implementation phase for issue #3877.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
