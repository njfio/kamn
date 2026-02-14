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

## Bootstrap Wiring and Startup Compatibility (Task #3068, Task #3078)

Bootstrap now validates prioritized runtime stores against durable file adapters before emitting a runtime plan:

- `content-storage:file-default` -> `content-store.snapshot`
- `did-registry:file-default` -> `did-chain-adapter.snapshot`
- `task-operation-snapshot-store:file-default` -> `task-operation.snapshot`
- `durable-guard-snapshot-store:file-default` -> `durable-guard.snapshot`
- `channel-snapshot-store:file-default` -> `channel.snapshot`
- `message-lifecycle-snapshot-store:file-default` -> `message-lifecycle.snapshot`
- `runtime-snapshot-store:file-default` -> `runtime.snapshot`

Bootstrap fail-closed compatibility error taxonomy:

- `ConfigError::RuntimeStoreCorruptPayload { store, reason_code, detail }`
- `ConfigError::RuntimeStoreSchemaIncompatible { store, reason_code, expected, found }`
- `ConfigError::RuntimeStoreCompatibility { store, reason_code, detail }`

Deterministic reason codes enforced for prioritized corruption/schema checks:

- `content_storage_corrupt_payload_rejected`
- `did_registry_corrupt_payload_rejected`
- `task_operation_snapshot_schema_mismatch_rejected`
- `durable_guard_snapshot_schema_mismatch_rejected`
- `channel_snapshot_corrupt_payload_rejected`
- `channel_snapshot_schema_mismatch_rejected`
- `message_lifecycle_snapshot_corrupt_payload_rejected`
- `message_lifecycle_snapshot_schema_mismatch_rejected`
- `runtime_snapshot_corrupt_payload_rejected`
- `runtime_snapshot_state_version_regression_rejected`

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
  - `restart_recovery_status=verified`
  - `corruption_fail_closed_status=verified`
  - `incompatible_schema_fail_closed_status=verified`
  - `fail_closed_status=verified`
  - `evidence_bundle_status=verified`
  - `execution_scope=local-scheduled`
  - `performance_budget_status=verified`
  - `fail_closed_reason_codes=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected`

Low-cost lane (PR-safe):

- `cargo test -p kamn-core --test content_storage_file_adapter content_storage_file_adapter_persists_round_trip_across_reopen`
- `cargo test -p kamn-core --test did_registry_file_chain_adapter did_registry_file_chain_adapter_persists_duplicate_detection_across_restart`
- `cargo test -p kamn-core --test content_storage_file_adapter content_storage_file_adapter_regression_rejects_corrupt_payload_line`
- `cargo test -p kamn-core --test did_registry_file_chain_adapter did_registry_file_chain_adapter_regression_rejects_corrupt_payload_line`
- `cargo test -p kamn-core --test task_operation_snapshot task_operation_snapshot_rejects_schema_version_mismatch`
- `cargo test -p kamn-core --test durable_guard_snapshot_store unit_bundle_schema_mismatch_is_rejected`
- `cargo test -p kamn-core --test task_operation_snapshot task_operation_snapshot_bounded_roundtrip_benchmark_is_fast_for_ci`
- `cargo test -p kamn-core bootstrap_wiring_includes_durable_store_components`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_content_store_payload_is_corrupt`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_task_snapshot_schema_is_incompatible`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_channel_snapshot_payload_is_corrupt`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_channel_snapshot_schema_is_incompatible`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_message_snapshot_payload_is_corrupt`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_message_snapshot_schema_is_incompatible`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_runtime_snapshot_payload_is_corrupt`
- `cargo test -p kamn-core regression_bootstrap_fails_closed_when_runtime_snapshot_state_version_regresses`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

Cost controls:

- No networked dependencies.
- No external database services.
- Deterministic file fixtures only.
- Runtime stays in sub-second range for targeted tests.
