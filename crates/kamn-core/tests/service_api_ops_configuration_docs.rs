const DOC: &str = include_str!("../../../docs/ops/configuration.md");

#[test]
fn service_api_ops_configuration_contains_signer_secret_zeroization_controls() {
    assert!(
        DOC.contains("## Signer Secret Decode Buffer Zeroization Controls (Issues #4165, #4166)")
    );
    assert!(DOC.contains("signer_secret_source_precedence_zeroization_status=verified"));
    assert!(DOC.contains("signer_private_key_parse_zeroization_status=verified"));
    assert!(DOC.contains("signer_transient_key_material_zeroization_status=verified"));
    assert!(DOC.contains("signer_secret_source_precedence_violation"));
    assert!(DOC.contains("managed_signer_private_key_adapter_unsupported"));
    assert!(DOC.contains(
        "signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer"
    ));
    assert!(DOC.contains(
        "signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material"
    ));
    assert!(DOC.contains("Regression: #4165"));
    assert!(DOC.contains("Regression: #4166"));
}

#[test]
fn service_api_ops_configuration_contains_async_backpressure_failure_modes() {
    assert!(DOC.contains("## Async API Backpressure Failure Modes (Issue #4315)"));
    assert!(DOC.contains(
        "service_api_backpressure_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("service_api_ingress_concurrency_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_rate_limit_exceeded"));
    assert!(DOC.contains("admission_inflight_budget_status=verified"));
    assert!(DOC.contains("admission_queue_budget_status=verified"));
    assert!(DOC.contains("admission_inflight_budget_limit=32"));
    assert!(DOC.contains("admission_queue_budget_limit=1"));
    assert!(DOC.contains(
        "admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch"
    ));
    assert!(DOC.contains("admission_decision_taxonomy_status=verified"));
    assert!(DOC.contains("admission_decision_accept_status=verified"));
    assert!(DOC.contains("admission_decision_defer_status=verified"));
    assert!(DOC.contains("admission_decision_reject_status=verified"));
    assert!(DOC.contains(
        "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject"
    ));
    assert!(DOC.contains("service_api_axum_policy_admission_inflight_budget_limit_mismatch"));
    assert!(DOC.contains("service_api_axum_policy_admission_queue_budget_limit_mismatch"));
    assert!(
        DOC.contains("service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch")
    );
    assert!(DOC.contains("service_api_axum_policy_admission_decision_reason_codes_csv_mismatch"));
    assert!(DOC.contains("fail-closed response contract"));
    assert!(DOC.contains("Regression: #4315"));
}

#[test]
fn service_api_ops_configuration_contains_tenant_isolation_matrix_markers() {
    assert!(DOC.contains("## Service API Tenant-Isolation Matrix Contract (Issue #4058)"));
    assert!(DOC.contains(
        "service_api_tenant_isolation_matrix_reason_taxonomy_version=kamn.runtime.service-api-tenant-isolation-matrix-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "service_api_tenant_isolation_matrix_reason_codes_csv=ci_fast_gate_failed,service_api_tenant_isolation_policy_schema_mismatch,service_api_tenant_isolation_policy_status_invalid,service_api_tenant_isolation_policy_final_decision_invalid,service_api_tenant_isolation_policy_final_decision_mismatch,service_api_tenant_isolation_policy_lane_mode_invalid,service_api_tenant_isolation_policy_matrix_schema_mismatch,service_api_tenant_isolation_policy_matrix_rows_invalid,service_api_tenant_isolation_policy_matrix_row_count_mismatch,service_api_tenant_isolation_policy_matrix_row_duplicate,service_api_tenant_isolation_policy_matrix_row_id_invalid,service_api_tenant_isolation_policy_matrix_row_missing,service_api_tenant_isolation_policy_matrix_row_status_mismatch,service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch,service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch,service_api_tenant_isolation_policy_matrix_row_selector_mismatch,service_api_tenant_isolation_policy_marker_missing,service_api_tenant_isolation_policy_execution_reason_code_mismatch,service_api_tenant_isolation_policy_command_count_invalid,service_api_tenant_isolation_policy_command_count_mismatch,service_api_tenant_isolation_policy_elapsed_seconds_invalid,service_api_tenant_isolation_policy_max_seconds_invalid,service_api_tenant_isolation_policy_runtime_budget_exceeded,service_api_tenant_isolation_policy_docs_marker_missing"
    ));
    assert!(DOC.contains(
        "service_api_tenant_isolation_matrix_matrix_schema_version=kamn.runtime.service-api-tenant-isolation-matrix.v1"
    ));
    assert!(DOC.contains(
        "service_api_tenant_isolation_matrix_required_row_ids_csv=m2_abac_cross_tenant_visibility_denied,m8_cross_owner_retention_and_shred_denied,m9_cross_owner_dispatch_and_presence_denied,m9_gateway_cross_owner_presence_denied"
    ));
    assert!(DOC.contains("m2_abac_scope_denied"));
    assert!(DOC.contains("m8_compliance_owner_scope_denied"));
    assert!(DOC.contains("m9_realtime_owner_scope_denied"));
    assert!(DOC.contains("service_api_tenant_isolation_policy_matrix_row_status_mismatch"));
    assert!(DOC.contains(
        "bash scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh --mode dry-run --output-json /tmp/service-api-tenant-isolation-matrix-live-summary.json"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/check_service_api_tenant_isolation_matrix_live_policy.sh --report-file /tmp/service-api-tenant-isolation-matrix-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-tenant-isolation-matrix-policy.json"
    ));
    assert!(DOC.contains("Regression: #4058"));
}

#[test]
fn service_api_ops_configuration_contains_api_version_policy_markers() {
    assert!(DOC.contains("## API Version-Policy Contract (Issue #4041)"));
    assert!(DOC.contains(
        "api_version_policy_reason_taxonomy_version=kamn.runtime.api-version-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "api_version_policy_reason_codes_csv=ci_fast_gate_failed,api_version_policy_schema_mismatch,api_version_policy_status_invalid,api_version_policy_final_decision_invalid,api_version_policy_final_decision_mismatch,api_version_policy_lane_mode_invalid,api_version_policy_fixture_schema_mismatch,api_version_policy_fixture_rows_invalid,api_version_policy_fixture_row_count_mismatch,api_version_policy_fixture_row_duplicate,api_version_policy_fixture_row_id_invalid,api_version_policy_fixture_row_missing,api_version_policy_fixture_row_status_mismatch,api_version_policy_fixture_row_decision_mismatch,api_version_policy_fixture_row_reason_code_mismatch,api_version_policy_fixture_row_version_mismatch,api_version_policy_fixture_row_window_mismatch,api_version_policy_marker_missing,api_version_policy_execution_reason_code_mismatch,api_version_policy_command_count_invalid,api_version_policy_command_count_mismatch,api_version_policy_elapsed_seconds_invalid,api_version_policy_max_seconds_invalid,api_version_policy_runtime_budget_exceeded,api_version_policy_docs_marker_missing"
    ));
    assert!(DOC.contains(
        "api_version_policy_fixture_schema_version=kamn.runtime.api-version-policy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "api_version_policy_fixture_path=fixtures/runtime/api_version_policy_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "api_version_policy_required_row_ids_csv=v1_messages_send,v2_channels_create,v0_messages_send,v3_future_route"
    ));
    assert!(DOC.contains("api_version_unsupported_window"));
    assert!(DOC.contains("api_version_policy_fixture_row_status_mismatch"));
    assert!(DOC.contains(
        "bash scripts/runtime/validate_api_version_policy_live.sh --mode dry-run --output-json /tmp/api-version-policy-live-summary.json"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/check_api_version_policy_live_policy.sh --report-file /tmp/api-version-policy-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/api-version-policy-live-policy.json"
    ));
    assert!(DOC.contains("Regression: #4041"));
}

#[test]
fn service_api_ops_configuration_contains_request_response_schema_compatibility_markers() {
    assert!(DOC.contains("## Request-Response Schema Compatibility Contract (Issue #4042)"));
    assert!(DOC.contains(
        "request_response_schema_compatibility_reason_taxonomy_version=kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "request_response_schema_compatibility_reason_codes_csv=ci_fast_gate_failed,request_response_schema_compatibility_schema_mismatch,request_response_schema_compatibility_status_invalid,request_response_schema_compatibility_final_decision_invalid,request_response_schema_compatibility_final_decision_mismatch,request_response_schema_compatibility_lane_mode_invalid,request_response_schema_compatibility_fixture_schema_mismatch,request_response_schema_compatibility_fixture_rows_invalid,request_response_schema_compatibility_fixture_row_count_mismatch,request_response_schema_compatibility_fixture_row_duplicate,request_response_schema_compatibility_fixture_row_id_invalid,request_response_schema_compatibility_fixture_row_missing,request_response_schema_compatibility_fixture_row_status_mismatch,request_response_schema_compatibility_fixture_row_decision_mismatch,request_response_schema_compatibility_fixture_row_reason_code_mismatch,request_response_schema_compatibility_fixture_row_version_pair_mismatch,request_response_schema_compatibility_fixture_row_change_class_mismatch,request_response_schema_compatibility_marker_missing,request_response_schema_compatibility_execution_reason_code_mismatch,request_response_schema_compatibility_command_count_invalid,request_response_schema_compatibility_command_count_mismatch,request_response_schema_compatibility_elapsed_seconds_invalid,request_response_schema_compatibility_max_seconds_invalid,request_response_schema_compatibility_runtime_budget_exceeded,request_response_schema_compatibility_docs_marker_missing"
    ));
    assert!(DOC.contains(
        "request_response_schema_compatibility_fixture_schema_version=kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "request_response_schema_compatibility_fixture_path=fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "request_response_schema_compatibility_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_messages_get_required_response_removal,v1_to_v2_tasks_create_required_request_removal"
    ));
    assert!(DOC.contains("schema_pair_breaking_change_detected"));
    assert!(DOC.contains("request_response_schema_compatibility_fixture_row_status_mismatch"));
    assert!(DOC.contains(
        "bash scripts/runtime/validate_request_response_schema_compatibility_live.sh --mode dry-run --output-json /tmp/request-response-schema-compatibility-live-summary.json"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/check_request_response_schema_compatibility_live_policy.sh --report-file /tmp/request-response-schema-compatibility-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/request-response-schema-compatibility-live-policy.json"
    ));
    assert!(DOC.contains("Regression: #4042"));
}

#[test]
fn service_api_ops_configuration_contains_api_compatibility_matrix_local_heavy_markers() {
    assert!(DOC.contains("## API Compatibility Matrix Local-Heavy Contract (Issue #4043)"));
    assert!(DOC.contains(
        "api_compatibility_matrix_local_heavy_reason_taxonomy_version=kamn.runtime.api-compatibility-matrix-local-heavy-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "api_compatibility_matrix_local_heavy_reason_codes_csv=ci_fast_gate_failed,api_compatibility_matrix_local_heavy_policy_schema_mismatch,api_compatibility_matrix_local_heavy_policy_status_invalid,api_compatibility_matrix_local_heavy_policy_final_decision_invalid,api_compatibility_matrix_local_heavy_policy_final_decision_mismatch,api_compatibility_matrix_local_heavy_policy_lane_mode_invalid,api_compatibility_matrix_local_heavy_policy_artifact_schema_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid,api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_duplicate,api_compatibility_matrix_local_heavy_policy_fixture_row_id_invalid,api_compatibility_matrix_local_heavy_policy_fixture_row_missing,api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_decision_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_reason_code_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_version_pair_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_route_selector_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_change_class_mismatch,api_compatibility_matrix_local_heavy_policy_marker_missing,api_compatibility_matrix_local_heavy_policy_execution_reason_code_mismatch,api_compatibility_matrix_local_heavy_policy_command_count_invalid,api_compatibility_matrix_local_heavy_policy_command_count_mismatch,api_compatibility_matrix_local_heavy_policy_elapsed_seconds_invalid,api_compatibility_matrix_local_heavy_policy_max_seconds_invalid,api_compatibility_matrix_local_heavy_policy_runtime_budget_exceeded,api_compatibility_matrix_local_heavy_policy_local_heavy_opt_in_required,api_compatibility_matrix_local_heavy_policy_local_heavy_scope_mismatch,api_compatibility_matrix_local_heavy_policy_docs_marker_missing"
    ));
    assert!(DOC.contains(
        "api_compatibility_matrix_local_heavy_fixture_schema_version=kamn.runtime.api-compatibility-matrix-local-heavy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "api_compatibility_matrix_local_heavy_fixture_path=fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "api_compatibility_matrix_local_heavy_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_tasks_create_required_request_removal,v1_to_v2_messages_get_required_response_removal,v1_to_v2_messages_send_enum_variant_removal"
    ));
    assert!(DOC.contains("incompatible_request_breaking_change"));
    assert!(DOC.contains("incompatible_response_breaking_change"));
    assert!(DOC.contains("incompatible_enum_breaking_change"));
    assert!(DOC.contains("api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch"));
    assert!(DOC.contains(
        "bash scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh --mode dry-run --ci-fast-gate PASS --output-json /tmp/api-compatibility-matrix-local-heavy-summary.json"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/check_api_compatibility_matrix_local_heavy_live_policy.sh --report-file /tmp/api-compatibility-matrix-local-heavy-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/api-compatibility-matrix-local-heavy-policy.json"
    ));
    assert!(DOC.contains("Regression: #4043"));
}

#[test]
fn service_api_ops_configuration_contains_realtime_presence_mode_and_guardrail_markers() {
    assert!(DOC.contains(
        "## Realtime Presence Mode Gateway and Guardrail Contracts (Issues #5279, #5281, #5283)"
    ));
    assert!(DOC.contains("service_api_ws_presence_mode_status=verified"));
    assert!(DOC.contains("service_api_ws_events_mode_header=x-kamn-events-mode"));
    assert!(DOC.contains("service_api_ws_events_mode_presence_value=presence"));
    assert!(DOC.contains(
        "service_api_ws_presence_required_headers_csv=x-kamn-presence-owner-did,x-kamn-presence-target-agent-did,x-kamn-requester-agent-did"
    ));
    assert!(DOC.contains(
        "service_api_ws_presence_optional_headers_csv=x-kamn-presence-target-owner-did,x-kamn-presence-gateway-node,x-kamn-presence-connected-since,x-kamn-presence-last-heartbeat,x-kamn-presence-capabilities"
    ));
    assert!(DOC.contains(
        "service_api_ws_presence_fail_closed_reason_codes_csv=service_api_ws_events_mode_invalid,service_api_ws_presence_owner_did_header_missing,service_api_ws_presence_target_agent_did_header_missing,service_api_ws_presence_requester_agent_did_header_missing,m9_realtime_owner_scope_denied,m9_realtime_presence_visibility_denied"
    ));
    assert!(DOC.contains("service_api_ws_presence_event_type=m9.presence.snapshot"));
    assert!(DOC.contains("service_api_ws_presence_transport_profile=websocket"));
    assert!(DOC.contains("realtime_guardrail_burst_validation_status=verified"));
    assert!(DOC.contains("replay_duplicate_reason_ordering_status=verified"));
    assert!(DOC.contains(
        "cargo test -p kamn-node integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable -- --exact"
    ));
    assert!(DOC.contains("Regression: #5283"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_archival_retry_policy_markers() {
    assert!(
        DOC.contains("## Phase-6 Archival Failure-Retry Policy Contracts (Issues #5285, #5287)")
    );
    assert!(DOC.contains("archival_retry_policy_status=verified"));
    assert!(DOC.contains(
        "archival_retry_reason_taxonomy_version=kamn.runtime.data-layer-m10-archival-retry-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "archival_retry_reason_codes_csv=m10_archival_retry_scheduled,m10_archival_retry_exhausted,m10_archival_failure_permanent,m10_archival_retry_policy_invalid,m10_archival_retry_attempt_invalid"
    ));
    assert!(DOC.contains(
        "archival_retry_policy_contract=max_attempts>=1;base_backoff_seconds>=1;max_backoff_seconds>=base_backoff_seconds"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c12_transient_archival_failure_projects_deterministic_retry_window -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed -- --exact"
    ));
    assert!(DOC.contains("Regression: #5287"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_execution_tick_orchestration_markers() {
    assert!(DOC.contains("## Phase-6 Retention+Archival Execution Tick Contracts (Issue #5289)"));
    assert!(DOC.contains("phase6_execution_tick_status=verified"));
    assert!(DOC.contains(
        "phase6_execution_tick_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-execution-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_reason_codes_csv=m10_phase6_execution_applied,m10_phase6_execution_owner_scope_denied,m10_phase6_execution_legal_hold_active,m10_phase6_execution_input_invalid,m10_phase6_execution_projection_input_invalid,m10_phase6_execution_projection_failed"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_contract=retention_due_lookup->crypto_shred->partition_projection->archive_due"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries -- --exact"
    ));
    assert!(DOC.contains("Regression: #5289"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_execution_tick_budget_markers() {
    assert!(DOC.contains("## Phase-6 Execution Tick Budget Guardrail Contracts (Issue #5291)"));
    assert!(DOC.contains("phase6_execution_tick_budget_status=verified"));
    assert!(DOC.contains(
        "phase6_execution_tick_budget_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-execution-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_budget_reason_codes_csv=m10_phase6_execution_budget_within_limit,m10_phase6_execution_budget_due_candidates_exceeded,m10_phase6_execution_budget_shredded_messages_exceeded,m10_phase6_execution_budget_projections_exceeded,m10_phase6_execution_budget_archive_entries_exceeded,m10_phase6_execution_budget_invalid"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_budget_contract=due_candidates<=max_due_candidates;shredded_messages<=max_shredded_messages;projection_reports<=max_projection_reports;archived_entries<=max_archived_entries"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed -- --exact"
    ));
    assert!(DOC.contains("Regression: #5291"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_scheduler_cycle_markers() {
    assert!(DOC.contains(
        "## Phase-6 Scheduler Cycle Trigger and Guarded Execution Contracts (Issue #5293)"
    ));
    assert!(DOC.contains("phase6_scheduler_cycle_status=verified"));
    assert!(DOC.contains(
        "phase6_scheduler_trigger_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-trigger-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_trigger_reason_codes_csv=m10_phase6_scheduler_trigger_deferred,m10_phase6_scheduler_trigger_due_threshold,m10_phase6_scheduler_trigger_interval_elapsed"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_cycle_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-cycle-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_cycle_reason_codes_csv=m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_policy_invalid,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded,m10_phase6_execution_budget_shredded_messages_exceeded,m10_phase6_execution_budget_projections_exceeded,m10_phase6_execution_budget_archive_entries_exceeded"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_cycle_contract=trigger_decision->preflight_budget_admission->phase6_execution_tick->budget_evidence"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution -- --exact"
    ));
    assert!(DOC.contains("Regression: #5293"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_scheduler_runtime_checkpoint_markers() {
    assert!(
        DOC.contains("## Phase-6 Stateful Scheduler Runtime Checkpoint Contracts (Issue #5295)")
    );
    assert!(DOC.contains("phase6_scheduler_runtime_checkpoint_status=verified"));
    assert!(DOC.contains(
        "phase6_scheduler_runtime_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-runtime-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_runtime_reason_codes_csv=m10_phase6_scheduler_runtime_initialized,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_runtime_state_contract=total_cycles=executed_cycles+deferred_cycles+fail_closed_cycles;last_successful_tick_epoch_seconds_updates_on_applied_only;last_observed_now_epoch_seconds_monotonic_non_decreasing"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance -- --exact"
    ));
    assert!(DOC.contains("Regression: #5295"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_runtime_evidence_bundle_markers() {
    assert!(DOC.contains("## Phase-6 Runtime Evidence Bundle Projection Contracts (Issue #5297)"));
    assert!(DOC.contains("phase6_runtime_evidence_bundle_status=verified"));
    assert!(DOC.contains(
        "phase6_runtime_evidence_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-runtime-evidence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_runtime_evidence_reason_codes_csv=m10_phase6_runtime_evidence_applied,m10_phase6_runtime_evidence_deferred,m10_phase6_runtime_evidence_input_invalid"
    ));
    assert!(DOC.contains(
        "phase6_runtime_evidence_bundle_contract=cycle_report+runtime_state->canonical_evidence_bundle;applied_requires_execution_and_budget_payload;deferred_requires_empty_execution_payload"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete -- --exact"
    ));
    assert!(DOC.contains("Regression: #5297"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_daemon_runtime_integration_markers() {
    assert!(DOC.contains("## Phase-6 Daemon Runtime Integration Contracts (Issue #5299)"));
    assert!(DOC.contains("phase6_daemon_runtime_contract_status=verified"));
    assert!(DOC.contains(
        "phase6_daemon_runtime_reason_taxonomy_version=kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_daemon_runtime_reason_codes_csv=m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded"
    ));
    assert!(DOC.contains(
        "phase6_daemon_runtime_contract=daemon_tick_executes_m10_scheduler_runtime;report_projects_phase6_reason_and_counters;clock_regression_fails_closed"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_phase6_runtime_projection_fail_closed_reason_is_stable_on_clock_regression -- --exact"
    ));
    assert!(DOC.contains("Regression: #5299"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_validation_slice_markers() {
    assert!(DOC.contains(
        "## PostgreSQL Live Integration + Daemon Runtime Validation Slice (Issue #5338)"
    ));
    assert!(DOC.contains("phase6_live_postgres_daemon_runtime_slice_status=verified"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_env_gate=KAMN_TEST_POSTGRES_URL|DATABASE_URL"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_reason_codes_csv=live_postgres_env_unset,live_postgres_adapter_connected,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_slice_contract=live_postgres_env_gate->adapter_connect_and_migrate->daemon_phase6_runtime_projection"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_postgres_execution_adapter spec_c01_and_c03_live_adapter_executes_insert_and_lookup_with_session_context -- --exact"
    ));
    assert!(DOC.contains("Regression: #5338"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_gate_and_deferred_markers() {
    assert!(DOC.contains("### Gate and Deferred Path Hardening (Issue #5340)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_gate_reason_contract=env_unset->skip_with_reason;env_set->adapter_connect_and_migrate"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_deferred_contract=live_postgres_adapter_connected+shutdown_signal->m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path -- --exact"
    ));
    assert!(DOC.contains("Regression: #5340"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_stability_markers() {
    assert!(DOC.contains("### Scenario Matrix Stability (Issue #5342)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_contract=env_unset->live_postgres_env_unset;env_set_no_shutdown->m10_phase6_scheduler_cycle_applied;env_set_shutdown->m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_stability_contract=repeated_runs_preserve_reason_code_per_matrix_scenario"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs -- --exact"
    ));
    assert!(DOC.contains("Regression: #5342"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_ordering_markers(
) {
    assert!(DOC.contains("### Matrix Taxonomy and Canonical Ordering (Issue #5344)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_reason_codes_csv=live_postgres_env_unset,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_scenarios_csv=env_unset,env_set_no_shutdown,env_set_shutdown"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_order_contract=matrix_rows_order=env_unset->env_set_no_shutdown->env_set_shutdown;reason_codes_align_with_scenarios"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_ordering_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5344"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_bridge_markers(
) {
    assert!(DOC.contains("### Runtime-to-Matrix Taxonomy Bridge (Issue #5346)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_reason_taxonomy_version=kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres-matrix.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_taxonomy_bridge_contract=runtime_reason_taxonomy_v1->matrix_scenario_taxonomy_v1;applied_and_deferred_reasons_must_align"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_taxonomy_versions_are_stable_across_repeated_runs -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_bridge_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5346"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_load_profile_markers()
{
    assert!(DOC.contains("### Bounded Load-Profile Matrix (Issue #5348)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_load_profile_ids_csv=applied_t3_i10,applied_t5_i25,applied_t9_i40,deferred_t5_i25_s3_d2_to4,deferred_t7_i25_s3_d2_to4,deferred_t9_i40_s3_d2_to4"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_load_profile_contract=applied_profiles->m10_phase6_scheduler_cycle_applied;deferred_profiles->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_load_profile_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5348"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_profile_markers()
{
    assert!(DOC.contains("### Role-Profile Matrix Determinism (Issue #5350)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_profile_ids_csv=processor_applied,processor_deferred,listener_applied,listener_deferred,approver_applied,approver_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_profile_contract=processor|listener|approver_applied->m10_phase6_scheduler_cycle_applied;processor|listener|approver_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_profile_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5350"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_pair_markers() {
    assert!(DOC.contains("### Two-Node Role-Pair Matrix (Issue #5352)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_pair_ids_csv=processor_to_listener_applied,processor_to_listener_deferred,listener_to_approver_applied,listener_to_approver_deferred,approver_to_processor_applied,approver_to_processor_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_role_pair_contract=role_pair_leg_a_applied->m10_phase6_scheduler_cycle_applied;role_pair_leg_b_applied->m10_phase6_scheduler_cycle_applied;role_pair_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;role_pair_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_pair_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5352"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_role_pair_markers(
) {
    assert!(DOC.contains("### Bounded Parallel Role-Pair Lane Matrix (Issue #5354)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_parallel_role_pair_lane_ids_csv=processor_listener_parallel_applied,processor_listener_parallel_deferred,listener_approver_parallel_applied,listener_approver_parallel_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_parallel_role_pair_contract=parallel_lane_leg_a_applied->m10_phase6_scheduler_cycle_applied;parallel_lane_leg_b_applied->m10_phase6_scheduler_cycle_applied;parallel_lane_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;parallel_lane_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_role_pair_lane_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_role_pair_lane_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_role_pair_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5354"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_markers(
) {
    assert!(DOC.contains("### Asymmetric Parallel Lane Matrix (Issue #5356)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_ids_csv=processor_listener_asymmetric_parallel_applied,processor_listener_asymmetric_parallel_deferred,listener_approver_asymmetric_parallel_applied,listener_approver_asymmetric_parallel_deferred"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_asymmetric_parallel_contract=asymmetric_parallel_leg_a_applied->m10_phase6_scheduler_cycle_applied;asymmetric_parallel_leg_b_applied->m10_phase6_scheduler_cycle_applied;asymmetric_parallel_leg_a_deferred->m10_phase6_scheduler_cycle_deferred;asymmetric_parallel_leg_b_deferred->m10_phase6_scheduler_cycle_deferred;runtime_taxonomy_version_stable"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5356"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_order_invariance_markers(
) {
    assert!(DOC.contains("### Parallel Lane Order-Invariance Matrix (Issue #5358)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_order_invariance_contract=baseline_and_permuted_lane_orders_must_produce_equivalent_sorted_reason_taxonomy_fingerprints"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_order_invariance_lane_sets_csv=symmetric_parallel,asymmetric_parallel"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_order_invariance_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5358"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_permutation_invariance_markers(
) {
    assert!(DOC.contains("### Parallel Lane Permutation-Invariance Matrix (Issue #5360)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_permutation_ids_csv=baseline,reverse,rotate_left_1,interleaved_even_then_odd"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_matrix_permutation_invariance_contract=deterministic_permutations_must_preserve_sorted_lane_reason_taxonomy_fingerprints"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_permutation_invariance_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5360"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_fingerprint_schema_markers(
) {
    assert!(DOC.contains("### Parallel Lane Fingerprint Schema Contracts (Issue #5362)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_fingerprint_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-fingerprint.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_fingerprint_field_order_csv=lane_id,leg_a_reason,leg_a_taxonomy,leg_b_reason,leg_b_taxonomy"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_fingerprint_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_fingerprint_schema_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5362"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_scope_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology-Scope Contracts (Issue #5364)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_ids_csv=same_host_parallel,distributed_label_parallel"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_contract=topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_scope_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5364"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_permutation_markers(
) {
    assert!(
        DOC.contains("### Parallel Lane Topology Permutation-Invariance Contracts (Issue #5366)")
    );
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_permutation_ids_csv=baseline,reverse,rotate_left_1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_permutation_contract=deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_permutation_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_permutations_are_invariant -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_permutation_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5366"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Host-Pair Contracts (Issue #5368)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_required_host_pair_ids_csv=node_alpha->node_alpha,node_alpha->node_beta"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_contract=host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pairs_are_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5368"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_directionality_markers(
) {
    assert!(
        DOC.contains("### Parallel Lane Topology Host-Pair Directionality Contracts (Issue #5370)")
    );
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_extraction_rule=host_a_to_host_b_arrow_notation_non_commutative"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_directionality_forbidden_reverse_pairs_csv=node_beta->node_alpha"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_directionality_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5370"));
}

#[test]
fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_mapping_markers(
) {
    assert!(DOC.contains("### Parallel Lane Topology Host-Pair Mapping Contracts (Issue #5372)"));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_schema_version=kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-mapping.v1"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_rows_csv=same_host_parallel->node_alpha->node_alpha,distributed_label_parallel->node_alpha->node_beta"
    ));
    assert!(DOC.contains(
        "phase6_live_postgres_daemon_runtime_parallel_lane_topology_host_pair_mapping_contract=topology_id_to_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_contract_is_canonical -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_mapping_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_mapping_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5372"));
}

#[test]
fn service_api_ops_configuration_contains_convergence_promotion_marker_contracts() {
    assert!(DOC.contains("## Convergence Promotion Marker Contracts (Issue #5301)"));
    assert!(DOC.contains("convergence_promotion_contract_status=verified"));
    assert!(DOC.contains(
        "convergence_reason_taxonomy_version=kamn.runtime.daemon.convergence.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "convergence_reason_codes_csv=convergence_promotion_gate_go,convergence_schema_drift_detected,convergence_error_path_drift_detected,convergence_concurrency_drift_detected,convergence_performance_budget_exceeded,convergence_cost_budget_exceeded"
    ));
    assert!(DOC.contains(
        "convergence_promotion_contract=schema+error_path+concurrency+performance+cost->decision;any_failed_gate=no_go"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_convergence_projection_fail_closed_reason_is_stable -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5301"));
}

#[test]
fn service_api_ops_configuration_contains_quota_policy_fixture_matrix_controls() {
    assert!(
        DOC.contains("## Quota Policy Fixture Matrix and Parser Helper Contracts (Issue #4090)")
    );
    assert!(DOC.contains(
        "quota_policy_fixture_matrix_path=fixtures/runtime/quota_policy_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "quota_policy_fixture_matrix_schema_version=kamn.runtime.quota-policy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "quota_policy_reason_taxonomy_version=kamn.runtime.quota-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "quota_policy_reason_codes_csv=quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive"
    ));
    assert!(DOC.contains(
        "quota_policy_fixture_columns=case_id|scope|window_seconds|limit|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("quota_scope_unknown"));
    assert!(DOC.contains("quota_window_non_positive"));
    assert!(DOC.contains("quota_limit_non_positive"));
    assert!(DOC.contains("cargo test -p kamn-core --test quota_policy_fixture_parser_contract"));
    assert!(DOC.contains("Regression: #4090"));
}

#[test]
fn service_api_ops_configuration_contains_fairness_starvation_fixture_controls() {
    assert!(DOC.contains("## Fairness Starvation Fixture and Checker Contracts (Issue #4092)"));
    assert!(DOC.contains(
        "fairness_fixture_matrix_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "fairness_fixture_matrix_schema_version=kamn.runtime.fairness-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "fairness_reason_taxonomy_version=kamn.runtime.fairness-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "fairness_reason_codes_csv=fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    ));
    assert!(DOC.contains(
        "fairness_fixture_columns=case_id|scope|window_seconds|active_weighted_share|max_weighted_share_gap|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("fairness_scope_unknown"));
    assert!(DOC.contains("fairness_window_non_positive"));
    assert!(DOC.contains("fairness_max_gap_non_positive"));
    assert!(DOC.contains("fairness_weighted_share_exceeds_gap"));
    assert!(DOC.contains("cargo test -p kamn-core --test fairness_policy_checker_contract"));
    assert!(DOC.contains("Regression: #4092"));
}

#[test]
fn service_api_ops_configuration_contains_overload_docs_parity_remediation_controls() {
    assert!(DOC.contains("## Daemon OS-Signal Stress Matrix Overload Profiles (Issue #4094)"));
    assert!(DOC.contains(
        "daemon_os_signal_stress_matrix_schema_version=kamn.ci.daemon-os-signal-stress-matrix-report.v1"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_reason_taxonomy_version=kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_reason_codes_csv=runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success"
    ));
    assert!(DOC.contains("overload_docs_parity_remediation_map_version=v1"));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.runtime_budget_exceeded=reduce iterations or increase max-seconds budget after validating reproducer runtime"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.matrix_failure_threshold_exceeded=triage failing iteration artifacts and rerun reproducer before promotion"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.quarantine_registry_missing=restore .ci/flaky-tests.txt or pass an explicit --registry-file"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.quarantine_reference_present_without_followup=add --quarantine-followup-issue #<id> or retire stale quarantine entries"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.matrix_failures_within_threshold=track flaky rows and keep threshold + waiver evidence attached to release review"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.stable_success_with_quarantine_followup=keep follow-up issue open until quarantine references are retired"
    ));
    assert!(DOC.contains(
        "overload_docs_parity_remediation.stable_success=no action required; retain report artifact link in release checklist"
    ));
    assert!(DOC.contains("Regression: #4097"));
}

#[test]
fn service_api_ops_configuration_contains_protocol_mismatch_reason_mapping_controls() {
    assert!(DOC.contains(
        "## API Protocol Compliance Mismatch Reason Mapping (Issues #4266, #4270, #4271)"
    ));
    assert!(DOC.contains("service_api_axum_protocol_mismatch_reason_mapping_status=verified"));
    assert!(DOC.contains(
        "service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation"
    ));
    assert!(DOC.contains("service_api_axum_protocol_mismatch_reason_code=none|<reason>"));
    assert!(DOC.contains("service_api_axum_policy_protocol_taxonomy_mismatch"));
    assert!(DOC.contains("service_api_axum_policy_limit_contract_mismatch"));
    assert!(DOC.contains("Regression: #4270"));
    assert!(DOC.contains("Regression: #4271"));
}

#[test]
fn service_api_ops_configuration_contains_audit_integrity_tamper_controls() {
    assert!(DOC.contains("## Audit Integrity Go/No-Go Policy Controls (Issue #4465)"));
    assert!(DOC.contains(
        "audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("gonogo_audit_integrity_reason_taxonomy_version_mismatch"));
    assert!(DOC.contains("gonogo_audit_integrity_reason_codes_csv_mismatch"));
    assert!(DOC.contains("audit integrity gate convergence mismatch"));
    assert!(DOC.contains("Regression: #4465"));
}

#[test]
fn service_api_ops_configuration_contains_journal_append_checkpoint_integrity_controls() {
    assert!(DOC
        .contains("## Journal Append/Checkpoint Integrity Controls (Issues #4236, #4240, #4241)"));
    assert!(DOC.contains("append_checkpoint_integrity_status=verified"));
    assert!(DOC.contains(
        "append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch"
    ));
    assert!(DOC.contains("sqlite_crash_recovery_policy_wal_append_status_mismatch"));
    assert!(DOC.contains("sqlite_crash_recovery_policy_wal_checkpoint_status_mismatch"));
    assert!(DOC.contains("sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch"));
    assert!(DOC.contains("Regression: #4240"));
    assert!(DOC.contains("Regression: #4241"));
}

#[test]
fn service_api_ops_configuration_contains_in_memory_provider_rejection_controls() {
    assert!(DOC.contains("## Production-Mode In-Memory Provider Rejection Controls (Issue #4371)"));
    assert!(DOC.contains("runtime_commit_in_memory_provider_reference_detected"));
    assert!(DOC.contains("runtime_commit_policy_check_in_memory_provider_reference_detected"));
    assert!(DOC.contains("InMemoryKolmeRuntimeCommitClient"));
    assert!(DOC.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(DOC.contains("Regression: #4371"));
}

#[test]
fn service_api_ops_configuration_contains_signer_material_validation_and_fallback_prohibition_contracts(
) {
    assert!(DOC.contains(
        "## Signer Material Validation and Fallback Prohibition Contracts (Issues #4167, #4168)"
    ));
    assert!(DOC.contains(
        "signer_config_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-signer-config-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "signer_config_reason_codes_csv=signer_secret_missing,signer_secret_invalid_hex,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing"
    ));
    assert!(DOC.contains("signer_config_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("signer_secret_missing"));
    assert!(DOC.contains("signer_secret_invalid_hex"));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
    assert!(DOC.contains("fallback_signer_secret_checkpoint_reason_mismatch"));
    assert!(DOC.contains("fallback_signer_secret_remediation_missing"));
    assert!(DOC.contains(
        "runtime_signer_key_source_policy_reason_codes_csv=production_signer_key_source_env_local_forbidden,fallback_signer_secret_present_violation"
    ));
    assert!(DOC.contains(
        "managed_signer_provenance_reason_codes_csv=managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch"
    ));
    assert!(DOC.contains("managed_signer_backend_response_provenance_missing"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_malformed"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_mismatch"));
    assert!(DOC.contains("signer secret env is required for selected profile"));
    assert!(DOC.contains("fallback signer secret env must not be set"));
    assert!(DOC.contains("remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"));
    assert!(DOC.contains("test_run_local_kolme_live_deployment_preflight_lane.sh"));
    assert!(DOC.contains("test_check_local_kolme_live_deployment_preflight_policy.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture"
    ));
    assert!(DOC.contains(
        "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json"
    ));
    assert!(DOC.contains("Regression: #4167"));
    assert!(DOC.contains("Regression: #4168"));
}

#[test]
fn service_api_ops_configuration_contains_managed_key_source_adapter_provenance_mapping() {
    assert!(DOC.contains("managed_key_source_adapter_provenance_status=verified"));
    assert!(DOC.contains(
        "managed_key_source_adapter_provenance_fields_csv=profile,key_source,key_reference_env,signer_public_key_hex"
    ));
    assert!(DOC.contains(
        "managed_key_source_adapter_provenance_reason_codes_csv=managed_signer_provenance_marker_profile_mismatch,managed_signer_provenance_marker_key_source_mismatch,managed_signer_provenance_marker_key_reference_env_mismatch,managed_signer_provenance_marker_public_key_missing"
    ));
    assert!(DOC.contains("managed_signer_provenance_marker_profile_mismatch"));
    assert!(DOC.contains("managed_signer_provenance_marker_key_source_mismatch"));
    assert!(DOC.contains("managed_signer_provenance_marker_key_reference_env_mismatch"));
    assert!(DOC.contains("managed_signer_provenance_marker_public_key_missing"));
    assert!(DOC.contains(
        "cargo test -p kamn-node signer::managed_backend::tests::unit_managed_key_source_adapter_emits_deterministic_provenance_marker -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node signer::tests::regression_managed_key_source_provenance_marker_profile_mismatch_fails_closed -- --exact"
    ));
    assert!(DOC.contains("Regression: #3955"));
}

#[test]
fn service_api_ops_configuration_contains_multi_signer_quorum_signature_decision_controls() {
    assert!(DOC
        .contains("## Multi-Signer Profile and Quorum Signature-Decision Controls (Issue #4357)"));
    assert!(DOC.contains(
        "signature_decision_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signature-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "signature_decision_reason_codes_csv=runtime_signer_profile_missing,runtime_signer_profile_invalid,runtime_signer_previous_profile_missing,runtime_signer_previous_profile_invalid,runtime_signer_failover_profile_unchanged,runtime_signer_profile_changed_without_failover,runtime_signer_rotation_epoch_stale,runtime_signer_rotation_epoch_regressed,runtime_signer_attestation_schema_invalid,runtime_signer_attestation_required_approvals_invalid,runtime_signer_attestation_approved_signers_invalid,runtime_signer_attestation_approved_signers_not_unique,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_quorum_linkage_contract_version_invalid,runtime_signer_quorum_linkage_contract_version_mismatch,runtime_signer_quorum_required_approvals_invalid,runtime_signer_quorum_required_approvals_mismatch,runtime_signer_quorum_approved_signers_count_invalid,runtime_signer_quorum_approved_signers_count_mismatch,runtime_signer_quorum_profile_linked_invalid,runtime_signer_quorum_profile_linked_mismatch,runtime_signer_quorum_satisfied_invalid,runtime_signer_quorum_satisfied_mismatch,runtime_signer_quorum_linked_invalid,runtime_signer_quorum_linkage_drift,runtime_signer_quorum_linkage_violation,runtime_signer_failover_attestation_required_approvals_insufficient,runtime_signer_failover_attestation_previous_profile_not_approved"
    ));
    assert!(DOC.contains("signature_decision_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("runtime_signer_attestation_quorum_shortfall"));
    assert!(DOC.contains("runtime_signer_quorum_linkage_drift"));
    assert!(DOC.contains("Regression: #4357"));
}

#[test]
fn service_api_ops_configuration_contains_signer_quorum_profile_matrix_controls() {
    assert!(DOC.contains("signer_quorum_profile_matrix_fixture_status=verified"));
    assert!(DOC.contains(
        "signer_quorum_profile_matrix_case_labels_csv=linked_non_failover_primary,profile_not_approved_non_failover,quorum_shortfall_non_failover,failover_previous_profile_not_approved,linked_failover_dual_approved"
    ));
    assert!(DOC.contains(
        "signer_quorum_profile_matrix_fail_closed_reason_codes_csv=runtime_signer_quorum_linkage_violation,runtime_signer_attestation_quorum_shortfall,runtime_signer_failover_attestation_previous_profile_not_approved"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node signer::signer_policy::tests::unit_signer_quorum_decision_path_matrix -- --exact --nocapture"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::signer_tests::integration_kolme_live_signer_preflight_quorum_profile_matrix_paths -- --exact --nocapture"
    ));
    assert!(DOC.contains("Regression: #3957"));
}

#[test]
fn service_api_ops_configuration_contains_retry_envelope_exhaustion_reconnect_bound_governance() {
    assert!(
        DOC.contains("### Retry Envelope Exhaustion and Reconnect Bound Governance (Issue #4296)")
    );
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v2"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,local_retry_envelope_exhaustion_fail_closed_missing,local_retry_reconnect_attempt_bound_drift,local_retry_reconnect_backoff_bound_drift,ci_local_network_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("retry_envelope_exhaustion_fail_closed_status=verified"));
    assert!(DOC.contains("reconnect_attempt_bound_status=verified"));
    assert!(DOC.contains("reconnect_backoff_bound_status=verified"));
    assert!(DOC.contains("retry_envelope_max_attempts=3"));
    assert!(DOC.contains("retry_envelope_max_backoff_seconds=8"));
    assert!(DOC.contains("local_retry_envelope_exhaustion_fail_closed_missing"));
    assert!(DOC.contains("local_retry_reconnect_attempt_bound_drift"));
    assert!(DOC.contains("local_retry_reconnect_backoff_bound_drift"));
    assert!(DOC.contains("Regression: #4300"));
    assert!(DOC.contains("Regression: #4301"));
}

#[test]
fn service_api_ops_configuration_contains_live_node_drift_marker_mismatch_policy_contracts() {
    assert!(DOC.contains("### Live-Node Drift Marker Mismatch Policy Contracts (Issue #4281)"));
    assert!(DOC.contains("failover_promotion_gate_status=verified"));
    assert!(DOC.contains("live_node_drift_parity_status=verified"));
    assert!(DOC.contains("ci_local_promotion_budget_boundary_status=verified"));
    assert!(DOC.contains(
        "failover_readiness_reason_taxonomy_version=kamn.runtime.failover-readiness-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "failover_readiness_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("failover_sync_drift_policy_status=verified"));
    assert!(DOC.contains(
        "bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-policy"
    ));
    assert!(DOC.contains("live_node_drift_marker_parity_mismatch"));
    assert!(DOC.contains("failover_readiness_progress_stalled"));
    assert!(DOC.contains("ci_local_promotion_budget_boundary_exceeded"));
    assert!(DOC.contains("failover_sync_drift_policy_required_field_missing:<field>"));
    assert!(DOC.contains("failover_sync_drift_policy_reason_taxonomy_version_mismatch"));
    assert!(DOC.contains("failover_sync_drift_policy_reason_codes_csv_mismatch"));
    assert!(DOC.contains("Regression: #4285"));
    assert!(DOC.contains("Regression: #4286"));
}

#[test]
fn service_api_ops_configuration_contains_shutdown_checkpoint_reconciliation_failure_modes() {
    assert!(DOC.contains("Shutdown signal failure matrix"));
    assert!(DOC.contains("full_supervisor_stop_graceful_drain_timeout_contract_mismatch"));
    assert!(DOC.contains(
        "shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch"));
    assert!(DOC.contains("shutdown_checkpoint_reconciliation_not_signaled_checkpoint_mismatch"));
    assert!(DOC.contains("runtime_shutdown_invariant_violation:<reason_code>"));
    assert!(DOC.contains("Regression: #4332"));
    assert!(DOC.contains("Regression: #4333"));
}

#[test]
fn service_api_ops_configuration_contains_full_stack_harness_marker_mismatch_controls() {
    assert!(DOC.contains(
        "## Full-Stack Harness Marker Completeness and Parity Mismatch Controls (Issue #4195)"
    ));
    assert!(DOC.contains("full_io_harness_marker_completeness_status=verified"));
    assert!(DOC.contains("full_io_harness_marker_parity_status=verified"));
    assert!(DOC.contains(
        "full_io_harness_policy_reason_taxonomy_version=kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "full_io_harness_policy_reason_codes_csv=full_io_scenario_matrix_policy_process_harness_mismatch,full_io_scenario_matrix_policy_api_route_matrix_mismatch,full_io_scenario_matrix_policy_auth_failure_matrix_mismatch,full_io_scenario_matrix_policy_websocket_matrix_mismatch,full_io_scenario_matrix_policy_multinode_propagation_mismatch,full_io_scenario_matrix_policy_dry_run_command_count_mismatch,full_io_scenario_matrix_policy_dry_run_command_status_mismatch"
    ));
    assert!(DOC.contains("full_io_scenario_matrix_policy_process_harness_mismatch"));
    assert!(DOC.contains("full_io_scenario_matrix_policy_dry_run_command_count_mismatch"));
    assert!(DOC.contains("full_io_scenario_matrix_policy_dry_run_command_status_mismatch"));
    assert!(DOC.contains("Regression: #4195"));
}

#[test]
fn service_api_ops_configuration_contains_upgrade_compatibility_marker_matrix_controls() {
    assert!(DOC.contains("## Upgrade Compatibility Marker Matrix Controls (Issue #4181)"));
    assert!(DOC.contains(
        "check_upgrade_compatibility_marker_matrix_policy.py --version-report-file /tmp/kolme-version-report.json --fork-policy-report-file /tmp/kolme-fork-compatibility-policy-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-upgrade-compatibility-marker-matrix-policy-report.json"
    ));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed"
    ));
    assert!(DOC.contains("version_report_schema_mismatch"));
    assert!(DOC.contains("fork_policy_report_reason_codes_csv_mismatch"));
    assert!(DOC.contains("fork_policy_report_rehearsal_bypass_guard_status_mismatch"));
    assert!(DOC.contains("Regression: #4180"));
    assert!(DOC.contains("Regression: #4181"));
}

#[test]
fn service_api_ops_configuration_contains_partition_healing_mismatch_mapping_controls() {
    assert!(DOC.contains(
        "### Block Reconciliation Partition-Healing Mismatch Mapping Contracts (Issues #4251, #4255, #4256)"
    ));
    assert!(DOC.contains("partition_healing_mismatch_reason_mapping_status=verified"));
    assert!(DOC.contains(
        "partition_healing_mismatch_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-healing-mismatch-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "partition_healing_mismatch_reason_codes_csv=block_reconciliation_partition_rejoin_policy_required_field_missing,block_reconciliation_partition_rejoin_policy_marker_mismatch,block_reconciliation_partition_rejoin_policy_transport_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_mismatch,block_reconciliation_partition_rejoin_policy_recovery_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid,block_reconciliation_partition_rejoin_policy_lane_mode_contract_mismatch,block_reconciliation_partition_rejoin_policy_ci_fast_gate_failed,block_reconciliation_partition_rejoin_policy_expected_decision_mismatch,block_reconciliation_partition_rejoin_policy_violation"
    ));
    assert!(DOC.contains("partition_healing_mismatch_reason_code=none|<reason>"));
    assert!(
        DOC.contains("block_reconciliation_partition_rejoin_policy_required_field_missing:<field>")
    );
    assert!(DOC.contains(
        "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid"
    ));
    assert!(DOC.contains("Regression: #4255"));
    assert!(DOC.contains("Regression: #4256"));
}
