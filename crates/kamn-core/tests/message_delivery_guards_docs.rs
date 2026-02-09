const DOC: &str = include_str!("../../../docs/foundation/message-delivery-guards.md");

#[test]
fn doc_contains_delivery_guard_scope_and_validation_rules() {
    assert!(DOC.contains("# Message Delivery Guards"));
    assert!(DOC.contains("MessageDeliveryGuards"));
    assert!(DOC.contains("Reject if `nonce` does not match sender expected nonce"));
}

#[test]
fn doc_contains_durable_snapshot_store_contracts() {
    assert!(DOC.contains("## Durable Snapshot Stores"));
    assert!(DOC.contains("DurableGuardSnapshotBundle::capture"));
    assert!(DOC.contains("InMemoryDurableGuardSnapshotStore"));
    assert!(DOC.contains("FileDurableGuardSnapshotStore"));
}

#[test]
fn regression_requires_corrupted_bundle_guard_marker() {
    // Regression: #679
    assert!(
        DOC.contains(
            "Truncated/corrupted durable bundle payloads fail closed (`Regression: #679`).",
        )
    );
}
