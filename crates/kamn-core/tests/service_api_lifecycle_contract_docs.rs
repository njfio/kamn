const DOC: &str = include_str!("../../../docs/service/api-contract.md");

#[test]
fn service_api_contract_contains_async_lifecycle_rejection_taxonomy_markers() {
    assert!(DOC.contains("## Async Lifecycle Rejection Taxonomy (Issue #4316)"));
    assert!(DOC.contains(
        "service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("service_api_ingress_concurrency_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_suspended"));
    assert!(DOC.contains("service_api_ingress_sender_duplicate_message_id"));
    assert!(DOC.contains("service_api_ingress_sender_insufficient_deposit"));
    assert!(DOC.contains("service_api_ingress_anti_spam_engine_invalid"));
    assert!(DOC.contains("## Async Lifecycle Rejection Projection Matrix"));
    assert!(DOC.contains("async-lifecycle-limiter"));
    assert!(DOC.contains("sender-admission-limiter"));
    assert!(DOC.contains("async-lifecycle-engine"));
    assert!(DOC.contains("Regression: #4316"));
}
