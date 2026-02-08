const DOC: &str = include_str!("../../../docs/foundation/content-retrieval-access-cache.md");

#[test]
fn doc_contains_retrieval_scope_and_engine_contract() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ContentRetrievalConfig"));
    assert!(DOC.contains("ContentRetrievalRequest"));
    assert!(DOC.contains("ContentRetrievalEngine"));
}

#[test]
fn doc_contains_authorization_and_cache_binding_rules() {
    assert!(DOC.contains("## Authorization and Cache Rules"));
    assert!(DOC.contains("grant_task_read"));
    assert!(DOC.contains("ChannelPermissionEngine"));
    assert!(DOC.contains("cache key binds `requester + scope + cid`."));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test content_retrieval_access_cache"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_scope_bound_cache_rule() {
    // Regression: #165
    assert!(DOC
        .contains("Cache entries cannot be reused across different requester/scope combinations."));
}
