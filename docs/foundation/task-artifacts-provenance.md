# Task Artifacts Integrity and Provenance (Issues #224, #225)

This document captures the first implementation slice for task artifact provenance with integrity references.

## Scope Delivered
- Added `crates/kamn-core/src/task_artifacts.rs` with:
  - `TaskArtifactRegistry` for deterministic artifact registration and lookup.
  - `TaskArtifactRecord` fields for artifact/task linkage, creator, timestamp, on-chain hash, off-chain URI, and content type.
  - `integrity_fingerprint(task_id, creator, off_chain_uri)` helper.
  - typed errors via `TaskArtifactError` for validation, duplicates, missing records, and integrity mismatch.
- Added integration tests in `crates/kamn-core/tests/task_artifacts.rs`.

## Validation Rules
- `artifact_id`, `task_id`, `on_chain_hash`, `off_chain_uri`, and `content_type` must be non-empty.
- `creator` must parse as `kamn:did:agent:*`.
- `created_at_unix` must be non-zero.
- `on_chain_hash` must match deterministic integrity fingerprint:
  - `fnv1a_hex(\"<task_id>|<creator>|<off_chain_uri>\")`

## Query Surfaces
- `artifact(artifact_id)` for direct lookup.
- `artifacts_for_task(task_id)` for task-linked listing.
- `artifacts_for_creator(creator)` for creator-linked listing.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test task_artifacts
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
