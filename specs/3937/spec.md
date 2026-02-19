# Issue #3937 Spec

- Title: Task: enforce panic-path policy and docs-contract fail-closed governance
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Without policy and docs-contract governance, panic primitives can re-enter production paths and remediation guidance can drift from enforced behavior.

## Acceptance Criteria
- AC-1: Panic-path policy checker fails closed for prohibited patterns in scoped production files.
- AC-2: Docs-contract checks enforce panic-policy marker and remediation parity.
- AC-3: CI-fast compatible low-cost validation paths are documented and testable.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- Child subtask outputs:
  - `#3942` (`PR #5156`)
  - `#3943` (`PR #5157`)
- Runtime/docs checker artifacts:
  - `scripts/ci/check_no_production_expect.py`
  - `scripts/ci/test_check_no_production_expect.sh`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`

Out of scope:
- Additional panic-policy expansion beyond this scoped checker/docs lane

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | cfg(test)-prefix production violation fixture | checker emits fail-closed `production_expect_reachable` |
| C-02 | AC-2 | Regression | ci-strategy docs contract test for panic-policy markers | fails closed on missing marker/remediation entries |
| C-03 | AC-3 | Integration | checker harness + docs-contract suite | deterministic low-cost validation commands pass |
| C-04 | AC-4 | Regression | shell guardrails + docs/test suites from child PRs | all mapped verification remains green |

## Test Mapping
- `bash scripts/ci/test_check_no_production_expect.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_panic_path_policy_checker_markers_and_remediation_parity -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs`
- shell guardrails from `#3942` (`check_shell_loc_hard_ceiling`, `check_shell_rust_ratio_guardrail`, `check_shell_surface_threshold_ratchet`)

## Success Metrics
- Panic-path checker has no top-level cfg(test) truncation blind spot.
- Docs marker/remediation parity is enforced by contract tests.
- Parent task closure is fully traceable to child implementations.
