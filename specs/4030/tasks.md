# Issue #4030 Tasks

- T1 (Tests first): add dependency advisory fixture parser contract tests (unit/functional/
  integration/regression/performance) and run RED against missing strategy marker assertion.
- T2 (Implementation): add dependency advisory fixture matrix and parser/helper threshold mapping
  logic required by tests.
- T3 (Docs): update `docs/ci/strategy.md` and docs-contract assertions for fixture/threshold
  markers and guard commands.
- T4 (Verification): run targeted parser/docs tests plus `cargo fmt --check` and
  `cargo clippy -p kamn-core --tests -- -D warnings`.
- T5 (Process): update issue process log and prepare PR with AC mapping + test-tier evidence.
