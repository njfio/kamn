# Plan: Issue #6205 - Replace Pipe-Delimited Snapshot Journal Records with Serde JSON

## Approach

1. Implement serde-backed journal record struct in extracted snapshot-journal crate.
2. Write entries as one JSON object per line.
3. Parse entries via serde with schema + payload validation.
4. Keep caller corruption reason prefixes unchanged by preserving `None` on parse failure.

## Affected Modules

- `crates/kamn-snapshot-journal/src/lib.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/task_operations.rs`

## Risks and Mitigations

- Risk: parser incompatibility for existing format fixtures.
  - Mitigation: update targeted tests to use current writer output and preserve error codes.
- Risk: JSON parse overhead.
  - Mitigation: use small fixed record structure and line-by-line replay.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p kamn-snapshot-journal -p kamn-core -- -D warnings`
- `cargo test -p kamn-snapshot-journal -- --nocapture`
- `cargo test -p kamn-core message_lifecycle -- --nocapture`
- `cargo test -p kamn-core channel_models -- --nocapture`
- `cargo test -p kamn-core task_operations -- --nocapture`
