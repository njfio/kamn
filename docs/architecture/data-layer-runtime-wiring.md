# Data Layer Runtime Wiring (R65 / #5936)

schema_version=kamn.docs.architecture.data-layer-runtime-wiring.v1
last_updated=2026-03-05

## M0-M11 Extraction Map (Issue #6379)

m0_m11_extraction_map_version=kamn.arch.data-layer-m0-m11-extraction-map.v1
m0_m11_extraction_sequence_csv=M0,M1,M2,M3,M4,M5,M6,M7,M8,M9,M10,M11
m1_batch_scheduler_extraction_slice_version=kamn.arch.data-layer-m1-batch-scheduler-extraction.v1
m1_batch_scheduler_target_crate=crates/kamn-data-layer
m1_batch_scheduler_compatibility_wrapper_path=kamn-core::data_layer_m1_batch_scheduler
m1_batch_scheduler_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m1_batch_scheduler.rs,crates/kamn-core/tests/data_layer_m1_anchoring_orchestrator.rs,crates/kamn-data-layer/tests/data_layer_m1_batch_scheduler_integration.rs
prd_critical_scenario_conformance_extraction_slice_version=kamn.arch.data-layer-prd-critical-scenario-conformance-extraction.v1
prd_critical_scenario_conformance_target_crate=crates/kamn-data-layer
prd_critical_scenario_conformance_compatibility_wrapper_path=kamn-core::data_layer_prd_critical_scenario_conformance
prd_critical_scenario_conformance_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_prd_critical_scenario_conformance.rs,crates/kamn-core/tests/data_layer_shell_neutral_policy.rs,crates/kamn-core/tests/data_layer_m11_closure_evidence.rs,crates/kamn-data-layer/tests/data_layer_prd_critical_scenario_conformance_integration.rs
m11_extraction_target_crate=crates/kamn-data-layer
m11_compatibility_shim_path=kamn-core::data_layer_m11_hardening_readiness
m11_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m11_hardening_readiness.rs,crates/kamn-core/tests/data_layer_m11_closure_evidence.rs,crates/kamn-data-layer/tests/data_layer_m11_hardening_readiness_integration.rs
m10_retry_extraction_slice_version=kamn.arch.data-layer-m10-retry-extraction.v1
m10_retry_extraction_target_crate=crates/kamn-data-layer
m10_retry_compatibility_wrapper_path=kamn-core::data_layer_m10_partition_archival::retry
m10_retry_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m10_partition_archival.rs,crates/kamn-data-layer/tests/data_layer_m10_archival_retry_integration.rs
m10_partition_month_policy_extraction_slice_version=kamn.arch.data-layer-m10-partition-month-policy-extraction.v1
m10_partition_month_policy_target_crate=crates/kamn-data-layer
m10_partition_month_policy_compatibility_wrapper_path=kamn-core::data_layer_m10_format_partition_name
m10_partition_month_policy_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m10_partition_archival.rs,crates/kamn-data-layer/tests/data_layer_m10_partition_month_policy_integration.rs
m10_partition_checksum_extraction_slice_version=kamn.arch.data-layer-m10-partition-checksum-extraction.v1
m10_partition_checksum_target_crate=crates/kamn-data-layer
m10_partition_checksum_compatibility_wrapper_path=kamn-core::data_layer_m10_partition_archival::shared::deterministic_checksum_marker
m10_partition_checksum_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m10_partition_archival.rs,crates/kamn-data-layer/tests/data_layer_m10_partition_month_policy_integration.rs
m10_partition_registry_state_machine_extraction_slice_version=kamn.arch.data-layer-m10-partition-registry-state-machine-extraction.v1
m10_partition_registry_state_machine_target_crate=crates/kamn-data-layer
m10_partition_registry_state_machine_compatibility_wrapper_path=kamn-core::DataLayerM10PartitionLifecycleRegistry
m10_partition_registry_state_machine_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m10_partition_archival.rs,crates/kamn-core/tests/data_layer_m10_partition_recoverability.rs,crates/kamn-data-layer/tests/data_layer_m10_partition_registry_state_machine_integration.rs
m10_projection_bookkeeping_extraction_slice_version=kamn.arch.data-layer-m10-projection-bookkeeping-extraction.v1
m10_projection_bookkeeping_target_crate=crates/kamn-data-layer
m10_projection_bookkeeping_compatibility_wrapper_path=kamn-core::DataLayerM10PartitionLifecycleRegistry::project_partition_shred_completeness_with_port
m10_projection_bookkeeping_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m10_partition_archival.rs,crates/kamn-data-layer/tests/data_layer_m10_compliance_projection_bookkeeping_integration.rs
m10_full_extraction_blocker_csv=data_layer_m8_compliance_lifecycle,KamnDid
m10_projection_port_seam_version=kamn.arch.data-layer-m10-projection-port.v1
m10_projection_port_trait_path=kamn-data-layer::DataLayerM10ComplianceProjectionPort
m10_projection_port_entrypoint=DataLayerM10PartitionLifecycleRegistry::project_partition_shred_completeness_with_port
m10_phase6_port_seam_version=kamn.arch.data-layer-m10-phase6-port.v1
m10_phase6_port_trait_path=kamn-data-layer::DataLayerM10Phase6CompliancePort
m10_phase6_orchestration_port_entrypoint=data_layer_m10_execute_phase6_orchestration_tick_with_port
m10_phase6_scheduler_port_entrypoint=data_layer_m10_execute_phase6_scheduler_cycle_with_port
m10_phase6_policy_extraction_slice_version=kamn.arch.data-layer-m10-phase6-policy-extraction.v1
m10_phase6_policy_target_crate=crates/kamn-data-layer
m10_phase6_policy_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6
m10_phase6_validator_extraction_slice_version=kamn.arch.data-layer-m10-phase6-validator-extraction.v1
m10_phase6_validator_target_crate=crates/kamn-data-layer
m10_phase6_validator_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6
m10_phase6_runtime_clock_extraction_slice_version=kamn.arch.data-layer-m10-phase6-runtime-clock-extraction.v1
m10_phase6_runtime_clock_target_crate=crates/kamn-data-layer
m10_phase6_runtime_clock_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6
m10_phase6_scheduler_cycle_report_extraction_slice_version=kamn.arch.data-layer-m10-phase6-scheduler-cycle-report-extraction.v1
m10_phase6_scheduler_cycle_report_target_crate=crates/kamn-data-layer
m10_phase6_scheduler_cycle_report_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6
m10_phase6_budget_overflow_projector_extraction_slice_version=kamn.arch.data-layer-m10-phase6-budget-overflow-projector-extraction.v1
m10_phase6_budget_overflow_projector_target_crate=crates/kamn-data-layer
m10_phase6_budget_overflow_projector_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6
m10_phase6_scheduler_preflight_extraction_slice_version=kamn.arch.data-layer-m10-phase6-scheduler-preflight-extraction.v1
m10_phase6_scheduler_preflight_target_crate=crates/kamn-data-layer
m10_phase6_scheduler_preflight_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6
m10_phase6_runtime_evidence_extraction_slice_version=kamn.arch.data-layer-m10-phase6-runtime-evidence-extraction.v1
m10_phase6_runtime_evidence_target_crate=crates/kamn-data-layer
m10_phase6_runtime_evidence_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6

| Milestone | Current ownership | Planned standalone ownership | Compatibility strategy |
|---|---|---|---|
| M0 | `kamn-core::data_layer_m0` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim until downstream import migration completes |
| M1 | `kamn-core::data_layer_m1*` + `kamn-data-layer::data_layer_m1_batch_scheduler` | `crates/kamn-data-layer` | batch scheduler extracted; keep `kamn-core` compatibility shim and contract-test parity gates while anchoring orchestrator and remaining M1 runtime surfaces stay in core |
| M2 | `kamn-core::data_layer_m2_gateway_access` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and runtime authz matrix contracts |
| M3 | `kamn-core::data_layer_m3_blind_index_search` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim with deterministic fixture parity |
| M4 | `kamn-core::data_layer_m4_escrow_integration` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and escrow transition contracts |
| M5 | `kamn-core::data_layer_m5_vector_integration` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and recall-drift contracts |
| M6 | `kamn-core::data_layer_m6_graph_integration` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and graph portability contracts |
| M7 | `kamn-core::data_layer_m7_timeseries_telemetry` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and rollup parity contracts |
| M8 | `kamn-core::data_layer_m8_compliance_lifecycle` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and retention/legal-hold contracts |
| M9 | `kamn-core::data_layer_m9_*` | `crates/kamn-data-layer` | keep `kamn-core` re-export shim and realtime dispatch contracts |
| M10 | `kamn-core::data_layer_m10_partition_archival` + `kamn-data-layer::data_layer_m10_archival_retry` + `kamn-data-layer::data_layer_m10_partition_month_policy` + `kamn-data-layer::data_layer_m10_partition_registry_state_machine` + `kamn-data-layer::data_layer_m10_compliance_projection_port` + `kamn-data-layer::data_layer_m10_compliance_projection_bookkeeping` + `kamn-data-layer::data_layer_m10_phase6_compliance_port` + `kamn-data-layer::data_layer_m10_phase6_policy_evaluator` | `crates/kamn-data-layer` | retry, partition month policy, deterministic checksum marker, registry state machine, compliance-projection bookkeeping, projection seam, and phase6 policy evaluators extracted; keep `kamn-core` compatibility wrappers while remaining M10 implementation ownership is migrated incrementally |
| M11 | `kamn-data-layer::data_layer_prd_critical_scenario_conformance` + `kamn-data-layer::data_layer_m11_hardening_readiness` + `kamn-core::data_layer_m11_closure_evidence` | `crates/kamn-data-layer` | PRD critical-scenario conformance and hardening are extracted; `kamn-core` keeps shims and closure API compatibility |

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
