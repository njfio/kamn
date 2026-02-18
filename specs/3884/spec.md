# Issue #3884 Spec

- Title: Story: prove live-node native libp2p and kolme interoperability readiness
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Activation confidence needs explicit live-node interoperability matrix and closure governance for artifacts, budgets, and documentation parity.

## Scope
In:
- Local-heavy native libp2p plus kolme interoperability matrix.
- Activation go-no-go budget and docs parity contracts.

Out:
- Mainnet rollout.

## Acceptance Criteria
- AC-1:  Interoperability matrix yields deterministic schema and taxonomy markers.
- AC-2:  Go-no-go contracts enforce activation readiness and budget thresholds.
- AC-3:  Documentation parity checks fail closed on drift.
- AC-4:  Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Interoperability matrix yields deterministic schema and taxonomy markers. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Go-no-go contracts enforce activation readiness and budget thresholds. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Documentation parity checks fail closed on drift. |
| C-04 | AC-4 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing. |

## Test Mapping
- To be completed in implementation phase for issue #3884.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
