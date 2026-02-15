const DOC: &str = include_str!("../../../docs/observability/streaming.md");

#[test]
fn doc_contains_stream_payload_schema_contract() {
    assert!(DOC.contains("GET /metrics.stream"));
    assert!(DOC.contains("application/x-ndjson"));
    assert!(DOC.contains("schema_version=\"kamn.runtime.observability.stream.v1\""));
    assert!(DOC.contains("readiness_reason_code"));
}

#[test]
fn doc_contains_backpressure_and_reconnect_contract_markers() {
    assert!(DOC.contains("stream_reconnect_churn_status=verified"));
    assert!(DOC.contains("queue_bound_budget_status=verified"));
    assert!(DOC.contains("scrape_failure_taxonomy_status=verified"));
    assert!(DOC.contains(
        "scrape_failure_taxonomy_csv=readiness_failure_drill_status,stream_reconnect_churn_status,queue_bound_budget_status"
    ));
}

#[test]
fn doc_contains_low_cost_validation_lane_commands() {
    assert!(DOC.contains("validate_local_observability_scrape_live.sh --mode dry-run"));
    assert!(DOC.contains("check_local_observability_scrape_live_policy.sh"));
    assert!(DOC.contains("validate_local_observability_scrape_live_contract_lane.sh"));
}
