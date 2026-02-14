# Persistence Backends

## Scope

This document tracks durable persistence backends used by `kamn-core` and the validation strategy for low-cost CI.

Current execution slices are Task #2901 (core implementation) and Task #2903 (live validation) under Story #2900.

## Backends

| Surface | In-memory adapter | Durable adapter | Notes |
|---|---|---|---|
| Channel snapshots | `InMemoryChannelSnapshotStore` | `FileChannelSnapshotStore` | Snapshot + journal replay/repair |
| Message lifecycle snapshots | `InMemoryMessageLifecycleSnapshotStore` | `FileMessageLifecycleSnapshotStore` | Snapshot + journal replay/repair |
| Task operation snapshots | `InMemoryTaskOperationSnapshotStore` | `FileTaskOperationSnapshotStore` | Snapshot + journal replay/repair |
| Runtime snapshots | `InMemoryRuntimeSnapshotStore` | `FileRuntimeSnapshotStore` | Snapshot continuity guards |
| Durable guard bundles | `InMemoryDurableGuardSnapshotStore` | `FileDurableGuardSnapshotStore` | Bundle schema validation |
| Content objects | `InMemoryContentAdapter` | `FileContentAdapter` | Deterministic file payload with CID/integrity verification |
| DID chain submission idempotency | `InMemoryDidRegistrationChainAdapter` | `FileDidRegistrationChainAdapter` | Durable duplicate/reject state across restarts |

## New File-Backed Formats

### Content Store

- Schema header: `schema|kamn.content.file-store.v1`
- Record line: `object|<cid>|<media_type_hex>|<payload_hex>|<integrity_tag>`
- Deterministic behavior:
  - CIDs are recomputed from payload and must match persisted `cid`.
  - Integrity tags are recomputed and must match persisted `integrity_tag`.
  - Malformed records fail closed as `ContentStorageError::InvalidPayload`.

### DID Chain Adapter Store

- Schema header: `schema|kamn.did.chain-adapter.v1`
- Rejection line: `reject|<idempotency_key_hex>|<reason_hex>`
- Receipt line: `receipt|<idempotency_key_hex>|<provider_hex>|<transaction_id_hex>`
- Deterministic behavior:
  - Duplicate keys in persisted payload fail closed as `DidRegistryError::PersistenceInvalidPayload`.
  - Restarted adapters preserve duplicate-detection behavior for idempotency keys.

## Validation Matrix

Live validation lane (local realistic dependency path):

- `bash scripts/runtime/validate_persistence_adapters_live.sh --output-json /tmp/persistence-adapters-live.json`
- `bash scripts/runtime/test_validate_persistence_adapters_live.sh`
- Evidence schema: `kamn.persistence.adapters-live-validation.v1`
- Required markers:
  - `status=pass`
  - `final_decision=GO`
  - `content_persistence_status=verified`
  - `did_duplicate_detection_status=verified`
  - `fail_closed_status=verified`

Low-cost lane (PR-safe):

- `cargo test -p kamn-core --test content_storage_file_adapter`
- `cargo test -p kamn-core --test did_registry_file_chain_adapter`
- `cargo test -p kamn-core --test content_storage_adapter`
- `cargo test -p kamn-core --test did_registry_transactions`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

Cost controls:

- No networked dependencies.
- No external database services.
- Deterministic file fixtures only.
- Runtime stays in sub-second range for targeted tests.
