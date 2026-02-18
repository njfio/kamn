# Issue #3891 Spec

- Title: Subtask: add activation readiness and budget marker checks to go-no-go policy
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Promotion gating must enforce both readiness markers and cost boundaries deterministically.

## Scope
In:
- Add activation readiness and budget marker checks.

Out:
- Docs parity checks.

## Acceptance Criteria
- AC-1:  Missing readiness markers fail closed.
- AC-2:  Budget threshold violations fail with deterministic reason codes.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Missing readiness markers fail closed. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Budget threshold violations fail with deterministic reason codes. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- To be completed in implementation phase for issue #3891.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
