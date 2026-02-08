# Processor HA Runtime Contracts (Issues #354 / #357 / #360 / #363)

This document captures processor high-availability runtime contract text for snapshot restore guards and construct-lock safety rules.

## Scope Delivered
- Added processor HA docs contract baseline for runtime snapshot restore safeguards.
- Added construct-lock safety rules for split-brain and stale-lease boundaries.
- Added runtime snapshot restore guard model references:
  - `RuntimeSnapshot`
  - `SnapshotRestoreGuard`
  - `SnapshotRestoreError`
- Added runtime snapshot store model references:
  - `RuntimeSnapshotStore`
  - `InMemoryRuntimeSnapshotStore`
  - `FileRuntimeSnapshotStore`
  - `SnapshotStoreError`
- Added construct-lock guard model references:
  - `ConstructLockLease`
  - `ConstructLockGuard`
  - `ConstructLockError`
- Added low-cost validation lane commands for docs-focused PR checks.

## Snapshot Restore Rules
- Snapshot restore requires deterministic expected state version and expected state hash inputs.
- Snapshot payloads must preserve stable state lineage fields for restore decisions.
- snapshot version/hash mismatch restores are rejected.
- Typed restore mismatches:
  - `SnapshotRestoreError::StateVersionMismatch`
  - `SnapshotRestoreError::StateHashMismatch`

## Snapshot Store Adapter Rules
- Runtime snapshot adapters expose deterministic `write`, `read_latest`, and `list` behavior.
- File-backed snapshot entries serialize as `<state-version>|<state-hash>` per line.
- malformed snapshot payload entries are rejected.
- Typed adapter guard failures:
  - `SnapshotStoreError::InvalidPayload`
  - `SnapshotStoreError::Io`

## Construct Lock Rules
- Processor construct-lock ownership must enforce single active lease semantics.
- split-brain lock acquisition attempts are rejected.
- stale lease renewal attempts are rejected.
- Typed lock/fencing guard failures:
  - `ConstructLockError::LeaseAlreadyHeld`
  - `ConstructLockError::StaleFencingToken`

## Test Coverage Mapping
- Unit: N/A (docs-focused contract slice).
- Functional: docs section assertions for snapshot and lock rules.
- Integration: docs command mapping assertions for runtime docs test lane.
- Regression:
  - snapshot restore mismatch rejection (`Regression: #361`)
  - split-brain and stale-renew lock rejection (`Regression: #362`)
  - malformed file-backed snapshot payload rejection (`Regression: #387`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-node --test runtime_processor_ha_docs
cargo test -p kamn-node --test node_runtime_cli_docs
cargo test -p kamn-core snapshot_store
```

Then run strict formatting/lint gates:

```bash
cargo fmt --check
cargo clippy -p kamn-node -- -D warnings
```
