# Issue #4133 Spec

- Title: Task: add deterministic fuzz harness governance for message envelope and did parser surfaces
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Parser-risk surfaces need deterministic fuzz governance with bounded CI/runtime budgets and fail-closed provenance/taxonomy contracts.

## Acceptance Criteria
- AC-1: Deterministic fuzz harness contracts encode seed/corpus provenance markers.
- AC-2: Parser failure taxonomy markers are validated by contract tests.
- AC-3: CI smoke checks validate marker lineage while keeping deep lanes local-heavy.
- AC-4: Unit/functional/integration/regression checks for this governance surface pass.

## Scope
In scope:
- Child subtask #4139 (corpus drift + parser taxonomy contracts)
- Child subtask #4140 (seed provenance + bounded budget marker contracts)
- Parent task closeout metadata in `specs/4133/{spec.md,plan.md,tasks.md}`

Out of scope:
- New fuzz targets
- Always-on deep fuzz execution in CI
- External fuzz orchestration platforms

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | cargo-fuzz replay metadata | Deterministic provenance markers are present and versioned |
| C-02 | AC-2 | Regression | cargo-fuzz metadata taxonomy assertions | Drift fails closed on parser failure taxonomy markers |
| C-03 | AC-3 | Conformance | CI strategy marker contracts | CI-smoke/local-heavy boundary and provenance markers stay explicit |
| C-04 | AC-4 | Regression | targeted local test/lint evidence | Contract suites and lint/format checks pass |

## Test Mapping
- `cargo test -p kamn-core --test cargo_fuzz_target_contract`
- `cargo test -p kamn-core --test invariant_and_fuzz_strategy_docs`
- `cargo test -p kamn-core --test ci_strategy_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Deterministic seed provenance and parser taxonomy drift are fail-closed by tests.
- CI smoke boundaries remain explicit and bounded.
- Shell LOC delta across child delivery remains `0`.
