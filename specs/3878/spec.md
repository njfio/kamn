# Issue #3878 Spec

- Title: Task: implement native runtime profile selector guardrails and fail-closed validation
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Profile activation paths can drift or accept invalid combinations without explicit guard coverage.

## Scope
In:
- Add profile selector validation for native and fallback paths.
- Add deterministic invalid-profile reason taxonomy checks.

Out:
- Activation lane rollout policy.

## Acceptance Criteria
- AC-1:  Invalid profile combinations are rejected with deterministic reasons.
- AC-2:  Selector behavior remains parity stable for valid profiles.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing.
- AC-4:  Performance overhead remains bounded.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Invalid profile combinations are rejected with deterministic reasons. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Selector behavior remains parity stable for valid profiles. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing. |
| C-04 | AC-4 | Unit/Functional/Integration/Regression | TBD in implementation task |  Performance overhead remains bounded. |

## Test Mapping
- To be completed in implementation phase for issue #3878.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
