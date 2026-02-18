# Issue #3879 Spec

- Title: Subtask: add native-fallback profile compatibility validation checks
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Compatibility rules must be explicit and enforced to avoid ambiguous startup behavior.

## Scope
In:
- Add compatibility checks for supported profile pairs.

Out:
- Rollback workflow implementation.

## Acceptance Criteria
- AC-1:  Unsupported profile pairs fail closed deterministically.
- AC-2:  Supported pairs pass with stable markers.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unsupported profile pairs fail closed deterministically. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Supported pairs pass with stable markers. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3879.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
