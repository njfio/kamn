# Spec: Issue 6191 - Extract Shared Snapshot/Journal Helpers

- Issue: #6191
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

Snapshot stores duplicated journal helper logic (journal path derivation, entry encoding,
entry parsing, hex decode) across message lifecycle, channel models, and task operations.

## Scope

In scope:
1. Extract shared snapshot-journal utilities into one core module.
2. Replace duplicated helper implementations in the three snapshot-store modules.
3. Preserve corrupt-tail detection behavior and recovery semantics.

Out of scope:
1. Rewriting durable guard or runtime snapshot store wire formats.
2. Altering snapshot payload schema contracts.

## Acceptance Criteria

### AC-1 Shared Helper Extraction
Given snapshot journal helper logic,
When building snapshot stores,
Then common helper code is sourced from a shared module.

### AC-2 Behavioral Parity
Given corrupt or malformed journal entries,
When replay/recovery paths run,
Then existing corrupt-tail rejection behavior remains unchanged.

### AC-3 No New Warnings
Given the extraction changes,
When running formatting and clippy gates,
Then the crate remains clean.

## Conformance Cases

- C-01 (AC-1, Unit): `crates/kamn-core/src/snapshot_journal.rs` provides shared helper primitives.
- C-02 (AC-2, Unit): `regression_file_message_lifecycle_snapshot_store_rejects_corrupt_journal_tail`, `regression_file_channel_snapshot_store_rejects_corrupt_journal_tail`, `regression_file_task_operation_snapshot_store_rejects_corrupt_journal_tail`.
- C-03 (AC-3, Verify): `cargo fmt --check` and `cargo clippy -p kamn-core -- -D warnings`.
