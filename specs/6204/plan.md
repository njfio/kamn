# Plan: Issue #6204 - Begin kamn-core Split with Snapshot Journal Extraction

## Approach

1. Introduce `crates/kamn-snapshot-journal` with shared helper functions.
2. Add workspace membership + `kamn-core` dependency wiring.
3. Replace `crate::snapshot_journal` imports in snapshot consumers with crate imports.
4. Remove local `snapshot_journal` module from `kamn-core`.

## Affected Modules

- `Cargo.toml` (workspace members)
- `crates/kamn-snapshot-journal/*` (new)
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/task_operations.rs`

## Risks and Mitigations

- Risk: snapshot replay parsing regressions.
  - Mitigation: keep error taxonomy stable and run targeted snapshot-related tests.
- Risk: cross-crate API drift.
  - Mitigation: keep extracted API narrowly scoped to current call sites.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p kamn-snapshot-journal -p kamn-core -- -D warnings`
- `cargo test -p kamn-core message_lifecycle -- --nocapture`
- `cargo test -p kamn-core channel_models -- --nocapture`
- `cargo test -p kamn-core task_operations -- --nocapture`
