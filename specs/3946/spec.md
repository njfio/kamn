# Issue #3946 Spec

- Title: Subtask: implement node test-surface budget checker and baseline fixtures
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Without deterministic baseline-backed budgets, decomposed node test surfaces can regrow into monoliths undetected.

## Acceptance Criteria
- AC-1: A checker fails closed on node runtime test-surface budget drift.
- AC-2: Baseline fixture is deterministic and versioned in repo.
- AC-3: Docs define threshold values and baseline refresh workflow.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-node/tests/main_tests_surface_budget_contract.rs`
- `fixtures/ci/main_tests_runtime_surface_budget_baseline.json`
- `docs/ci/strategy.md`
- `specs/3946/spec.md`
- `specs/3946/plan.md`
- `specs/3946/tasks.md`

Out of scope:
- Shell/python checker additions.
- Production runtime behavior changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | budget contract test run | fails closed when any shell/module threshold is exceeded |
| C-02 | AC-2 | Unit | baseline fixture parse/schema checks | invalid/missing baseline fails with deterministic reason marker |
| C-03 | AC-3 | Integration | docs threshold section | includes taxonomy markers, fixture path, guard command, refresh workflow |
| C-04 | AC-4 | Regression | parity + docs suites | mapped tests pass |

## Test Mapping
- `cargo test -p kamn-node --test main_tests_surface_budget_contract`
- `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Runtime test-surface budget drift is blocked by deterministic baseline-backed checks.
