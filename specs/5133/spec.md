# Issue #5133 Spec

- Title: Task: fix strict clippy assertions_on_constants regressions in proptest budgets
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Strict clippy in `ci-fast-gate` fails deterministically on constant-expression assertions in proptest budget envelope tests.

## Acceptance Criteria
- AC-1: strict clippy passes for workspace and `kamn-core` manifests with `-D warnings`.
- AC-2: proptest budget envelope unit tests remain passing after the lint-safe rewrite.
- AC-3: no shell/workflow surface growth is introduced.

## Scope
In scope:
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
- `crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs`
- `specs/5133/{spec.md,plan.md,tasks.md}`

Out of scope:
- Changing case budget limits or property semantics
- CI workflow policy edits

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | strict clippy workspace+crate commands | exit code 0 |
| C-02 | AC-2 | Unit | budget envelope tests (task escrow + peer lifecycle) | both tests pass |
| C-03 | AC-3 | Regression | shell-surface guardrails | no shell surface regression |

## Test Mapping
- `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
- `cargo clippy --all-targets --all-features --manifest-path crates/kamn-core/Cargo.toml -- -D warnings`
- `cargo test -p kamn-core --test task_escrow_proptest_invariants unit_task_escrow_proptest_budget_envelope_is_bounded -- --exact`
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants unit_peer_lifecycle_proptest_budget_envelope_is_bounded -- --exact`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-5133.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-5133.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-5133.json`

## Success Metrics
- CI strict-clippy failure mode is removed without changing behavior budgets.
- `shell_loc_delta_actual` remains `0`.
