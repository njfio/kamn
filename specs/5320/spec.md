# Issue #5320 Spec

- Title: Hotfix: restore rustfmt conformance for `ci_strategy_docs` after #5319
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
`ci-fast-gate` on main is failing because `crates/kamn-core/tests/ci_strategy_docs.rs` is not rustfmt-conformant after #5319.

## Acceptance Criteria
- AC-1: `cargo fmt --all --check` passes for the touched file.
- AC-2: Targeted docs contract test still passes after formatting.
- AC-3: Mainline CI regression from #5319 is remediated with no behavior change.

## Scope
In scope:
- Formatting-only update in `crates/kamn-core/tests/ci_strategy_docs.rs`.
- Targeted verification commands.

Out of scope:
- Behavior changes to docs contract policy.
- Additional CI workflow changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | rustfmt check | no diff; exit success |
| C-02 | AC-2 | Functional | docs contract test | test pass |
| C-03 | AC-3 | Integration | PR fast gate rerun | workflow green |

## Test Mapping
- `cargo fmt --all --check`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_performance_baseline_provenance_contract_markers -- --exact`

## Success Metrics
- Main branch returns to green for the failed rustfmt gate.
