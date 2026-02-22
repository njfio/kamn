# Tasks: #5672 Activate Remaining CLI Core Message/Task Operations

- [x] T1 (Conformance/Functional): add RED tests for register/send-message/create-channel/query-message/create-task command execution (C-01..C-05).
- [x] T2 (Unit): add RED missing-arg validation assertions for supported commands (C-02..C-05).
- [x] T3 (Implementation): implement command modules using shared CLI helpers (AC-1..AC-6).
- [x] T4 (Regression): verify unsupported command regressions remain explicit (C-06).
- [x] T5 (Regression): run `cargo test -p kamn-cli`.
- [x] T6 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-cli -- -D warnings`.
- [x] T7 (Closeout): set `spec.md` status to Implemented and post RED/GREEN evidence to issue.
