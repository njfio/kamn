# Off-Chain Retention, Tombstones, and Cleanup Lifecycle Controls (Issues #162 / #163)

This document captures the first implementation slice for retention-class lifecycle controls and deterministic cleanup handling.

## Scope Delivered
- Added `crates/kamn-core/src/content_lifecycle.rs` with:
  - `ContentRetentionClass` and `ContentRetentionProfile`.
  - `ContentLifecycleManager` for registration, tombstoning, cleanup planning, and cleanup execution.
  - `ContentLifecycleStatus` and `ContentCleanupActionKind`.
  - URI accessibility enforcement via `assert_uri_accessible(...)`.
  - typed errors via `ContentLifecycleError`.
- Added integration and regression tests in `crates/kamn-core/tests/content_retention_tombstones.rs`.

## Lifecycle and Cleanup Rules
- Retention classes map to deterministic retention and tombstone windows.
- Lifecycle progression:
  - `Active` -> `Expired` -> `Tombstoned` -> `Purged`
- Cleanup planning:
  - expired content yields deterministic tombstone actions.
  - tombstoned content past `purge_after_unix` yields purge actions.
- Cleanup safety:
  - purge operations require tombstone retention window to elapse.
  - no-op cleanup calls return explicit `NoCleanupDue` errors.

## Deleted Reference Semantics
- `assert_accessible(...)` blocks access for `Expired`, `Tombstoned`, and `Purged` records with explicit typed errors.
- `assert_uri_accessible(...)` parses canonical content URI and enforces the same lifecycle gate.
- Deleted/tombstoned references remain blocked under replay attempts.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test content_retention_tombstones --test content_retention_tombstones_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
