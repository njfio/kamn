# Issue #3943 Spec

- Title: Subtask: add docs-contract tests for panic-policy markers and remediation parity
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Panic-path policy checker value degrades if CI strategy docs drift from enforced checker commands, reason taxonomy markers, and remediation guidance.

## Acceptance Criteria
- AC-1: `docs/ci/strategy.md` contains deterministic panic-policy checker command/taxonomy markers and explicit remediation steps.
- AC-2: docs-contract tests fail closed when panic-policy markers/remediation entries drift.
- AC-3: Regression evidence demonstrates docs-contract parity is enforced in test runs.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/3943/spec.md`
- `specs/3943/plan.md`
- `specs/3943/tasks.md`

Out of scope:
- Checker implementation changes (`#3942`)
- CI workflow step wiring beyond docs-contract coverage

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | panic-policy section in `docs/ci/strategy.md` | contains checker commands, reason taxonomy markers, and remediation marker set |
| C-02 | AC-2 | Regression | `ci_strategy_docs` panic-policy docs test | fails closed if any required marker/remediation entry is missing |
| C-03 | AC-3 | Integration | run `cargo test -p kamn-core --test ci_strategy_docs` | panic-policy docs-contract test passes with existing suite |

## Test Mapping
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_panic_path_policy_checker_markers_and_remediation_parity -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Panic-policy docs and contract tests remain synchronized via deterministic markers.
- Remediation guidance parity is enforced by the docs test harness.
