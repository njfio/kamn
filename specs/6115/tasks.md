# Tasks: Issue #6115

- T1 (Red): Add/adjust lifecycle tests that expect overdue validated messages to expire via policy transition.
- T2 (Green): Refactor expiry paths to use `transition(...Expired)` and update transition policy edges.
- T3 (Regression): Run `cargo test -p kamn-core message_lifecycle::tests::`.
- T4 (Verify): Run `cargo test -p kamn-core --lib` and record AC/C-case coverage in PR.
