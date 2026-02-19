# Issue #4129 Spec

- Title: Story: enforce property-based state-machine invariants across task escrow and peer lifecycle domains
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Example-based tests were insufficient for lifecycle transition correctness; deterministic property-based invariant coverage was required for task, escrow, and peer domains.

## Acceptance Criteria
- AC-1: State-machine transition invariants are covered by property-based tests.
- AC-2: Deterministic seed controls reproduce failing cases.
- AC-3: Unit/Functional/Integration/Regression tests for this surface pass.
- AC-4: CI property-smoke coverage remains bounded/low-cost.

## Scope
In scope:
- Child task chain: `#4131` and `#4132`.
- Property helper, suite modularization, and discovery-parallel guardrails.
- `specs/4129/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Formal verification tooling adoption.
- Production protocol redesign.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `task_escrow_proptest_invariants` + `peer_lifecycle_proptest_invariants` | Transition invariants are enforced across task/escrow/peer domains |
| C-02 | AC-2 | Unit | `property_invariant_helpers_contracts` | Deterministic seed/helper contracts remain reproducible |
| C-03 | AC-3 | Regression | modularization + discovery-parallel contract suites | Coverage/discovery parity remains fail-closed and green |
| C-04 | AC-4 | Conformance | CI strategy marker contracts | Property suite smoke boundaries remain explicit and bounded |

## Test Mapping
- `cargo test -p kamn-core --test property_invariant_helpers_contracts`
- `cargo test -p kamn-core --test task_escrow_proptest_invariants`
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants`
- `cargo test -p kamn-core --test task_escrow_suite_modularization_contract`
- `cargo test -p kamn-core --test task_escrow_suite_discovery_parallel_contract`

## Success Metrics
- Transition-invariant coverage is explicit and deterministic.
- Property suite modularization and discovery contracts prevent silent drift.
- Story closure is traceable via spec artifacts and passing suites.
