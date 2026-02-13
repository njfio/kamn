# Persistence Durability Model

This document defines the deterministic crash-recovery contract for file-backed snapshot stores that persist runtime-adjacent state in `kamn-core`.

## Scope

Applies to:

- `FileChannelSnapshotStore`
- `FileMessageLifecycleSnapshotStore`
- `FileTaskOperationSnapshotStore`

## Contract Markers

- `durability_schema_version=kamn.persistence.snapshot-journal.v1`
- `write_path=latest_snapshot_plus_append_only_journal`
- `replay_path=snapshot_base_plus_journal_tail`
- `journal_tail_corruption=fail_closed`
- `recovery_reason_codes=stable`

## Write Path

On each successful state write:

1. Validate snapshot semantics with the in-memory verifier for that domain.
2. Persist the latest snapshot payload to the snapshot file (truncate+replace).
3. Append a deterministic journal record to the companion journal file.

Journal entries are deterministic single-line records:

- `entry|1|<hex-encoded-snapshot-payload>`

## Replay Path

Startup replay resolves state in this order:

1. Parse and validate snapshot file payload as the base state (if present).
2. Parse and replay all journal records in order.
3. Return the latest valid replayed snapshot when journal entries exist.
4. Fall back to snapshot-only state when journal is absent/empty.

## Fail-Closed Rules

Corrupt or truncated journal tails do not auto-repair. Replay fails closed with deterministic reason text:

- `channel_snapshot_journal_corrupt_tail:<line>`
- `message_lifecycle_snapshot_journal_corrupt_tail:<line>`
- `task_operation_snapshot_journal_corrupt_tail:<line>`

Recovery result objects also publish deterministic reason codes:

- empty store: `*_snapshot_recovery_empty`
- clean recovery: `*_snapshot_recovery_clean`
- repaired corrupt snapshot payload: `*_snapshot_recovery_repaired_corrupt_payload`

## Startup Budget

Bounded performance checks for snapshot persistence/replay stay in fast CI:

- channel snapshot roundtrip budget: `< 300ms`
- message lifecycle snapshot roundtrip budget: `< 250ms`
- task operation snapshot roundtrip budget: `< 250ms`

## Evidence

- Journal replay regression lane:
  - `cargo test -p kamn-core --lib journal`
- Module-level durability lanes:
  - `cargo test -p kamn-core --lib channel_models::tests::`
  - `cargo test -p kamn-core --lib message_lifecycle::tests::`
  - `cargo test -p kamn-core --lib task_operations::tests::`

## Regression

- Regression: #2690
