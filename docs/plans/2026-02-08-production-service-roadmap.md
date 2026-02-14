# KAMN: From Protocol Spec to Production Service

## Current State (honest assessment)

What you have is **88K lines of Rust that define the rules** — domain types, state machines, validation guards, contract tests — plus a working HTTP/WebSocket client for Kolme blockchain integration and a localhost TCP demo. What you don't have is anything that runs as a service.

There is **zero async code** in the entire codebase. No tokio, no `async fn`, no `.await`. All networking is synchronous blocking I/O. There is no database. There is no HTTP server. There is no live libp2p network layer yet; current p2p coverage is deterministic in-memory transport contracts. Every store is `InMemory*` with no persistence.

---

## Execution Status Update (2026-02-14)

- Phase 1.1 delivered: `kamn-node` now runs with a tokio runtime boundary (Story #2890).
- Phase 1.2 delivered: daemon shutdown now handles real OS signals on tokio signal streams (Story #2895).
- Phase 1.3 in progress:
  - Delivered in this slice: `FileContentAdapter` and `FileDidRegistrationChainAdapter` in `kamn-core` (Task #2901).
  - Live validation lane added for persistence adapter restart and fail-closed checks (Task #2903).
  - Remaining: broader persistence backend consolidation and runtime wiring across additional stores.
- Post-roadmap hardening wave 3 persistence live-validation expansion delivered:
  - Runtime lane: `scripts/runtime/validate_persistence_adapters_live.sh` and `scripts/runtime/test_validate_persistence_adapters_live.sh` (Task #3068, Subtask #3070).
  - Follow-on bootstrap coverage expansion delivered (Task #3078) for remaining snapshot stores.
  - Lane reason-code matrix and harness expectations expanded to include new bootstrap snapshot reason codes (Task #3080).
  - Runtime `kolme-live` report-contract assertions now fail closed if any durable bootstrap component marker is missing (Task #3082).
  - Bootstrap/runtime wiring now defaults prioritized persistence surfaces to durable file adapters in plan components:
    - `content-storage:file-default`
    - `did-registry:file-default`
    - `task-operation-snapshot-store:file-default`
    - `durable-guard-snapshot-store:file-default`
    - `channel-snapshot-store:file-default`
    - `message-lifecycle-snapshot-store:file-default`
    - `runtime-snapshot-store:file-default`
  - Bootstrap startup now validates prioritized persisted store compatibility and fails closed with typed config errors:
    - `ConfigError::RuntimeStoreCorruptPayload`
    - `ConfigError::RuntimeStoreSchemaIncompatible`
    - `ConfigError::RuntimeStoreCompatibility`
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `restart_recovery_status=verified`, `corruption_fail_closed_status=verified`, `incompatible_schema_fail_closed_status=verified`, `execution_scope=local-scheduled`, `performance_budget_status=verified`.
  - Fail-closed reason-code matrix validated: `content_storage_corrupt_payload_rejected`, `did_registry_corrupt_payload_rejected`, `task_operation_snapshot_schema_mismatch_rejected`, `durable_guard_snapshot_schema_mismatch_rejected`, `channel_snapshot_corrupt_payload_rejected`, `channel_snapshot_schema_mismatch_rejected`, `message_lifecycle_snapshot_corrupt_payload_rejected`, `message_lifecycle_snapshot_schema_mismatch_rejected`, `runtime_snapshot_corrupt_payload_rejected`, `runtime_snapshot_state_version_regression_rejected`.
- Post-roadmap hardening wave 1 initial slice delivered:
  - Added deterministic `execution_id` structured logging correlation field for runtime dispatch/start/complete lifecycle events in `kamn-node` (Task #3032, Subtask #3033).
  - Added regression assertions in `crates/kamn-node/src/main_tests/core_behavior_tests.rs` to fail closed when runtime structured events omit `execution_id`.
  - Updated observability documentation contracts in `docs/foundation/observability-slo-dashboards.md`.
- Post-roadmap hardening wave 1 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_structured_logging_live.sh` and `scripts/runtime/test_validate_structured_logging_live.sh` (Task #3035, Subtask #3036).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `structured_logging_contract_status=verified`, `correlation_contract_status=verified`, `docs_contract_status=verified`, `fail_closed_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for invalid log config drill: `fail_closed_reason_code=invalid_log_config_level`.
- Post-roadmap hardening wave 1 nonce-retry live validation delivered:
  - Runtime lane: `scripts/runtime/validate_nonce_retry_live.sh` and `scripts/runtime/test_validate_nonce_retry_live.sh` (Task #3042, Subtask #3043).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `nonce_retry_contract_status=verified`, `nonce_malformed_fail_closed_status=verified`, `docs_contract_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for malformed nonce response guard behavior: `fail_closed_reason_code=nonce_response_malformed`.
- Post-roadmap hardening wave 2 runtime observability endpoint live validation delivered:
  - Runtime lane: `scripts/runtime/validate_runtime_observability_endpoint_live.sh` and `scripts/runtime/test_validate_runtime_observability_endpoint_live.sh` (Task #3047, Subtask #3048).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `runtime_observability_stream_contract_status=verified`, `fail_closed_status=verified`, `docs_contract_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for unknown-path guard behavior: `fail_closed_reason_code=observability_endpoint_not_found`.
- Post-roadmap hardening wave 6 runtime observability endpoint contract-lane policy delivered:
  - Runtime lane: `scripts/runtime/validate_runtime_observability_endpoint_live_contract_lane.sh` and `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh` (Task #3150, Subtask #3160).
  - Policy checker: `scripts/runtime/check_runtime_observability_endpoint_live_policy.sh` and `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`.
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `runtime_observability_stream_contract_status=verified`, `runtime_observability_policy_status=verified`, `runtime_observability_contract_lane_status=verified`, `docs_contract_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for policy tamper drill behavior: `fail_closed_reason_code=runtime_observability_policy_final_decision_mismatch`.
- Post-roadmap hardening wave 6 shell-surface trend-guard tranche-3 delivered:
  - CI checker now fails closed on undocumented current-script growth for non-Kolme wave trend-test LOC soft-budget governance (Task #3152, Subtask #3162):
    - `scripts/ci/check_non_kolme_wave_trend_test_loc_soft_budget.py`
    - `scripts/ci/test_check_non_kolme_wave_trend_test_loc_soft_budget.sh`
  - Deterministic reason-code surface expanded with: `unexpected_current_scripts`.
  - Trend-policy interpretation guidance updated in `docs/ci/strategy.md` to require baseline refresh for any new script path.
- Post-roadmap hardening wave 6 fast-gate runtime/cost trend contract lane delivered:
  - Added manifest-backed contract lane wrapper + shared implementation for deterministic fast-gate budget-delta path coverage (Task #3154, Subtask #3164):
    - `scripts/ci/run_fast_gate_budget_delta_contract_lane.sh`
    - `scripts/ci/fast_gate_budget_delta_contract_lane_impl.sh`
    - `scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json`
    - `scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh`
  - Fast-gate delta threshold config now validates freshness metadata and fails closed on stale/corrupt threshold files:
    - required keys in `.ci/fast-gate-budget-delta.env`: `FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON`, `FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS`
    - stale/corrupt guard reason markers: `fast_gate_delta_threshold_file_stale`, `fast_gate_delta_threshold_file_corrupt`
  - CI strategy/cost docs and contract markers updated with remediation guidance:
    - refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata.
- Post-roadmap hardening wave 2 runtime decomposition initial tranche delivered:
  - Extracted state-divergence orchestration from `crates/kamn-core/src/runtime.rs` into dedicated module `crates/kamn-core/src/runtime_state_divergence.rs` with unchanged external behavior (Task #3050, Subtask #3051).
  - Module-ownership contract documented in `docs/foundation/runtime-watchdog-attestation.md` and enforced by `crates/kamn-core/tests/runtime_watchdog_attestation_docs.rs`.
- Post-roadmap hardening wave 2 runtime decomposition tranche 2 delivered:
  - Extracted daemon phase-coordination orchestration from `crates/kamn-core/src/runtime.rs` into dedicated module `crates/kamn-core/src/runtime_phase_coordination.rs` with unchanged external behavior (Task #3050, Subtask #3057).
  - Module-ownership contract documented in `docs/foundation/runtime-network.md` and enforced by `crates/kamn-core/tests/runtime_network_docs.rs` and `crates/kamn-core/tests/runtime_module_extraction_contract.rs`.
- Post-roadmap hardening wave 2 runtime decomposition tranche 3 delivered:
  - Extracted watchdog/transport-coordination simulation orchestration from `crates/kamn-core/src/runtime.rs` into dedicated module `crates/kamn-core/src/runtime_transport_coordination.rs` with unchanged external behavior (Task #3050, Subtask #3059).
  - Module-ownership contract documented in `docs/foundation/runtime-network.md` and enforced by `crates/kamn-core/tests/runtime_network_docs.rs` and `crates/kamn-core/tests/runtime_module_extraction_contract.rs`.
- Post-roadmap hardening wave 4 runtime decomposition tranche 4 delivered:
  - Extracted runtime snapshot persistence/restore/store orchestration from `crates/kamn-core/src/runtime.rs` into dedicated module `crates/kamn-core/src/runtime_snapshot_store.rs` with unchanged external behavior (Task #3090, Subtask #3091).
  - Module-ownership and docs-contract markers enforced by `crates/kamn-core/tests/runtime_module_extraction_contract.rs` and `crates/kamn-core/tests/runtime_network_docs.rs` (Task #3092, Subtask #3093).
- Post-roadmap hardening wave 4 runtime decomposition tranche 5 delivered:
  - Extracted recovery/rejoin guard orchestration from `crates/kamn-core/src/runtime.rs` into dedicated module `crates/kamn-core/src/runtime_recovery_guard.rs` with unchanged external behavior (Task #3129, Subtask #3130).
  - Module-ownership and docs-contract markers enforced by `crates/kamn-core/tests/runtime_module_extraction_contract.rs` and `crates/kamn-core/tests/runtime_network_docs.rs`.
- Post-roadmap hardening wave 5 runtime decomposition tranche 6 delivered:
  - Extracted peer lifecycle/queue/frame/proposal/runtime-wiring orchestration from `crates/kamn-core/src/runtime.rs` into dedicated module `crates/kamn-core/src/runtime_peer_coordination.rs` with unchanged external behavior (Task #3145, Subtask #3155).
  - Module-ownership and docs-contract markers enforced by `crates/kamn-core/tests/runtime_module_extraction_contract.rs` and `crates/kamn-core/tests/runtime_network_docs.rs`.
- Post-roadmap hardening wave 3 managed-signer startup live validation delivered:
  - Contract lane: `scripts/kolme/run_managed_signer_startup_live_validation_contract_lane.sh` and `scripts/kolme/test_run_managed_signer_startup_live_validation_contract_lane.sh` (Subtask #3067).
  - Contract report schema: `kamn.kolme.managed-signer-startup-live-validation-contract-report.v1`.
  - Baseline startup pass marker: `deployment_preflight_passed`.
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `managed_signer_profile_status=verified`, `managed_signer_reason_code_status=verified`, `execution_scope=local-scheduled`, `performance_budget_status=verified`.
  - Signer key-source profile matrix validation delivered (Task #3108, Subtask #3109):
    - matrix markers: `signer_key_source_profile_matrix_status=verified`, `signer_key_source_production_reject_status=verified`, `signer_key_source_local_override_allow_status=verified`, `signer_fallback_private_key_reject_status=verified`.
    - production strict env-local fail-closed reason code: `production_signer_key_source_env_local_forbidden`.
    - fallback private-key env fail-closed reason code: `fallback_signer_secret_present_violation`.
    - explicit local override marker: `KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true`.
  - Fail-closed validation confirmed for startup fault-injection matrix:
    - missing managed-external key-source contract: `checkpoint_failed_signer_provenance_contract` + `signer_key_source_production_managed_external_required`
    - invalid signer profile: `checkpoint_failed_signer_profile_contract` + `signer_profile_mismatch`
    - stale signer rotation metadata: `checkpoint_failed_signer_rotation_freshness_contract` + `signer_rotation_epoch_stale`
- Post-roadmap hardening wave 4 local-live managed-signer runtime+finality validation delivered (Story #3088):
  - Local-live composition lane delivered and validated (Task #3102, Task #3104):
    - `scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh`
    - `scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh`
    - `scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
    - `scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh`
    - `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
  - Runtime submit/finality checks are composed through:
    - `scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh`
    - `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py`
  - Local-only heavy execution gate remains enforced for run mode via `KAMN_KOLME_LOCAL_HEAVY=1`.
  - Deterministic marker/schema contracts validated:
    - `schema_version=kamn.kolme.local-kamn-live-runtime-integration-summary.v1`
    - `runtime_profile=real-node`
    - `runtime_commit_command_profile=real-node-non-synthetic-v1`
    - `runtime_commit_policy_command_profile=real-node-non-synthetic-v1`
  - Fail-closed reason-code matrix validated (Subtask #3103, Subtask #3105):
    - `runtime_commit_command_profile_mismatch`
    - `runtime_signer_failover_profile_unchanged`
    - `runtime_signer_rotation_epoch_stale`
    - `runtime_signer_key_source_profile_pair_disallowed`
    - `runtime_commit_live_policy_report_missing`
  - Runtime endpoint failure taxonomy remains deterministic for preflight/connectivity classes:
    - `transport.preflight.timeout`
    - `transport.preflight.failed`
- Post-roadmap hardening wave 4 wrapper migration tranche A delivered:
  - Migrated contract wrappers to dispatcher symlink + manifest execution path (Task #3094, Subtask #3095):
    - `scripts/kolme/run_continuous_runtime_commit_contract_lane.sh`
    - `scripts/kolme/run_did_lifecycle_chain_adapter_contract_lane.sh`
    - `scripts/kolme/run_message_proof_anchoring_contract_lane.sh`
  - Added manifest contracts:
    - `scripts/framework/manifests/kolme_continuous_runtime_commit_contract_lane.json`
    - `scripts/framework/manifests/kolme_did_lifecycle_chain_adapter_contract_lane.json`
    - `scripts/framework/manifests/kolme_message_proof_anchoring_contract_lane.json`
  - Added Python contract-lane implementations:
    - `scripts/kolme/contracts/continuous_runtime_commit_contract_lane.py`
    - `scripts/kolme/contracts/did_lifecycle_chain_adapter_contract_lane.py`
    - `scripts/kolme/contracts/message_proof_anchoring_contract_lane.py`
  - Dispatcher lookup and parity harness expanded:
    - `scripts/kolme/run_contract_lane_dispatch.sh`
    - `scripts/kolme/test_contract_lane_dispatch_wrapper_matrix.sh`
    - `scripts/kolme/test_run_continuous_runtime_commit_contract_lane.sh`
    - `scripts/kolme/test_run_did_lifecycle_chain_adapter_contract_lane.sh`
    - `scripts/kolme/test_run_message_proof_anchoring_contract_lane.sh`
- Post-roadmap hardening wave 4 wrapper governance tranche B delivered (Story #3119):
  - Added generic Kolme wave wrapper-family contract harnesses to consolidate baseline/trend policy checks:
    - `scripts/ci/test_kolme_wave_wrapper_family_baseline_contract_impl.sh`
    - `scripts/ci/test_check_kolme_wave_wrapper_family_budget_trend_impl.sh`
  - Migrated wave-specific Kolme wrapper-family tests to thin wrappers over generic impl:
    - `scripts/ci/test_kolme_wave8_wrapper_family_baseline_contract.sh`
    - `scripts/ci/test_check_kolme_wave8_wrapper_family_budget_trend.sh`
    - `scripts/ci/test_kolme_wave10_wrapper_family_baseline_contract.sh`
    - `scripts/ci/test_check_kolme_wave10_wrapper_family_budget_trend.sh`
    - `scripts/ci/test_kolme_wave11_wrapper_family_baseline_contract.sh`
    - `scripts/ci/test_check_kolme_wave11_wrapper_family_budget_trend.sh`
  - Added deterministic wave-11 wrapper-family governance fixtures/checker (Task #3120, Subtask #3121):
    - `fixtures/ci/kolme_wave11_wrapper_family_matrix.json`
    - `fixtures/ci/kolme_wave11_wrapper_family_baseline.json`
    - `fixtures/ci/kolme_wave11_wrapper_family_trend_thresholds.json`
    - `scripts/ci/check_kolme_wave11_wrapper_family_budget_trend.sh`
  - CI/docs parity and selector scope contracts expanded for Kolme wave-10/11 wrapper-family governance:
    - `scripts/ci/test_ci_tools.sh`
    - `scripts/ci/test_ci_tools_command_surface_contract.sh`
    - `scripts/ci/test_ci_strategy_contract.sh`
    - `scripts/ci/select_targets.sh`
    - `docs/ci/strategy.md`
    - `crates/kamn-core/tests/ci_strategy_docs.rs`
- Phase 2.1 initial slice delivered: deterministic `runtime-mode api` ingress server with required messaging/channel/task/profile/health/metrics route contracts (Task #2906).
- Phase 2.1 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_service_api_live.sh` and `scripts/runtime/test_validate_service_api_live.sh` (Task #2908, Subtask #2909).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `route_contract_status=verified`, `failure_case_status=verified`.
  - Fail-closed validation confirmed for invalid runtime budget argument: `max-seconds must be an integer`.
- Phase 2.2 initial slice delivered: request signature auth and per-sender nonce replay middleware enforcement on service API ingress (Task #2911, Subtask #2912).
- Phase 2.2 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_service_api_request_auth_live.sh` and `scripts/runtime/test_validate_service_api_request_auth_live.sh` (Task #2913, Subtask #2914).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `unauthorized_guard_status=verified`, `replay_guard_status=verified`, `probe_status=verified`.
  - Fail-closed validation confirmed for invalid runtime budget argument: `max-seconds must be an integer`.
- Phase 2.3 initial slice delivered: realtime websocket event route with deterministic upgrade/frame contract and fail-closed upgrade/auth/replay guards (Task #2916, Subtask #2917).
- Phase 2.3 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_service_api_websocket_live.sh` and `scripts/runtime/test_validate_service_api_websocket_live.sh` (Task #2918, Subtask #2919).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `websocket_upgrade_status=verified`, `fail_closed_status=verified`, `probe_status=verified`.
  - Fail-closed validation confirmed for invalid runtime budget argument: `max-seconds must be an integer`.
- Runtime decomposition wave 1 delivered:
  - Extracted network-fault simulation APIs from `crates/kamn-core/src/runtime_transport_coordination.rs` into `crates/kamn-core/src/runtime_network_fault.rs` with stable `runtime.rs` re-export compatibility (Task #3186, Subtask #3187).
  - Added extraction regression contract coverage for module declaration and re-export continuity (`runtime::tests::regression_runtime_source_routes_network_fault_domain_via_dedicated_module`).
- Runtime decomposition wave 2 delivered:
  - Moved inline runtime tests from `crates/kamn-core/src/runtime.rs` into `crates/kamn-core/src/runtime_tests.rs`, leaving `runtime.rs` focused on runtime module wiring/re-exports (Task #3191, Subtask #3192).
  - Added extraction-routing regression coverage for path-based runtime test module loading (`runtime::tests::regression_runtime_source_routes_tests_via_dedicated_module_file`).
- Runtime decomposition wave 3 delivered:
  - Extracted runtime snapshot-store test surface from `crates/kamn-core/src/runtime_tests.rs` into `crates/kamn-core/src/runtime_tests_snapshot_store.rs` with path-based routing through `runtime_tests.rs` (Task #3206, Subtask #3207).
  - Added extraction-routing regression coverage to fail closed if snapshot-store tests drift back inline (`runtime::tests::regression_runtime_tests_source_routes_snapshot_store_domain_via_dedicated_module_file`).
- Runtime decomposition wave 4 delivered:
  - Extracted runtime network-fault simulation tests from `crates/kamn-core/src/runtime_tests.rs` into `crates/kamn-core/src/runtime_tests_network_fault.rs` with path-based routing through `runtime_tests.rs` (Task #3211, Subtask #3212).
  - Added extraction-routing regression coverage to fail closed if network-fault tests drift back inline (`runtime::tests::regression_runtime_tests_source_routes_network_fault_domain_via_dedicated_module_file`).
- Phase 3.1 initial slice delivered:
  - Added deterministic p2p transport contracts in `kamn-core`: `PeerLifecycleTransport`, `InMemoryPeerLifecycleTransport`, `PeerDiscoveryRecord`, `PeerGossipFrame`, and `PeerLifecycleTransportCoordinator` (Task #2921, Subtask #2922).
  - Added bootstrap/runtime wiring integration so `enable_gossip` toggles explicit `p2p-discovery`/`p2p-gossip-transport` components (or `gossip-transport-disabled` when off).
  - Added unit/functional/integration/regression coverage in `crates/kamn-core/src/p2p_transport.rs` and `crates/kamn-core/tests/p2p_transport_runtime.rs`.
  - Architecture documentation added at `docs/architecture/p2p-transport.md`.
- Phase 3.1 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_p2p_transport_live.sh` and `scripts/runtime/test_validate_p2p_transport_live.sh` (Task #2923, Subtask #2924).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `p2p_transport_contract_status=verified`, `docs_contract_status=verified`, `fail_closed_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for disconnected broadcast guard behavior: `fail_closed_reason_code=p2p_transport_inactive_lifecycle_state`.
- Phase 3.2 initial slice delivered:
  - Added `MempoolBlockPipeline` with explicit listener quorum, approver quorum, and commit orchestration over pending mempool transactions (Task #2926, Subtask #2927).
  - Added deterministic fail-closed digest mismatch and empty-mempool guards before commit.
  - Added processor runtime wiring component projection for `consensus-validator`.
  - Added unit/functional/integration/regression coverage in `crates/kamn-core/src/block_pipeline.rs` and `crates/kamn-core/tests/block_pipeline.rs`.
  - Architecture documentation added at `docs/architecture/block-pipeline.md`.
- Phase 3.2 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_block_pipeline_live.sh` and `scripts/runtime/test_validate_block_pipeline_live.sh` (Task #2928, Subtask #2929).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `block_pipeline_contract_status=verified`, `docs_contract_status=verified`, `fail_closed_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for approver payload digest mismatch guard behavior: `fail_closed_reason_code=block_pipeline_payload_digest_mismatch`.
- Phase 4.1 initial slice delivered: `runtime-mode kolme-live` now supports bounded continuous commit/finality execution when paired cycle controls are supplied (`--daemon-max-ticks` and `--daemon-tick-interval-ms`) with fail-closed guardrails for partial declarations (Task #2931, Subtask #2932).
- Phase 4.1 live validation delivered:
  - Runtime lane: `scripts/kolme/validate_continuous_runtime_commit_live.sh` and `scripts/kolme/test_validate_continuous_runtime_commit_live.sh` (Task #2933, Subtask #2934).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `continuous_runtime_contract_status=verified`, `evidence_bundle_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for partial cycle-control declarations: `--daemon-tick-interval-ms`.
- Phase 4.2 initial slice delivered:
  - Added lifecycle chain-submission contracts in `DidRegistry`: deterministic lifecycle idempotency keys, lifecycle retry classification, and nonce-scoped lifecycle finality recording (Task #2936, Subtask #2937).
  - Added `KolmeDidLifecycleChainAdapter` to project lifecycle mutations into deterministic runtime-commit submissions through `KolmeRuntimeCommitClient`.
  - Added unit/functional/integration/regression coverage in `crates/kamn-core/tests/did_registry_transactions.rs` for lifecycle submission through Kolme-backed adapters.
- Phase 4.2 live validation delivered:
  - Runtime lane: `scripts/kolme/validate_did_lifecycle_chain_adapter_live.sh` and `scripts/kolme/test_validate_did_lifecycle_chain_adapter_live.sh` (Task #2938, Subtask #2939).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `did_lifecycle_contract_status=verified`, `evidence_bundle_status=verified`, `docs_contract_status=verified`, `fail_closed_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for conflicting same DID+nonce lifecycle payloads: `fail_closed_reason_code=did_registry_submission_key_conflict`.
- Phase 4.3 initial slice delivered:
  - Added `MessageProofAnchoringService` with lifecycle-aligned anchor submission (`Broadcast|Included`), deterministic idempotency/retry classification, and anchor finality tracking (Task #2941, Subtask #2942).
  - Added `MessageProofChainAdapter` surfaces with `InMemoryMessageProofChainAdapter` and `KolmeMessageProofChainAdapter`.
  - Added unit/functional/integration/regression/performance coverage in `crates/kamn-core/tests/message_proof_anchoring.rs`.
- Phase 4.3 live validation delivered:
  - Runtime lane: `scripts/kolme/validate_message_proof_anchoring_live.sh` and `scripts/kolme/test_validate_message_proof_anchoring_live.sh` (Task #2943, Subtask #2944).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `message_anchor_contract_status=verified`, `evidence_bundle_status=verified`, `docs_contract_status=verified`, `fail_closed_status=verified`, `performance_budget_status=verified`.
  - Fail-closed validation confirmed for conflicting idempotency submissions: `fail_closed_reason_code=message_proof_anchor_conflicting_key`.
- Phase 6.1 initial slice delivered: service API `/metrics` now exports deterministic runtime telemetry gauges and source/health labels with fail-closed unknown defaults when daemon/kolme telemetry is unavailable (Task #2961, Subtask #2962).
- Phase 6.1 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_service_api_observability_live.sh` and `scripts/runtime/test_validate_service_api_observability_live.sh` (Task #2963, Subtask #2964).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `metrics_contract_status=verified`, `health_contract_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for invalid runtime budget argument: `max-seconds must be an integer`.
- Phase 6.2 initial slice delivered:
  - Added deterministic config layering in `kamn-node` with `--config-file` and `KAMN_NODE_CONFIG_FILE`.
  - Added validated `KAMN_NODE_*` override projection over config-file values with precedence `config < env < CLI`.
  - Extended layering so `KAMN_NODE_*` overrides apply even without a config file and added daemon env contracts for `KAMN_NODE_DAEMON_MAX_TICKS` and `KAMN_NODE_DAEMON_TICK_INTERVAL_MS` (Story #3200, Task #3201, Subtask #3202).
  - Added fail-closed config/override validation with typed `ConfigError` surfaces and regression coverage for invalid env override values (Story #2965, Task #2966, Subtask #2967).
  - Operator contracts documented in `docs/ops/configuration.md`.
- Phase 6.2 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_config_layering_live.sh` and `scripts/runtime/test_validate_config_layering_live.sh` (Task #2968).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `layering_contract_status=verified`, `precedence_contract_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for invalid override injection: `invalid sync mode: turbo`.
- Phase 5 parity-matrix implementation delivered:
  - Added unified SDK parity orchestration runner: `scripts/sdk/run_cross_language_sdk_parity_matrix.sh`.
  - Runner composes register validation parity and live transport contract parity with bounded runtime and deterministic machine-readable markers.
  - Added deterministic harness: `scripts/sdk/test_run_cross_language_sdk_parity_matrix.sh` (Task #2956, Subtask #2957).
  - Added operator/reference documentation: `docs/sdk/parity-matrix.md`.
- Phase 5 parity-matrix live validation delivered:
  - Runtime lane: `scripts/sdk/validate_cross_language_sdk_parity_matrix_live.sh` and `scripts/sdk/test_validate_cross_language_sdk_parity_matrix_live.sh` (Task #2958, Subtask #2959).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `matrix_contract_status=verified`, `evidence_bundle_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for invalid mode drill: `mode must be one of: contract,deep`.
- Phase 5 Python SDK packaging implementation delivered:
  - Added publishable packaging metadata at repository root: `pyproject.toml`.
  - Added deterministic packaging contract runner/harness: `scripts/sdk/run_python_sdk_packaging_contract.sh` and `scripts/sdk/test_run_python_sdk_packaging_contract.sh` (Task #2951, Subtask #2952).
  - Added operator/developer packaging contract documentation: `docs/sdk/python-sdk.md`.
- Phase 5 Python SDK packaging live validation delivered:
  - Runtime lane: `scripts/sdk/validate_python_sdk_packaging_live.sh` and `scripts/sdk/test_validate_python_sdk_packaging_live.sh` (Task #2953, Subtask #2954).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `packaging_contract_status=verified`, `evidence_bundle_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for missing metadata drill: `expected python sdk packaging metadata file: pyproject.toml`.
- Phase 5 Rust SDK service-client implementation delivered:
  - Added synchronous service API client primitives in `kamn-sdk`: `ServiceApiClient`, `ServiceRequestAuth`, `service_signature_for_fields`, and typed route/event response models (Task #2946).
  - Added deterministic unit/functional/integration/regression coverage in `crates/kamn-sdk/tests/service_api_client.rs`.
  - Added operator/developer reference documentation: `docs/sdk/rust-sdk.md`.
- Phase 5 Rust SDK service-client live validation delivered:
  - Runtime lane: `scripts/sdk/validate_rust_sdk_service_client_live.sh` and `scripts/sdk/test_validate_rust_sdk_service_client_live.sh` (Task #2948, Subtask #2949).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `service_client_contract_status=verified`, `evidence_bundle_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for invalid runtime budget drill: `max-seconds must be greater than zero`.
- Phase 6.3 initial slice delivered: deployment artifacts now include a multi-stage `Dockerfile`, `deploy/docker-compose.yml` role topology, and `deploy/k8s/kamn-node.yaml` baseline manifests (Task #2971, Subtask #2972).
- Phase 6.3 live validation delivered:
  - Runtime lane: `scripts/deploy/validate_deployment_assets_live.sh` and `scripts/deploy/test_validate_deployment_assets_live.sh` (Task #2973, Subtask #2974).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `asset_contract_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for invalid runtime budget argument: `max-seconds must be an integer`.
- Phase 6.3 hardening slice delivered:
  - Added `.dockerignore` build-context hygiene markers and extended deployment asset contract checks to fail closed on missing exclusions (`.git`, `target`, `.tmp`) (Task #3196, Subtask #3197).
  - Added compose resiliency markers (`restart: unless-stopped`) for processor/listener/approver services and documented hardening expectations in deployment ops docs.
- Phase 6.4 implementation delivered:
  - Runtime lane: `scripts/runtime/run_live_validation_environment_lane.sh` and `scripts/runtime/test_run_live_validation_environment_lane.sh` (Task #2976, Subtask #2977).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `topology_contract_status=verified`, `kolme_connectivity_contract_status=verified`, `fail_closed_status=verified`.
  - Local-only run safety gate validated: run mode requires explicit opt-in via `KAMN_KOLME_LOCAL_HEAVY=1`.
- Phase 6.4 live validation delivered:
  - Runtime lane: `scripts/runtime/validate_live_validation_environment_live.sh` and `scripts/runtime/test_validate_live_validation_environment_live.sh` (Task #2978, Subtask #2979).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `lane_contract_status=verified`, `evidence_bundle_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for missing local-only opt-in in run mode: `run mode requires explicit local-only opt-in via KAMN_KOLME_LOCAL_HEAVY=1`.
- Failure-drills implementation delivered:
  - Runtime lane: `scripts/runtime/run_network_signer_finality_failure_drills_lane.sh` and `scripts/runtime/test_run_network_signer_finality_failure_drills_lane.sh` (Task #2981, Subtask #2982).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `network_partition_status=verified`, `signer_fault_status=verified`, `finality_fault_status=verified`.
  - Injected signer fault profile validated as fail-closed: `signer_fault_injection_triggered`.
- Failure-drills live validation delivered:
  - Runtime lane: `scripts/runtime/validate_failure_drills_live.sh` and `scripts/runtime/test_validate_failure_drills_live.sh` (Task #2983, Subtask #2984).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `baseline_contract_status=verified`, `fault_injection_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for signer fault injection: `signer_fault_injection_triggered`.
- Go/no-go gate implementation delivered:
  - Runtime lane: `scripts/runtime/run_go_no_go_gate_lane.sh` and `scripts/runtime/test_run_go_no_go_gate_lane.sh` (Task #2986, Subtask #2987).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `go_no_go_evidence_status=verified`, `rollback_readiness_status=verified`, `dr_readiness_status=verified`.
  - Injected decision-fault profile validated as fail-closed: `gate_decision_fault_injection_triggered`.
- Go/no-go gate live validation delivered:
  - Runtime lane: `scripts/runtime/validate_go_no_go_gate_live.sh` and `scripts/runtime/test_validate_go_no_go_gate_live.sh` (Task #2988, Subtask #2989).
  - Deterministic GO markers validated: `status=pass`, `final_decision=GO`, `baseline_contract_status=verified`, `fault_injection_status=verified`, `fail_closed_status=verified`.
  - Fail-closed validation confirmed for decision fault injection: `gate_decision_fault_injection_triggered`.

---

## Phase 1: Make a Node That Stays Running (Foundation)

### 1.1 — Add an async runtime

This is the single biggest blocker. The entire codebase is synchronous. You need tokio.

- Add `tokio` to `kamn-node/Cargo.toml` with `rt-multi-thread`, `macros`, `net`, `time`, `signal` features
- Convert `fn main()` to `#[tokio::main] async fn main()`
- The existing synchronous domain logic in `kamn-core` does NOT need to change — it's pure computation. Wrap it at the node boundary
- Estimated scope: `kamn-node` only. `kamn-core` stays sync

### 1.2 — Real OS signal handling

`daemon_shutdown.rs` currently simulates signals via CLI tick arguments. Replace with actual signal handling:

- Add `tokio::signal::unix::signal(SignalKind::terminate())` and `ctrl_c()`
- Wire into the existing `evaluate_daemon_completion` logic
- This is ~50 lines of real code wrapping what already exists

### 1.3 — Persistent storage

Every store is in-memory. You need at minimum one real backend. Recommended: **redb** (embedded, zero-config, Rust-native) or **SQLite via rusqlite**.

Stores that need real backends (priority order):

| Store | What it holds | Current impl |
|---|---|---|
| `ReputationStore` | Agent trust scores, endorsements, disputes | `BTreeMap` |
| `ChannelSnapshotStore` | Channel metadata, membership | `InMemoryChannelSnapshotStore` |
| `MessageLifecycleSnapshotStore` | Message status, delivery tracking | `InMemoryMessageLifecycleSnapshotStore` |
| `TaskOperationSnapshotStore` | Task state, dependency graph | `InMemoryTaskOperationSnapshotStore` |
| `DurableGuardSnapshotStore` | Delivery guard nonces, replay protection | `InMemoryDurableGuardSnapshotStore` |
| `ContentStorageAdapter` | Content blobs, CIDs | `InMemoryContentAdapter` |
| `DidRegistry` | Agent DID documents | `InMemoryDidRegistrationChainAdapter` |

The trait interfaces already exist (`ContentStorageAdapter`, `ChannelSnapshotStore`, etc.). You need to implement `SqliteChannelSnapshotStore`, `SqliteMessageLifecycleSnapshotStore`, etc. The `File*` variants exist for some stores but they're single-file JSON dumps — not suitable for concurrent access.

Status update: file-backed adapters now also include content storage and DID chain submission persistence for deterministic restart-safe behavior. Database-backed adapters remain planned follow-up work.

Estimated scope: ~1,500–2,500 lines. One generic key-value store adapter would cover most of these since they all follow put/get/list patterns.

---

## Phase 2: Accept Incoming Connections (Server)

### 2.1 — HTTP/JSON-RPC API server

The node needs to listen for requests. Recommended: **axum** (async, tower-based, lightweight).

Minimum endpoints:

```
POST /v1/messages/send          — Submit a signed message envelope
GET  /v1/messages/{id}          — Get message status
POST /v1/channels/create        — Create a channel
GET  /v1/channels/{id}/messages — List messages in a channel
POST /v1/tasks/create           — Create a task
GET  /v1/tasks/{id}             — Get task state
GET  /v1/agents/{did}           — Get agent reputation/profile
GET  /healthz                   — Health check
GET  /metrics                   — Prometheus metrics
```

All the request validation, envelope parsing, state machine transitions, and response construction logic **already exists** in `kamn-core`. The API layer is mostly wiring — deserialize request, call domain function, serialize response.

Estimated scope: ~2,000–3,000 lines for the server skeleton + route handlers.

### 2.2 — Authentication middleware

The cryptographic primitives exist (`k256` ECDSA signing, DID verification). You need:

- Request signature verification middleware (extract signature from header, verify against sender DID)
- The `EnvelopeProof` validation in `message_envelope.rs` already defines the schema
- Nonce/replay checking via `MessageDeliveryGuards` already works

Estimated scope: ~300–500 lines of axum middleware wrapping existing logic.

### 2.3 — WebSocket server for real-time events

Agents need push notifications (message received, task state changed, payment confirmed). Add WebSocket upgrade support to the axum server:

- Use `axum::extract::ws::WebSocket`
- The `KolmeRuntimeCommitNotificationsConsumer` already defines the event model
- Broadcast events when domain state transitions occur

Estimated scope: ~500–800 lines.

---

## Phase 3: Node-to-Node Communication (P2P)

### 3.1 — libp2p gossip layer

The PRD specifies a triadic network (Processor/Listener/Approver) with gossip. The `PeerLifecycle` state machine exists but has no real transport.

- Add `libp2p` with `gossipsub`, `identify`, `kad` (Kademlia DHT for peer discovery), `noise` (encryption), `yamux` (multiplexing)
- Wire `PeerLifecycleEvent` transitions to actual connection events
- Gossip topics: `messages`, `blocks`, `reputation-updates`
- The `RoleSmokeNetwork` (processor/listener/approver) is the skeleton — it needs real networking underneath

This is the largest single piece of work. Estimated scope: ~4,000–6,000 lines.

### 3.2 — Block production and consensus

The `ProducedBlock`, `BaselineTransaction`, and `TransactionGuards` types exist. The processor role needs to:

- Collect transactions from gossip into a real mempool (currently `Vec<BaselineTransaction>` in memory)
- Produce blocks on demand (the `RoleSmokeNetwork.produce_block()` logic exists)
- Broadcast blocks to listeners/approvers
- Validate incoming blocks against `TransactionGuards`
- Persist committed blocks

This connects to Kolme's block production model. Estimated scope: ~2,000–3,000 lines.

---

## Phase 4: Kolme Blockchain Integration (On-Chain)

### 4.1 — Runtime commit pipeline (partially exists)

The `KolmeRuntimeCommitLiveProvider` HTTP client works. What's missing:

- Automatic transaction submission (currently requires manual CLI invocation)
- Finality polling loop (the `KolmeRuntimeCommitFinalityChecker` exists but must be driven by async task)
- WebSocket reconnection (budget tracking exists, reconnect logic doesn't)
- Block fallback reconciliation on reorgs

Estimated scope: ~1,000–1,500 lines to make the existing client code run continuously.

### 4.2 — DID on-chain registration

`DidRegistry` and `DidRegistrationChainAdapter` traits exist with full lifecycle (create, update, deactivate). The `InMemoryDidRegistrationChainAdapter` needs to be replaced with one that submits DID operations as Kolme transactions.

Estimated scope: ~500–800 lines.

### 4.3 — On-chain message anchoring

Messages are currently processed entirely in-memory. For auditability:

- Hash message envelopes and anchor hashes on-chain via Kolme transactions
- The `MessageLifecycleStore` status transitions (Created -> Signed -> Broadcast -> Included) map directly to on-chain states
- Content stays off-chain; only proof/hash goes on-chain

Estimated scope: ~800–1,200 lines.

---

## Phase 5: SDK and Client Libraries

### 5.1 — Rust SDK (partially exists)

`kamn-sdk` is a scaffold with TCP transport examples. Needs:

- HTTP client wrapping the API from Phase 2
- WebSocket subscription client
- High-level `KamnClient` with `send_message()`, `create_task()`, `check_reputation()`
- Connection management, retry, auth

Estimated scope: ~1,500–2,000 lines.

### 5.2 — Python SDK (partially exists)

`tests/python/test_sdk.py` and the `LiveKAMNClient` class exist with a backend adapter pattern. Needs:

- Real HTTP transport (currently uses mock adapter)
- WebSocket support for events
- PyPI-publishable package

Estimated scope: ~1,000–1,500 lines.

---

## Phase 6: Operational Readiness

### 6.1 — Observability endpoints

`daemon_observability.rs` and `kolme_live_observability.rs` synthesize SLO metrics but only write to stdout. Need:

- Prometheus `/metrics` endpoint (add `prometheus` or `metrics` crate)
- Export the existing metrics (latency_p50, throughput_tps, error_rate_bps, availability_bps) as Prometheus gauges/histograms
- Structured JSON logging already exists (`logging.rs`) — wire it to the async runtime

Estimated scope: ~500–800 lines.

### 6.2 — Configuration management

Currently everything is CLI flags (30+ of them in `cli.rs`). For production:

- TOML/YAML config file support (add `toml` or `serde_yaml`)
- Environment variable overrides (partially exists for signer keys)
- Config validation already exists in `NodeConfig.validate()`

Estimated scope: ~300–500 lines.

### 6.3 — Container and deployment

- Dockerfile (multi-stage Rust build)
- Docker Compose for local multi-node setup (processor + listener + approver)
- Kubernetes manifests or Helm chart
- The `upgrade-rollback-runbook.md` exists but needs real deployment tooling

Estimated scope: ~500 lines of infra config.

### 6.4 — Live validation environment

- Stand up deterministic live-environment lane for multi-process topology checks.
- Compose the Kolme local live-node validation bundle as the connectivity contract path.
- Keep CI cost bounded by defaulting to `dry-run` and requiring explicit local-only opt-in for `run`.
- Publish machine-readable evidence (`status`, `final_decision`, reason markers) for go/no-go consumption.

Estimated scope: ~300–600 lines across lane scripts, manifest wiring, and validation docs.

### 6.5 — Failure drills

- Execute deterministic failure drills for network partition/reconnect, signer incident recovery, and finality evidence contracts.
- Provide injected fault profile(s) that fail closed with reason-coded outputs.
- Keep runtime cost bounded by composing existing fast local lanes and budget guards.

Estimated scope: ~300–700 lines across runtime lane composition, regression harnesses, and operator docs.

### 6.6 — Go/No-Go Gate

- Enforce a deterministic release gate with evidence bundle checks and rollback readiness contracts.
- Require explicit NO-GO fail-closed behavior when decision evidence drifts from policy.
- Keep operational cost low by composing existing deploy contract lanes and bounded budgets.

Estimated scope: ~250–600 lines across runtime lane composition, regression harnesses, and operator docs.

---

## Priority Order and Effort Estimate

| Phase | What | Depends On | Estimated Lines | Criticality |
|---|---|---|---|---|
| **1.1** | Tokio async runtime | Nothing | ~200 | BLOCKER |
| **1.2** | Real signal handling | 1.1 | ~50 | HIGH |
| **1.3** | Persistent storage | Nothing | ~2,000 | BLOCKER |
| **2.1** | HTTP API server | 1.1, 1.3 | ~2,500 | BLOCKER |
| **2.2** | Auth middleware | 2.1 | ~400 | HIGH |
| **2.3** | WebSocket server | 2.1 | ~600 | MEDIUM |
| **3.1** | libp2p P2P layer | 1.1 | ~5,000 | HIGH |
| **3.2** | Block production | 3.1, 1.3 | ~2,500 | HIGH |
| **4.1** | Kolme commit pipeline | 1.1 | ~1,200 | MEDIUM |
| **4.2** | DID on-chain | 4.1 | ~600 | MEDIUM |
| **4.3** | Message anchoring | 4.1, 1.3 | ~1,000 | MEDIUM |
| **5.1** | Rust SDK | 2.1 | ~1,500 | MEDIUM |
| **5.2** | Python SDK | 2.1 | ~1,200 | LOW |
| **6.1** | Prometheus metrics | 2.1 | ~600 | HIGH |
| **6.2** | Config file support | Nothing | ~400 | MEDIUM |
| **6.3** | Container/deploy | 2.1 | ~500 | MEDIUM |

**Total estimated new code: ~20,000–25,000 lines of Rust** on top of the existing 88K.

---

## What You Can Skip (YAGNI)

Given the current state, these PRD features should be deferred past MVP:

- **ZK message proofs** — `zk_message_proofs.rs` has 1,000+ lines of evaluation logic but zero actual ZK circuits. Park it.
- **Cross-chain bridges** (Ethereum, Solana, Near) — Focus on Kolme only first
- **Telegram/Discord bridges** — Nice-to-have, not core
- **Service marketplace** — Requires a working economy first
- **Group channel encryption** — Start with direct messages
- **Content replication** — Start with single-node storage

---

## Minimum Viable Production Service

If you do only **Phases 1 + 2 + 6.1**, you get:

- A node that starts, stays running, handles signals
- Persistent state that survives restarts
- An HTTP API that agents can call to send messages, create tasks, check reputation
- Prometheus metrics for monitoring
- All backed by the 88K lines of domain logic and 2,000 tests that already exist

That's roughly **5,000–6,000 lines of new code** and would be a real, deployable service — single-node, no P2P, no blockchain anchoring, but functional. The P2P and Kolme integration (Phases 3 + 4) turn it into the decentralized network the PRD describes.
