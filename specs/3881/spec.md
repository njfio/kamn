# Issue #3881 Spec

- Title: Task: deliver native transport cutover-rollback policy lane with CI governance
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Activation workflows need explicit evidence and policy checks to safely rollback when conditions fail.

## Scope
In:
- Implement cutover/rollback evidence lane.
- Add policy checker and CI exclusion tests.

Out:
- Live-node interoperability scenario expansion.

## Acceptance Criteria
- AC-1:  Cutover and rollback evidence lane emits deterministic artifacts.
- AC-2:  Policy checker fails closed on marker drift.
- AC-3:  CI exclusion and budget boundaries are enforced.
- AC-4:  Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Cutover and rollback evidence lane emits deterministic artifacts. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Policy checker fails closed on marker drift. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  CI exclusion and budget boundaries are enforced. |
| C-04 | AC-4 | Unit/Functional/Integration/Regression | TBD in implementation task |  Unit, Functional, Integration, and Regression tests are present and passing. |

## Test Mapping
- C-01 -> planned lane + artifact schema contract tests (to be defined in implementation PR)
- C-02 -> planned policy fail-closed drift tests (to be defined in implementation PR)
- C-03 -> planned CI exclusion/budget boundary tests (to be defined in implementation PR)
- C-04 -> planned tier matrix verification in PR evidence

## Staleness Review (2026-02-19)
- Scope and acceptance criteria remain valid for the active R26.4 milestone.
- Spec remains implementation-ready and is intentionally kept in planning until parent story sequencing resumes.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
