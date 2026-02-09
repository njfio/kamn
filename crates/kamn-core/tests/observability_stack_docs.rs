const DOC: &str = include_str!("../../../docs/foundation/observability-slo-dashboards.md");

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
