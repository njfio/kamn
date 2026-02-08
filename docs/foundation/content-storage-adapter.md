# Content Storage Adapter Contract (Issues #168 / #169)

This document captures the first implementation slice for PRD-aligned content-addressed storage contracts and integrity verification.

## Scope Delivered
- Added `crates/kamn-core/src/content_storage.rs` with:
  - `ContentStorageAdapter` trait covering `put`, `get`, `head`, and `verify`.
  - `InMemoryContentAdapter` for deterministic local/dev behavior.
  - `ContentHead` and `ContentObject` records for metadata and payload retrieval.
  - typed errors via `ContentStorageError`.
- Added integration and regression tests in `crates/kamn-core/tests/content_storage_adapter.rs`.

## CID and URI Contract
- Canonical CID format: `kamn:cid:v1:<16-hex-fnv1a64>`.
- Canonical content URI format: `kamn:content:v1:<cid>`.
- Helper APIs:
  - `content_uri_for_cid(cid)` validates and serializes URI form.
  - `cid_from_content_uri(uri)` validates URI prefix and decodes CID.

## Integrity Verification Rules
- `put` computes deterministic CID + integrity tag from payload bytes.
- `verify` recomputes expected CID/integrity from persisted payload and fails on mismatch.
- Corruption/tampering returns `ContentStorageError::IntegrityMismatch`.

## Task Artifact Integration Path
- Content URIs produced by the adapter integrate with `TaskArtifactRegistry` via `off_chain_uri`.
- `TaskArtifactRegistry::integrity_fingerprint(task_id, creator, off_chain_uri)` can use adapter-produced URIs without format translation.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test content_storage_adapter --test content_storage_adapter_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
