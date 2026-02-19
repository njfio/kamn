# Issue #3887 Spec

- Title: Subtask: add interoperability artifact schema and marker policy checks
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Interoperability outputs need deterministic schema enforcement for release-grade evidence.

## Scope
In:
- Add schema contract and marker policy checker.

Out:
- Activation go-no-go integration.

## Acceptance Criteria
- AC-1:  Schema validation fails closed on missing fields.
- AC-2:  Marker taxonomy checks remain deterministic.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Schema validation fails closed on missing fields. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Marker taxonomy checks remain deterministic. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A). |

## Test Mapping
- C-01 -> planned artifact schema fail-closed tests
- C-02 -> planned marker taxonomy drift tests
- C-03 -> planned tier matrix verification in PR evidence

## Staleness Review (2026-02-19)
- Subtask scope remains valid and consistent with parent task #3885.
- No requirement drift detected; execution is pending parent scheduling.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
