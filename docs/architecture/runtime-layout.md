# Runtime Layout

## `kamn-node` runtime/test module layout

`crates/kamn-node/src/main_tests.rs` is organized into domain-scoped modules:

- `cli_contract_tests` for startup/argument contract validation.
- `signer_tests` for signer adapter/key-source policy behavior.
- `runtime_tests` for core runtime and Kolme live execution behavior.
- `daemon_tests` for daemon-mode lifecycle and shutdown controls.
- `report_tests` for bootstrap/report rendering and deterministic output checks.
- `core_behavior_tests` compatibility wrappers for legacy test selectors still referenced by automation contracts.

Supporting runtime-focused modules remain scoped by responsibility:

- `cli.rs` for CLI/config parsing and mode resolution.
- `runtime_orchestration.rs` for runtime-mode dispatch, daemon/full phase orchestration, and signer policy contract enforcement.
- `runtime_kolme_live.rs` for Kolme live runtime orchestration.
- `daemon_shutdown.rs` and `daemon_observability.rs` for daemon lifecycle/telemetry.
- `report_builder.rs` and `report_render.rs` for runtime report shaping/rendering.
- `service_api_endpoint.rs` and `observability_endpoint.rs` for local API/observability surfaces.

This split keeps test ownership aligned with runtime domains while preserving backward-compatible selector coverage for existing contract lanes.

## Managed-Signer Rollout Governance

Managed-signer startup promotion/custody governance is validated through the contract lane:

- `scripts/kolme/contracts/managed_signer_startup_live_validation_contract_lane.py`
- `scripts/kolme/test_run_managed_signer_startup_live_validation_contract_lane.sh`

Deterministic fail-closed rollout markers include:

- `signer_rotation_promotion_stalled` for no-progress rotation gate drift.
- `quorum_evidence_custody_sha256_mismatch` for custody-audit parity drift.
- `ci_local_promotion_budget_boundary_status=verified` for bounded local/scheduled validation scope.

## Structured Logging Policy Governance

Structured logging schema/correlation drift is enforced through:

- `scripts/runtime/check_structured_logging_live_policy.sh`
- `scripts/runtime/validate_structured_logging_live_contract_lane.sh`

Deterministic lane markers include:

- `structured_logging_policy_status=verified`
- `structured_logging_contract_lane_status=verified`
- `correlation_id_parity_status=verified`
- `trace_classification_contract_status=verified`
- `log_classification_gate_status=verified`
- `reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1`
- `correlation_error_reason_taxonomy_version=kamn.runtime.correlation-error-reason-taxonomy.v1`
- `correlation_error_reason_codes_csv=correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped`

## Sqlite Crash-Recovery WAL Durability Governance

Sqlite crash-recovery WAL append/checkpoint taxonomy drift is enforced through:

- `scripts/runtime/check_sqlite_crash_recovery_live_policy.sh`
- `scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh`

Deterministic lane and policy markers include:

- `wal_append_status=verified`
- `wal_checkpoint_status=verified`
- `wal_durability_reason_taxonomy_version=kamn.runtime.wal-durability-reason-taxonomy.v1`
- `wal_durability_reason_codes_csv=wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete`
- `historical_query_index_status=verified`
- `historical_query_latency_budget_status=verified`
- `historical_query_reason_taxonomy_version=kamn.runtime.historical-query-reason-taxonomy.v1`
- `historical_query_reason_codes_csv=historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch`
- `crash_recovery_promotion_gate_status=verified`
- `audit_trail_parity_status=verified`
- `ci_local_promotion_budget_boundary_status=verified`
- `durability_governance_reason_taxonomy_version=kamn.runtime.durability-governance-reason-taxonomy.v1`
- `durability_governance_reason_codes_csv=crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded`
- `sqlite_crash_recovery_policy_status=verified`

## Reconciliation Consistency Governance

Snapshot-vs-WAL reconciliation and consistency-classification drift is enforced through:

- `scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh`
- `scripts/runtime/validate_block_reconciliation_partition_rejoin_live_contract_lane.sh`

Deterministic lane and policy markers include:

- `reconciliation_reason_taxonomy_status=verified`
- `snapshot_wal_reconciliation_status=verified`
- `consistency_classification_status=verified`
- `reconciliation_consistency_reason_taxonomy_version=kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1`
- `reconciliation_consistency_reason_codes_csv=snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch`

## Service API Protocol Compliance Governance

Service API ingress protocol-compliance and route-contract parity drift are enforced through:

- `scripts/runtime/check_service_api_axum_ingress_live_policy.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`

Deterministic lane and policy markers include:

- `protocol_compliance_status=verified`
- `route_contract_parity_status=verified`
- `protocol_compliance_reason_taxonomy_version=kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1`
- `protocol_compliance_reason_codes_csv=method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected`
