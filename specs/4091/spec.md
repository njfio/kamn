# Issue #4091 Spec

- Title: Subtask: add fail-closed quota checker and deterministic violation taxonomy tests
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Quota policy fixtures exist (`#4090`), but fail-closed checker behavior and deterministic violation taxonomy contracts are not yet codified for enforcement and drift detection.

## Acceptance Criteria
- AC-1: Checker fails closed on quota violations and malformed policy input.
- AC-2: Violation taxonomy markers remain deterministic and aligned with fixture-derived expectations.
- AC-3: Unit/Functional/Integration/Regression tests pass for checker/taxonomy scope.
- AC-4: CI strategy docs include quota checker taxonomy markers and validation command references.

## Scope
In scope:
- Add quota checker logic and deterministic reason taxonomy helpers in `kamn-core`.
- Add checker contract tests that compose with fixture semantics from `#4090`.
- Add CI strategy marker docs + docs-contract assertions.
- `specs/4091/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Scheduler redesign.
- Dynamic policy distribution.
- Shell/workflow additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Quota checker evaluation inputs | Invalid scope/window/limit and exceeded limits fail closed with deterministic reasons |
| C-02 | AC-2 | Regression | Taxonomy version + reason CSV checks | Taxonomy markers stay stable across checker and fixtures |
| C-03 | AC-3 | Integration | Checker tests composed with fixture semantics | Expected pass/fail outcomes remain deterministic |
| C-04 | AC-4 | Conformance | CI strategy docs marker assertions | Quota taxonomy markers and validation command are present |

## Test Mapping
- `cargo test -p kamn-core --test quota_policy_checker_contract`
- `cargo test -p kamn-core --test ci_strategy_docs -- doc_contains_quota_policy_checker_taxonomy_contract_markers --exact`
- `cargo test -p kamn-core --test quota_policy_fixture_parser_contract`

## Success Metrics
- Quota checker fail-closed behavior is deterministic and contract-tested.
- Taxonomy markers are shared and stable across fixture and checker surfaces.
- No shell script LOC growth in this subtask.
