# Issue #4140 Spec

- Title: Subtask: implement bounded deterministic fuzz runner contracts with seed provenance markers
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
`cargo-fuzz` seed corpus metadata and CI budget boundary markers exist, but the deterministic seed provenance contract is not explicit and fail-closed in tests.

## Acceptance Criteria
- AC-1: Cargo-fuzz seed metadata includes deterministic provenance markers for both parser targets.
- AC-2: CI strategy docs include explicit seed provenance marker contracts alongside bounded fuzz budgets.
- AC-3: Contract tests fail closed when provenance or bounded-budget markers drift.
- AC-4: Targeted verification commands pass without adding new shell scripts/wrappers.

## Scope
In scope:
- `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`
- `fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json`
- `docs/ci/strategy.md`
- `specs/4140/{spec.md,plan.md,tasks.md}`

Out of scope:
- New `cargo-fuzz` targets
- New shell scripts or CI workflow changes
- Production runtime behavior changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Seed corpus metadata file | Deterministic provenance markers are present for both targets |
| C-02 | AC-2 | Conformance | CI strategy doc markers | Bounded fuzz budget + provenance marker contracts are present |
| C-03 | AC-3 | Regression | Cargo-fuzz contract tests | Missing/tampered provenance markers fail closed |
| C-04 | AC-4 | Regression | Targeted checks | Tests/lint remain green with no shell-surface growth |

## Test Mapping
- `cargo test -p kamn-core --test cargo_fuzz_target_contract`
- `cargo test -p kamn-core --test ci_strategy_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Deterministic fuzz seed provenance is explicitly versioned and enforced by contract tests.
- CI budget and provenance governance remain bounded and fail closed.
- Shell LOC delta remains `0`.
