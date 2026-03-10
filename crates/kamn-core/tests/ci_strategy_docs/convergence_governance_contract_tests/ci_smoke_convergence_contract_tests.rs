use super::super::DOC;
use super::super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_transport_observability_tls_ci_smoke_convergence_governance() {
    assert_contains_all(
        DOC,
        &[
            "Transport/Observability/TLS CI smoke convergence governance",
            "python3 scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py",
            "test_check_transport_observability_tls_ci_smoke_convergence.sh",
            "transport_observability_tls_reason_taxonomy_version=kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1",
            "transport_observability_tls_reason_codes_csv=transport_ci_smoke_composition_missing,observability_ci_smoke_composition_missing,tls_ci_smoke_composition_missing,transport_local_heavy_command_leaked_in_fast_mode,observability_local_heavy_command_leaked_in_fast_mode,tls_local_heavy_command_leaked_in_fast_mode,ci_fast_gate_transport_run_mode_not_excluded,ci_fast_gate_observability_run_mode_not_excluded,ci_fast_gate_tls_deep_lane_not_excluded,ci_strategy_convergence_markers_missing,production_plan_convergence_markers_missing,transport_observability_tls_ci_smoke_seconds_exceeded",
            "transport_observability_tls_ci_smoke_max_seconds=120",
            "transport_observability_tls_local_heavy_max_seconds=900",
            "transport_observability_tls_ci_smoke_lane_cost_profile=low",
            "transport_observability_tls_local_heavy_execution_mode=opt_in",
            "transport_ci_smoke_composition_missing",
            "observability_ci_smoke_composition_missing",
            "tls_ci_smoke_composition_missing",
            "transport_observability_tls_ci_smoke_seconds_exceeded",
            "Regression: #4299",
        ],
        "transport observability tls convergence",
    );
}

#[test]
fn doc_contains_admission_backpressure_ci_smoke_convergence_governance() {
    assert_contains_all(
        DOC,
        &[
            "### Admission-Backpressure CI smoke convergence governance",
            "python3 scripts/ci/check_admission_backpressure_ci_smoke_convergence.py",
            "bash scripts/ci/test_check_admission_backpressure_ci_smoke_convergence.sh",
            "admission_backpressure_ci_smoke_reason_taxonomy_version=kamn.ci.admission-backpressure-ci-smoke-convergence-reason-taxonomy.v1",
            "admission_backpressure_ci_smoke_reason_codes_csv=service_api_axum_policy_ci_smoke_composition_missing,service_api_axum_contract_lane_ci_smoke_composition_missing,service_api_axum_run_command_leaked_in_fast_mode,ci_fast_gate_service_api_axum_run_command_not_excluded,ci_strategy_admission_backpressure_convergence_markers_missing,production_plan_admission_backpressure_convergence_markers_missing,admission_backpressure_ci_smoke_seconds_exceeded",
            "admission_backpressure_ci_smoke_max_seconds=120",
            "admission_backpressure_local_heavy_max_seconds=900",
            "admission_backpressure_ci_smoke_lane_cost_profile=low",
            "admission_backpressure_local_heavy_execution_mode=opt_in",
            "service_api_axum_run_command_leaked_in_fast_mode",
            "ci_fast_gate_service_api_axum_run_command_not_excluded",
            "admission_backpressure_ci_smoke_seconds_exceeded",
        ],
        "admission backpressure convergence",
    );
}

#[test]
fn doc_contains_sqlite_crash_replay_ci_smoke_convergence_governance() {
    assert_contains_all(
        DOC,
        &[
            "### SQLite Crash-Replay CI smoke convergence governance",
            "python3 scripts/ci/check_sqlite_crash_recovery_ci_smoke_convergence.py",
            "bash scripts/ci/test_check_sqlite_crash_recovery_ci_smoke_convergence.sh",
            "sqlite_crash_recovery_ci_smoke_reason_taxonomy_version=kamn.ci.sqlite-crash-recovery-ci-smoke-convergence-reason-taxonomy.v1",
            "sqlite_crash_recovery_ci_smoke_reason_codes_csv=sqlite_crash_recovery_validate_ci_smoke_composition_missing,sqlite_crash_recovery_policy_ci_smoke_composition_missing,sqlite_crash_recovery_contract_lane_ci_smoke_composition_missing,sqlite_crash_recovery_evidence_ci_smoke_composition_missing,sqlite_crash_recovery_run_mode_command_leaked_in_fast_mode,ci_fast_gate_sqlite_crash_recovery_run_mode_not_excluded,ci_strategy_sqlite_crash_recovery_convergence_markers_missing,production_plan_sqlite_crash_recovery_convergence_markers_missing,sqlite_crash_recovery_ci_smoke_seconds_exceeded",
            "sqlite_crash_recovery_ci_smoke_max_seconds=120",
            "sqlite_crash_recovery_local_heavy_max_seconds=900",
            "sqlite_crash_recovery_ci_smoke_lane_cost_profile=low",
            "sqlite_crash_recovery_local_heavy_execution_mode=opt_in",
            "sqlite_crash_recovery_run_mode_command_leaked_in_fast_mode",
            "ci_fast_gate_sqlite_crash_recovery_run_mode_not_excluded",
            "sqlite_crash_recovery_ci_smoke_seconds_exceeded",
        ],
        "sqlite crash replay convergence",
    );
}

#[test]
fn doc_contains_failover_drift_ci_smoke_convergence_governance() {
    assert_contains_all(
        DOC,
        &[
            "failover_drift_ci_smoke_reason_codes_csv=failover_selector_ci_smoke_composition_missing,failover_preflight_ci_smoke_composition_missing,failover_deep_lane_guard_ci_smoke_composition_missing,failover_suite_ci_smoke_composition_missing,failover_deep_lane_run_command_leaked_in_fast_mode,ci_fast_gate_failover_deep_lane_not_excluded,ci_strategy_failover_convergence_markers_missing,production_plan_failover_convergence_markers_missing,failover_drift_ci_smoke_seconds_exceeded",
            "failover_drift_ci_smoke_max_seconds=120",
            "failover_drift_local_heavy_max_seconds=900",
            "failover_drift_ci_smoke_lane_cost_profile=low",
            "failover_drift_local_heavy_execution_mode=opt_in",
            "failover_deep_lane_run_command_leaked_in_fast_mode",
            "ci_fast_gate_failover_deep_lane_not_excluded",
            "failover_drift_ci_smoke_seconds_exceeded",
        ],
        "failover drift convergence",
    );
}

#[test]
fn doc_contains_websocket_session_ci_smoke_convergence_governance() {
    assert_contains_all(
        DOC,
        &[
            "websocket_session_ci_smoke_reason_codes_csv=websocket_validate_ci_smoke_composition_missing,websocket_policy_ci_smoke_composition_missing,websocket_contract_ci_smoke_composition_missing,websocket_session_drill_run_command_leaked_in_fast_mode,ci_fast_gate_websocket_session_drill_not_excluded,ci_strategy_websocket_session_convergence_markers_missing,production_plan_websocket_session_convergence_markers_missing,websocket_session_ci_smoke_seconds_exceeded",
            "websocket_session_ci_smoke_max_seconds=120",
            "websocket_session_local_heavy_max_seconds=900",
            "websocket_session_ci_smoke_lane_cost_profile=low",
            "websocket_session_local_heavy_execution_mode=opt_in",
            "websocket_session_drill_run_command_leaked_in_fast_mode",
            "ci_fast_gate_websocket_session_drill_not_excluded",
            "websocket_session_ci_smoke_seconds_exceeded",
        ],
        "websocket session convergence",
    );
}

#[test]
fn doc_contains_partition_finality_ci_smoke_convergence_governance() {
    assert_contains_all(
        DOC,
        &[
            "partition_finality_ci_smoke_reason_codes_csv=libp2p_validate_ci_smoke_composition_missing,libp2p_policy_ci_smoke_composition_missing,libp2p_contract_lane_ci_smoke_composition_missing,libp2p_evidence_ci_smoke_composition_missing,partition_finality_run_mode_command_leaked_in_fast_mode,ci_fast_gate_partition_finality_run_mode_not_excluded,ci_strategy_partition_finality_convergence_markers_missing,production_plan_partition_finality_convergence_markers_missing,partition_finality_ci_smoke_seconds_exceeded",
            "partition_finality_ci_smoke_max_seconds=120",
            "partition_finality_local_heavy_max_seconds=900",
            "partition_finality_ci_smoke_lane_cost_profile=low",
            "partition_finality_local_heavy_execution_mode=opt_in",
            "partition_finality_run_mode_command_leaked_in_fast_mode",
            "ci_fast_gate_partition_finality_run_mode_not_excluded",
            "partition_finality_ci_smoke_seconds_exceeded",
        ],
        "partition finality convergence",
    );
}
