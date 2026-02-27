# Tasks: Issue #6131

- T1 (Red): Add failing tests for bounded duplicate retention, eviction behavior, and zero-capacity config rejection.
- T2 (Green): Add `max_seen_message_ids` config field and implement FIFO eviction for retained IDs.
- T3 (Regression): Update impacted anti-spam config literals and run targeted anti-spam tests in `kamn-core`.
- T4 (Verify): Run fmt/clippy/tests and capture AC/C-case evidence in PR.
