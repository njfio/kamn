# Tasks: Issue #6134

- T1 (Red): Add tests for unknown-flag rejection and `--` passthrough sentinel.
- T2 (Green): Implement parser changes to reject unknown flags and support `--` passthrough boundary.
- T3 (Regression): Run full `kamn-cli` test suite to validate no behavior regressions.
- T4 (Verify): Run fmt/clippy/tests and record AC/C-case evidence in PR.
