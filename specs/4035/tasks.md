# Issue #4035 Tasks

- T1 (Tests first): add RED docs-contract tests for dependency-license remediation marker coverage/parity in strategy + ops docs.
- T2 (Tests): add workspace checker regression test for deterministic multi-reason mismatch marker ordering/classification.
- T3 (Implementation): add dependency-license remediation marker blocks in `docs/ci/strategy.md` and `docs/ops/configuration.md`.
- T4 (Verification): run targeted RED/GREEN tests, workspace checker shell test lane, `cargo fmt --check`, and `cargo clippy -p kamn-core --tests -- -D warnings`.
- T5 (Process): prepare PR with AC mapping, TDD evidence, tier matrix, and shell-surface DoD markers.
