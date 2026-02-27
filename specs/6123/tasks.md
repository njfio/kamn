# Tasks: Issue #6123

- T1 (Red): Update non-collision regression expectations for equal-length payload hashes.
- T2 (Green): Implement payload-value fingerprint component in runtime identity helpers.
- T3 (Regression): Run `cargo test -p kamn-kolme --test runtime_request_identity_policy_contracts`.
- T4 (Verify): Run `cargo fmt --check`, `cargo clippy -p kamn-kolme --tests -- -D warnings`, `cargo test -p kamn-kolme`.
