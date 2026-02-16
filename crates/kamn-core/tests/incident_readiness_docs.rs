const DOC: &str = include_str!("../../../docs/ops/incident-readiness.md");

#[test]
fn incident_readiness_doc_contains_gonogo_bundle_schema_convergence_gate() {
    assert!(DOC.contains(
        "## Go/No-Go Incident Readiness Bundle Convergence Gate (Issue #4470)"
    ));
    assert!(DOC.contains("--incident-readiness-report-file"));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.release.gonogo-incident-readiness-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("reason_codes_csv=none|<csv>"));
}

#[test]
fn incident_readiness_doc_contains_mismatch_tamper_failure_cases() {
    assert!(DOC.contains("Mismatch and tamper failure cases"));
    assert!(DOC.contains("gonogo_incident_readiness_reason_taxonomy_schema_mismatch"));
    assert!(DOC.contains("gonogo_incident_readiness_freshness_window_exceeded"));
    assert!(DOC.contains("incident readiness gate convergence mismatch"));
    assert!(DOC.contains("Regression: #4469"));
}
