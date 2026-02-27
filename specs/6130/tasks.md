# Tasks: Issue #6130

- T1 (Red): Add/update deterministic known-vector expectation test for `from_agent_name("Alice")`.
- T2 (Green): Refactor `derive_name_seed_bytes` to explicit byte-wise FNV-1a rounds.
- T3 (Regression): Run `kamn-agent-lib` identity-focused and crate-level test suites.
- T4 (Verify): Run fmt/clippy/tests and capture AC/C-case evidence in PR.
