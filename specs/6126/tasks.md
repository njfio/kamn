# Tasks: Issue #6126

## Ordered Tasks
- [x] T1 (RED/Conformance): Added and ran failing RED tests proving delimiter/format limitations in pipe-delimited snapshot payloads:
  `regression_file_channel_snapshot_store_roundtrips_delimiter_rich_metadata`,
  `regression_file_task_operation_snapshot_store_roundtrips_delimiter_rich_description`,
  `regression_file_message_lifecycle_snapshot_store_roundtrips_delimiter_rich_message_id`,
  `regression_file_snapshot_store_uses_json_line_payload`,
  `regression_bundle_serialization_uses_json_payload`.
- [x] T2 (Implementation): Migrated channel/task/message/runtime/durable snapshot serialization to serde JSON; retained legacy pipe parser fallback to preserve upgrade-path reads.
- [x] T3 (GREEN/Regression): Re-ran all RED tests as GREEN with deterministic pass/fail assertions.
- [x] T4 (Verification): Ran scoped suites:
  `cargo test -p kamn-core --lib channel_models::tests::`,
  `cargo test -p kamn-core --lib task_operations::tests::`,
  `cargo test -p kamn-core --lib message_lifecycle::tests::`,
  `cargo test -p kamn-core --lib durable_guard_store::tests::`,
  `cargo test -p kamn-core --lib runtime::tests::runtime_tests_snapshot_store::`,
  `cargo fmt --check`,
  `cargo clippy -p kamn-core --tests -- -D warnings`.
- [x] T5 (Closure): Opened PR #6181 with AC->test mapping and RED/GREEN evidence; issue process log updated with verification outputs.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Integration: T4 (when cross-module behavior is affected)
- Regression: T1, T3, T4
- Conformance: T1, T4, T5
