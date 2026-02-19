# Issue #4131 Spec

- Title: Task: add proptest invariants for task escrow and peer lifecycle transition correctness
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Transition correctness needed invariant-focused property coverage and deterministic seeds beyond example-only tests.

## Acceptance Criteria
- AC-1: Core transition invariants are encoded in property tests.
- AC-2: Deterministic seed controls reproduce failures.
- AC-3: CI-smoke property checks are bounded.
- AC-4: Unit/Functional/Integration/Regression tests pass for the task scope.

## Scope
In scope:
- Child subtasks `#4135` and `#4136`.
- Property helper contracts plus task/escrow/peer property suites.
- `specs/4131/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Formal-methods integration.
- Unbounded property budgets in CI.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `task_escrow_proptest_invariants` + `peer_lifecycle_proptest_invariants` | Invariants enforce legal transitions and reject illegal transitions deterministically |
| C-02 | AC-2 | Unit | `property_invariant_helpers_contracts` | Seed/helper contracts remain deterministic and reproducible |
| C-03 | AC-3 | Conformance | property suite CI-smoke commands | bounded property checks remain green |
| C-04 | AC-4 | Regression | aggregate targeted property suite runs | all mapped commands pass |

## Test Mapping
- `cargo test -p kamn-core --test property_invariant_helpers_contracts`
- `cargo test -p kamn-core --test task_escrow_proptest_invariants`
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants`

## Success Metrics
- Deterministic property invariants for task/escrow/peer transitions are enforced by contracts.
- Task closure is traceable to child implementation suites.
