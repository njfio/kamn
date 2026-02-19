# Issue #3932 Spec

- Title: Subtask: add low-cost CI smoke fuzz/concurrency checks and local-heavy opt-in matrix runner
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Fuzz and concurrency contract lanes need explicit CI-smoke vs local-heavy governance markers in CI strategy docs, with fail-closed docs-contract coverage, so lane-cost boundaries cannot silently drift.

## Acceptance Criteria
- AC-1: CI strategy documents bounded CI-smoke and explicit local-heavy opt-in commands for invariant-fuzz-concurrency lanes.
- AC-2: Docs-contract tests fail closed if those governance markers drift.
- AC-3: Existing smoke/policy script tests for invariant-fuzz-concurrency lanes remain green.
- AC-4: Targeted Rust/doc lint gates pass with no shell-surface growth.

## Scope
In scope:
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/3932/{spec.md,plan.md,tasks.md}`

Out of scope:
- New runtime/parser production behavior
- New shell lane scripts or workflow expansions
- Heavy local matrix execution in CI-fast defaults

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | CI strategy section markers | CI-smoke/local-heavy commands and boundary text are present |
| C-02 | AC-2 | Regression | `ci_strategy_docs` contract test | Missing markers fail closed |
| C-03 | AC-3 | Integration | Invariant-fuzz-concurrency lane script tests | Contract lane and policy checks remain green |
| C-04 | AC-4 | Regression | fmt/clippy/shell guardrails | All gates pass, shell surface remains neutral |

## Test Mapping
- `cargo test -p kamn-core --test ci_strategy_docs`
- `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- CI strategy contains explicit, enforceable fuzz/concurrency lane governance markers.
- Docs-contract tests fail closed on marker drift.
- No shell LOC growth is introduced.
