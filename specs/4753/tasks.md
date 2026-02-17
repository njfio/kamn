# Tasks: Issue #4753

Status: Reviewed
Issue: #4753

## Ordered Tasks

T1 (RED)
- Confirm both production panic-path markers exist before implementation:
  - `crates/kamn-core/src/zk_message_proofs.rs` ranking `expect()`
  - `crates/kamn-core/src/kolme_runtime_commit/notifications_consumer.rs` connection-read
    `expect()`

T2 (GREEN)
- Implement typed-error ranking invariant in zk recommendation flow.
- Implement deterministic non-panicking connection-read handling in notifications consumer.

T3 (VERIFY)
- Run:
  - `cargo test -p kamn-core --test zk_message_proofs`
  - `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `rg -n \"non-empty option set should produce ranked list|connection should exist before read\" crates/kamn-core/src`
  - Returned both production-path `expect()` markers.

- GREEN command/output:
  - `cargo test -p kamn-core --test zk_message_proofs`
    - Passed (15 passed, 0 failed).
  - `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
    - Passed (7 passed, 0 failed).

- Regression summary:
  - `cargo fmt --check` passed.
  - `cargo clippy -p kamn-core -- -D warnings` passed.
