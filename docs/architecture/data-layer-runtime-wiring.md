# Data Layer Runtime Wiring (R65 / #5936)

schema_version=kamn.docs.architecture.data-layer-runtime-wiring.v1
last_updated=2026-02-25

## Objective

Wire Data Layer modules `M0..M11` into a real Service API runtime path so
`POST /v1/messages/send` executes each module contract and persists verifiable
runtime evidence with the message record.

## Runtime Path

1. `POST /v1/messages/send` enters `service_api_endpoint::middleware_impl`.
2. Route handler calls `ServiceApiMessageStore::create_message`.
3. `create_message` calls `build_data_layer_runtime_evidence(...)` before write.
4. Message state is persisted with
   `messages.<message_id>.data_layer_runtime_evidence`.

Fail-closed behavior:
- Any data-layer contract error aborts message creation and returns an internal
  persistence failure response.

## Module Contracts

| Module | Runtime operation used on send path | Persisted evidence field |
|---|---|---|
| M0 | Append + verify hash-chain in `DataLayerM0AppendOnlyLedger` | `m0_content_hash` |
| M1 | Assemble Merkle batch + verify inclusion proof | `m1_merkle_root` |
| M2 | DID auth + ABAC visibility + access-audit hash-chain | `m2_authorization_reason_code`, `m2_audit_record_hash` |
| M3 | Blind-index token derivation + owner-scoped search | `m3_blind_index_token`, `m3_match_count` |
| M4 | Escrow draft + funded transition | `m4_transition_reason_code` |
| M5 | Embedding append + owner integrity verification | `m5_record_hash` |
| M6 | Graph node/edge registration + portable projection export | `m6_projection_edge_count` |
| M7 | Telemetry ingest + owner observability evaluation | `m7_observability_health` |
| M8 | Compliance registration + retention due projection | `m8_retention_due_count` |
| M9 | Presence connect + dispatch acknowledgement projection | `m9_dispatch_ack_status`, `m9_dispatch_reason_code` |
| M10 | Partition registration + archival due projection | `m10_archived_partition_count` |
| M11 | Closure evidence evaluation | `m11_decision`, `m11_reason_codes_csv` |

## Persisted Evidence Schema

- message-level field:
  - `data_layer_runtime_evidence`
- schema marker:
  - `schema_version=kamn.runtime.service-api-data-layer-runtime-evidence.v1`

## Verification Coverage

- Unit/functional:
  - `service_api_endpoint::tests::integration_message_store_persists_data_layer_runtime_evidence_for_m0_to_m11`
- Integration:
  - `main_tests::service_api_endpoint_tests::integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11`
