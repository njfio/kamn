const DOC: &str = include_str!("../../../docs/foundation/runtime-network.md");

#[test]
fn doc_contains_runtime_network_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("PeerLifecycle"));
    assert!(DOC.contains("BoundedRuntimeQueue<T>"));
    assert!(DOC.contains("RuntimeLifecycleError"));
}

#[test]
fn doc_contains_peer_lifecycle_and_queue_rules() {
    assert!(DOC.contains("## Peer Lifecycle Rules"));
    assert!(DOC.contains("## Queue Guard Rules"));
    assert!(DOC.contains("## Scheduler Determinism Rules"));
    assert!(DOC.contains("## Recovery and Rejoin Guard Rules"));
    assert!(DOC.contains("Overflow does not evict existing entries"));
    assert!(DOC.contains("Empty peer IDs are rejected"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core runtime::tests::"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_rejoin_and_overflow_rejection_rules() {
    // Regression: #324
    assert!(DOC.contains("rejoin without disconnect is rejected"));
    assert!(DOC.contains("queue overflow rejects new event"));
    assert!(DOC.contains("duplicate candidate ID is rejected"));
    assert!(DOC.contains("stale state hash is rejected"));
    assert!(DOC.contains("rejoin replay token is rejected"));
    assert!(DOC.contains("rejoin state hash mismatch is rejected"));
}
