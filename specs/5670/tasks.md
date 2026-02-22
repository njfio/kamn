# Tasks: #5670 Activate `kamn-cli` Execution for Supported Operations

- [x] T1 (Conformance/Functional): add RED tests for `health`, `list-messages`, and `verify-proof` command execution (C-01, C-02, C-03).
- [x] T2 (Unit): add RED tests for deterministic argument validation errors (C-02, C-03, C-04).
- [x] T3 (Implementation): add shared CLI handle bootstrap helper and implement command modules (AC-1, AC-2, AC-3, AC-4).
- [x] T4 (Regression): add/verify explicit unsupported command regression assertions (C-04).
- [x] T5 (Regression): run `cargo test -p kamn-cli`.
- [x] T6 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-cli -- -D warnings`.
- [x] T7 (Closeout): set `spec.md` status to Implemented and post RED/GREEN evidence on issue.
