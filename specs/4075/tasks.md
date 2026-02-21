# Issue #4075 Tasks

- T1 (Tests first): add retention fixture parser contract tests (unit/functional/integration/regression)
  and run RED against docs parity.
- T2 (Implementation): add retention fixture matrix and docs markers required by parser/docs tests.
- T3 (Verification): run targeted retention/docs tests, then `cargo fmt --check` and
  `cargo clippy -- -D warnings`.
- T4 (Process): publish issue process-log updates and open PR with AC/tier/TDD evidence.
