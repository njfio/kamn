# Issue #3939 Spec

- Title: Task: add CI test-surface budget and ownership-marker governance checks
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Node test-surface decomposition requires deterministic budget and ownership governance to prevent regression.

## Acceptance Criteria
- AC-1: CI checker fails closed on test-surface budget drift.
- AC-2: Baseline refresh workflow and thresholds are documented.
- AC-3: Ownership/docs drift produces deterministic failure markers.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `#3946` budget checker + baseline fixture (`main_tests_surface_budget_contract`)
- `#3947` ownership/docs marker governance (`node_test_surface_ownership_docs`)
- docs marker updates in `docs/ci/strategy.md` and `docs/foundation/runtime-watchdog-attestation.md`

Out of scope:
- Shell/python governance expansion.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `main_tests_surface_budget_contract` | fails closed on threshold drift |
| C-02 | AC-2 | Integration | budget fixture + docs refresh workflow | deterministic baseline and update path |
| C-03 | AC-3 | Regression | ownership/docs contract suite | deterministic marker drift failures |
| C-04 | AC-4 | Regression | mapped suites and lint/format gates | all pass |

## Test Mapping
- `cargo test -p kamn-node --test main_tests_surface_budget_contract`
- `cargo test -p kamn-core --test node_test_surface_ownership_docs`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`
- `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Test-surface budget and ownership drift are blocked by deterministic governance contracts.
