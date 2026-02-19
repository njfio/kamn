# Issue #3935 Spec

- Title: Story: decompose node test monoliths and enforce test-surface governance
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Oversized node test monoliths and missing governance checks increase maintenance cost and drift risk.

## Acceptance Criteria
- AC-1: oversized node test surfaces are split into focused modules without behavior loss.
- AC-2: governance checks enforce test-surface budgets and ownership marker parity.
- AC-3: CI remains low-cost with deterministic fail-closed drift signals.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `#3938` decomposition and selector parity delivery (`#3944`, `#3945`)
- `#3939` budget/ownership governance delivery (`#3946`, `#3947`)

Out of scope:
- test framework replacement.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | runtime test decomposition shell/fragments | stable selector behavior |
| C-02 | AC-2 | Functional | command-surface + ownership docs contracts | deterministic drift detection |
| C-03 | AC-3 | Integration | low-cost rust governance tests | bounded CI-compatible checks |
| C-04 | AC-4 | Regression | mapped governance/docs suites | all pass |

## Test Mapping
- `cargo test -p kamn-node --test main_module_extraction_contract`
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`
- `cargo test -p kamn-node --test main_tests_surface_budget_contract`
- `cargo test -p kamn-core --test node_test_surface_ownership_docs`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`
- `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Node runtime test monoliths remain decomposed and governed by deterministic contracts.
