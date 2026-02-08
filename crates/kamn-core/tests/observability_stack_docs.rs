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
