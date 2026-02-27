# Tasks: Issue #6122

- T1 (Red): Add/update regression expectations for shared helper paths (live flag parsing/replay marker/latency budget path) and capture failing state before extraction.
- T2 (Green): Implement shared helper module and replace duplicated helper definitions in all three drivers.
- T3 (Regression): Run `cargo test -p kamn-e2e-harness`.
- T4 (Verify): Run `cargo fmt --check` and `cargo clippy -p kamn-e2e-harness --tests -- -D warnings`.
