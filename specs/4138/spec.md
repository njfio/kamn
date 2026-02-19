# Issue #4138 Spec

- Title: Subtask: add regression checks for test discovery stability and parallel execution boundaries
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
The suite split delivered by #4137 improves maintainability, but the test surface can still drift silently if root/module wiring or deterministic parallel boundaries regress.

## Acceptance Criteria
- AC-1: Add deterministic regression checks that fail closed if modularized suite discovery markers drift.
- AC-2: Add regression checks that enforce bounded parallel-execution guardrails (seed isolation and bounded case budgets).
- AC-3: Document CI strategy markers for suite discovery stability and parallel boundaries.
- AC-4: Conformance and regression commands remain green for targeted suites.

## Scope
In scope:
- `crates/kamn-core/tests/task_escrow_suite_modularization_contract.rs`
- `crates/kamn-core/tests/task_escrow_suite_discovery_parallel_contract.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `specs/4138/{spec.md,plan.md,tasks.md}`

Out of scope:
- Production runtime behavior changes
- New shell/python/workflow executables
- Fuzz/concurrency heavy-lane additions

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | Read root+domain suite source markers | Contract test fails closed when module declarations/source markers drift |
| C-02 | AC-2 | Unit | Shared config constants/env key markers | Guardrails fail closed on duplicate seed env keys or unbounded case budgets |
| C-03 | AC-3 | Conformance | `docs/ci/strategy.md` marker assertions | Docs include deterministic discovery/parallel marker taxonomy |
| C-04 | AC-4 | Regression | Targeted contract + docs tests | All targeted commands pass |

## Test Mapping
- `cargo test -p kamn-core --test task_escrow_suite_discovery_parallel_contract`
- `cargo test -p kamn-core --test task_escrow_suite_modularization_contract`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_task_escrow_suite_discovery_and_parallel_contract_markers -- --exact`
- `cargo fmt --check`

## Success Metrics
- Discovery/parallel drift causes deterministic local failures before CI merge.
- No new shell scripts are added.
- CI strategy documentation and test contracts stay in sync.
