const DOC: &str = include_str!("../../../docs/foundation/observability-slo-dashboards.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn doc_contains_observability_models_and_monitor_outputs() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ObservabilitySample"));
    assert!(DOC.contains("ObservabilitySloProfile"));
    assert!(DOC.contains("ObservabilityMonitor"));
    assert!(DOC.contains("ObservabilitySnapshot"));
    assert!(DOC.contains("packages/kamn-dashboard"));
}

#[test]
fn doc_contains_slo_alert_severity_rules() {
    assert!(DOC.contains("## SLO Evaluation Rules"));
    assert!(DOC.contains("`LatencyP99`: critical when above max threshold."));
    assert!(DOC.contains("`ErrorRate`:"));
    assert!(DOC.contains("critical when above 2x max threshold."));
    assert!(DOC.contains("`Availability`: critical when below minimum threshold."));
}

#[test]
fn doc_contains_post_cutover_slo_evidence_contract() {
    assert!(DOC.contains("## Post-Cutover SLO Gate Evidence Contract"));
    assert!(DOC.contains("generate_post_cutover_slo_evidence_bundle.sh"));
    assert!(DOC.contains("check_post_cutover_slo_policy.sh"));
    assert!(DOC.contains("post_cutover_slo_contract_lane_contract.py"));
    assert!(DOC.contains("run_post_cutover_slo_contract_lane.sh"));
    assert!(DOC.contains("run_post_cutover_slo_deep_lane.sh"));
}

#[test]
fn doc_contains_slo_alert_policy_checker_contract() {
    assert!(DOC.contains("## SLO/Alert Evidence Policy Checker Contract"));
    assert!(DOC.contains("slo_alert_reason_codes:GO:v1"));
    assert!(DOC.contains("slo_alert_reason_codes:NO-GO:v1"));
    assert!(DOC.contains("KAMN_POST_CUTOVER_SLO_MAX_SECONDS"));
}

#[test]
fn doc_contains_dashboard_stale_error_budget_contract_lane() {
    assert!(DOC.contains("## Dashboard Stale/Error Budget Policy Checker Contract"));
    assert!(DOC.contains("stale_error_budget_policy_contract.py"));
    assert!(DOC.contains("stale_error_budget_lane_contract.py"));
    assert!(DOC.contains("stale_error_budget_contract_lane_contract.py"));
    assert!(DOC.contains("run_dashboard_stale_error_budget_lane.sh"));
    assert!(DOC.contains("check_dashboard_stale_error_budget_policy.sh"));
    assert!(DOC.contains("run_dashboard_stale_error_budget_contract_lane.sh"));
    assert!(DOC.contains("kamn.dashboard.stale-error-budget-report.v1"));
    assert!(DOC.contains("KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS"));
    assert!(DOC.contains("KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS"));
}

#[test]
fn doc_contains_moderation_recovery_observability_hooks() {
    assert!(DOC.contains("## Moderation and Recovery Observability Hooks"));
    assert!(DOC.contains("run_reputation_signal_quarantine_contract_lane.sh"));
    assert!(DOC.contains("run_reputation_recovery_contract_lane.sh"));
    assert!(DOC.contains("ingestion_action"));
    assert!(DOC.contains("recovery_action"));
}

#[test]
fn doc_contains_structured_logging_live_validation_lane() {
    assert!(DOC.contains("## Structured Runtime Logging Correlation Contract"));
    assert!(DOC.contains("validate_structured_logging_live.sh"));
    assert!(DOC.contains("test_validate_structured_logging_live.sh"));
    assert!(DOC.contains("structured_logging_contract_status=verified"));
    assert!(DOC.contains("correlation_contract_status=verified"));
}

#[test]
fn doc_contains_structured_logging_contract_lane_and_policy() {
    assert!(DOC.contains("check_structured_logging_live_policy.sh"));
    assert!(DOC.contains("test_check_structured_logging_live_policy.sh"));
    assert!(DOC.contains("validate_structured_logging_live_contract_lane.sh"));
    assert!(DOC.contains("test_validate_structured_logging_live_contract_lane.sh"));
    assert!(DOC.contains("structured_logging_policy_status=verified"));
    assert!(DOC.contains("structured_logging_contract_lane_status=verified"));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1",
    ));
}

#[test]
fn roadmap_tracks_post_roadmap_wave1_structured_logging_live_validation() {
    assert!(ROADMAP.contains("Post-roadmap hardening wave 1 live validation delivered"));
    assert!(ROADMAP.contains("Task #3035, Subtask #3036"));
    assert!(ROADMAP.contains("scripts/runtime/validate_structured_logging_live.sh"));
    assert!(ROADMAP.contains("fail_closed_reason_code=invalid_log_config_level"));
}

#[test]
fn roadmap_tracks_wave6_structured_logging_contract_lane_and_policy() {
    assert!(ROADMAP.contains(
        "Post-roadmap hardening wave 6 structured logging contract-lane policy delivered"
    ));
    assert!(ROADMAP.contains("Task #4641, Subtasks #4645 and #4646"));
    assert!(ROADMAP.contains("scripts/runtime/validate_structured_logging_live_contract_lane.sh"));
    assert!(ROADMAP.contains("scripts/runtime/check_structured_logging_live_policy.sh"));
    assert!(ROADMAP.contains(
        "fail_closed_reason_code=structured_logging_policy_marker_missing:structured_logging_contract_status"
    ));
}

#[test]
fn doc_contains_runtime_observability_stream_contract_lane() {
    assert!(DOC.contains("## Runtime Endpoint Stream Contract (Issue #3047)"));
    assert!(DOC.contains("/metrics.stream"));
    assert!(DOC.contains("validate_runtime_observability_endpoint_live.sh"));
    assert!(DOC.contains("test_validate_runtime_observability_endpoint_live.sh"));
    assert!(DOC.contains("runtime_observability_stream_contract_status=verified"));
}

#[test]
fn roadmap_tracks_wave2_runtime_observability_stream_live_validation() {
    assert!(ROADMAP.contains("Task #3047, Subtask #3048"));
    assert!(ROADMAP.contains("scripts/runtime/validate_runtime_observability_endpoint_live.sh"));
    assert!(
        ROADMAP.contains("scripts/runtime/test_validate_runtime_observability_endpoint_live.sh")
    );
    assert!(ROADMAP.contains("runtime_observability_stream_contract_status=verified"));
}

#[test]
fn doc_contains_runtime_observability_stream_contract_lane_and_policy() {
    assert!(DOC.contains("validate_runtime_observability_endpoint_live_contract_lane.sh"));
    assert!(DOC.contains("check_runtime_observability_endpoint_live_policy.sh"));
    assert!(DOC.contains("test_validate_runtime_observability_endpoint_live_contract_lane.sh"));
    assert!(DOC.contains("runtime_observability_policy_status=verified"));
    assert!(DOC.contains("runtime_observability_contract_lane_status=verified"));
}

#[test]
fn roadmap_tracks_wave2_runtime_observability_stream_contract_lane_and_policy() {
    assert!(ROADMAP.contains("Task #3150, Subtask #3160"));
    assert!(ROADMAP
        .contains("scripts/runtime/validate_runtime_observability_endpoint_live_contract_lane.sh"));
    assert!(ROADMAP.contains("scripts/runtime/check_runtime_observability_endpoint_live_policy.sh"));
    assert!(ROADMAP.contains("runtime_observability_policy_status=verified"));
}

#[test]
fn regression_requires_availability_critical_rule() {
    // Regression: #206
    assert!(DOC.contains("`Availability`: critical when below minimum threshold."));
}

#[test]
fn regression_requires_frontend_stale_and_severity_projection_rules() {
    // Regression: #591
    assert!(DOC.contains("severity-critical"));
    assert!(DOC.contains("stale-data-banner"));
    assert!(DOC.contains("dashboard-error"));
    assert!(DOC.contains("dashboard-empty"));
}

#[test]
fn regression_requires_post_cutover_slo_stale_evidence_guard() {
    // Regression: #711
    assert!(DOC.contains(
        "stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`)."
    ));
}

#[test]
fn regression_requires_post_cutover_slo_shared_contract_marker() {
    // Regression: #1282
    assert!(DOC.contains(
        "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1282`)."
    ));
}

#[test]
fn regression_requires_slo_alert_schema_fail_closed_guard() {
    // Regression: #913
    assert!(DOC.contains(
        "missing or drifted alert evidence schema/keys must fail closed (`Regression: #913`)."
    ));
}

#[test]
fn regression_requires_moderation_recovery_observability_guard() {
    // Regression: #924
    assert!(DOC.contains(
        "quarantined stale/replayed signals and irreversible recovery reversals must remain visible through deterministic evidence keys (`Regression: #924`)."
    ));
}

#[test]
fn regression_requires_dashboard_stale_error_budget_fail_closed_guard() {
    // Regression: #942
    assert!(DOC.contains(
        "stale threshold drift, error-budget threshold drift, docs parity drift, or runtime budget overflow force `NO-GO` (`Regression: #942`)."
    ));
}

#[test]
fn regression_requires_dashboard_stale_error_shared_contract_marker() {
    // Regression: #1258
    assert!(DOC.contains(
        "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1258`)."
    ));
}
