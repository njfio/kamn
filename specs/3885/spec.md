# Issue #3885 Spec

- Title: Task: implement local-heavy native libp2p plus kolme interoperability matrix lane
- Status: Draft
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Real integration readiness requires deterministic multi-scenario evidence across native transport and runtime commit paths.

## Scope
In:
- Add triadic interoperability scenario runner.
- Emit deterministic interoperability artifact schema.

Out:
- Activation go-no-go policy enforcement.

## Acceptance Criteria
- AC-1:  Scenario runner covers defined interoperability matrix and emits stable markers.
- AC-2:  Artifact schema validation fails closed on missing markers.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing.
- AC-4:  Performance budget thresholds are checked.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Scenario runner covers defined interoperability matrix and emits stable markers. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Artifact schema validation fails closed on missing markers. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing. |
| C-04 | AC-4 | Unit/Functional/Integration/Regression | TBD in implementation task |  Performance budget thresholds are checked. |

## Test Mapping
- To be completed in implementation phase for issue #3885.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
