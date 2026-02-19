# Issue #3947 Spec

- Title: Subtask: add docs-contract and ownership-marker governance tests for decomposed node tests
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Decomposition work can decay unless ownership markers and docs contracts fail closed when drift occurs.

## Acceptance Criteria
- AC-1: Docs/ownership drift is detected with deterministic marker assertions.
- AC-2: Ownership governance checks run via low-cost Rust test lanes.
- AC-3: Ownership marker taxonomy/status is documented in runtime-watchdog and CI strategy docs.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-core/tests/node_test_surface_ownership_docs.rs`
- `docs/foundation/runtime-watchdog-attestation.md`
- `docs/ci/strategy.md`
- `specs/3947/spec.md`
- `specs/3947/plan.md`
- `specs/3947/tasks.md`

Out of scope:
- New shell/python governance scripts.
- Runtime behavior changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | ownership docs-contract test | fails when runtime ownership markers/paths drift |
| C-02 | AC-2 | Integration | low-cost docs test command | ownership checks run in targeted Rust test lane |
| C-03 | AC-3 | Unit | docs marker assertions | deterministic taxonomy/status markers present |
| C-04 | AC-4 | Regression | docs-contract suites | all mapped tests pass |

## Test Mapping
- `cargo test -p kamn-core --test node_test_surface_ownership_docs`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`
- `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Node test ownership/docs drift is blocked via deterministic docs-contract tests.
