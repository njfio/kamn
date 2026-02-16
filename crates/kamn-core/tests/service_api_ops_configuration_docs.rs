const DOC: &str = include_str!("../../../docs/ops/configuration.md");

#[test]
fn service_api_ops_configuration_contains_async_backpressure_failure_modes() {
    assert!(DOC.contains("## Async API Backpressure Failure Modes (Issue #4315)"));
    assert!(DOC.contains(
        "service_api_backpressure_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("service_api_ingress_concurrency_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_rate_limit_exceeded"));
    assert!(DOC.contains("fail-closed response contract"));
    assert!(DOC.contains("Regression: #4315"));
}
