# Issue #3945 Spec

- Title: Subtask: add deterministic test command-surface parity checks after module extraction
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
After runtime test decomposition, selector discoverability can drift silently if docs commands or routed test function names diverge.

## Acceptance Criteria
- AC-1: Deterministic parity checks fail closed when required runtime test command-surface selectors drift.
- AC-2: Parity checks emit stable reason markers for selector/docs drift categories.
- AC-3: `docs/ci/strategy.md` carries deterministic runtime command-surface parity markers and guard command references.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
- `docs/ci/strategy.md`
- `specs/3945/spec.md`
- `specs/3945/plan.md`
- `specs/3945/tasks.md`

Out of scope:
- Shell/python checker additions.
- Runtime behavior changes in `kamn-node` production code.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | required runtime selector inventory | parity contract fails when selector function symbol is missing |
| C-02 | AC-2 | Unit | parity contract reason marker mapping | deterministic reason marker appears for each drift class |
| C-03 | AC-3 | Integration | `docs/ci/strategy.md` runtime parity section | includes taxonomy/version/reason CSV and guard command markers |
| C-04 | AC-4 | Regression | targeted parity contract + docs contract run | mapped tests pass |

## Test Mapping
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`
- `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Selector drift is detected pre-merge with deterministic failure markers.
- Runtime test command references in docs remain synchronized with extracted runtime test modules.
