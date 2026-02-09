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
