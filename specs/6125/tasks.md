# Tasks: Issue #6125

## Ordered Tasks
- [x] T1 (RED/Conformance): Added failing boundary contract test file `crates/kamn-core/tests/core_split_phase1_contract.rs`; RED command failed before implementation:
  `cargo test -p kamn-core --test core_split_phase1_contract -- --nocapture`.
- [x] T2 (Implementation): Created extracted crate `crates/kamn-snapshot-journal` and migrated shared snapshot journal helpers (path derivation, hex encode/decode, journal record parse); rewired `kamn-core` snapshot domains to consume new crate.
- [x] T3 (GREEN/Regression): Re-ran boundary contract and module journal replay/corrupt-tail regressions as GREEN.
- [x] T4 (Verification): Executed:
  `cargo test -p kamn-core --test core_split_phase1_contract -- --nocapture`,
  `cargo test -p kamn-snapshot-journal -- --nocapture`,
  `cargo test -p kamn-core --lib channel_models::tests::integration_file_channel_snapshot_store_replays_journal_when_snapshot_is_stale`,
  `cargo test -p kamn-core --lib channel_models::tests::regression_file_channel_snapshot_store_rejects_corrupt_journal_tail`,
  `cargo test -p kamn-core --lib task_operations::tests::integration_file_task_operation_snapshot_store_replays_journal_when_snapshot_is_stale`,
  `cargo test -p kamn-core --lib task_operations::tests::regression_file_task_operation_snapshot_store_rejects_corrupt_journal_tail`,
  `cargo test -p kamn-core --lib message_lifecycle::tests::integration_file_message_lifecycle_snapshot_store_replays_journal_when_snapshot_is_stale`,
  `cargo test -p kamn-core --lib message_lifecycle::tests::regression_file_message_lifecycle_snapshot_store_rejects_corrupt_journal_tail`,
  `cargo fmt --check`,
  `cargo clippy -p kamn-snapshot-journal -p kamn-core --tests -- -D warnings`.
- [x] T5 (Closure): Opened PR #6182 with AC->test mapping and RED/GREEN evidence; issue process log updated.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Integration: T4 (if cross-crate behavior is affected)
- Regression: T1, T3, T4
- Conformance: T1, T4, T5
