use kamn_core::{
    ContentAvailabilityHealth, ContentRepairReason, ContentReplicationError,
    ContentReplicationManager, ContentReplicationPolicy, ContentStorageAdapter,
    InMemoryContentAdapter,
};

#[test]
fn content_replication_policy_rejects_invalid_thresholds() {
    assert_eq!(
        ContentReplicationPolicy::new(0, 2, 2),
        Err(ContentReplicationError::InvalidPolicy("minimum_replicas"))
    );
    assert_eq!(
        ContentReplicationPolicy::new(2, 1, 2),
        Err(ContentReplicationError::InvalidPolicy(
            "target_replicas must be >= minimum_replicas"
        ))
    );
    assert_eq!(
        ContentReplicationPolicy::new(1, 2, 0),
        Err(ContentReplicationError::InvalidPolicy(
            "max_repair_attempts"
        ))
    );
}

#[test]
fn content_replication_detects_degraded_availability_and_surfaces_alerts() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/octet-stream", b"artifact-1")
        .expect("put should succeed");

    let policy = ContentReplicationPolicy::new(2, 3, 2).expect("policy should be valid");
    let mut manager = ContentReplicationManager::new(policy);
    let snapshot = manager
        .register_content(&adapter, &head.cid, &["node-a"], 1_716_000_100)
        .expect("register should succeed");
    assert_eq!(snapshot.health, ContentAvailabilityHealth::Degraded);

    let alerts = manager.availability_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].cid, head.cid);
    assert_eq!(alerts[0].health, ContentAvailabilityHealth::Degraded);
}

#[test]
fn content_replication_repair_flow_is_idempotent_until_resolution() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/octet-stream", b"artifact-2")
        .expect("put should succeed");

    let policy = ContentReplicationPolicy::new(1, 3, 3).expect("policy should be valid");
    let mut manager = ContentReplicationManager::new(policy);
    manager
        .register_content(&adapter, &head.cid, &["node-a"], 1_716_000_200)
        .expect("register should succeed");

    let first_plan = manager.plan_repairs();
    assert_eq!(first_plan.len(), 1);
    assert_eq!(first_plan[0].missing_replicas, 2);
    assert_eq!(first_plan[0].reason, ContentRepairReason::UnderReplicated);

    let second_plan = manager.plan_repairs();
    assert!(second_plan.is_empty());

    manager
        .apply_repair_success(&head.cid, "node-b", 1_716_000_201)
        .expect("repair success should apply");
    let third_plan = manager.plan_repairs();
    assert_eq!(third_plan.len(), 1);
    assert_eq!(third_plan[0].missing_replicas, 1);
}

#[test]
fn content_replication_integration_validates_storage_integrity_before_tracking() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/octet-stream", b"artifact-3")
        .expect("put should succeed");
    adapter
        .replace_payload_unchecked(&head.cid, b"tampered".to_vec())
        .expect("tamper should succeed");

    let policy = ContentReplicationPolicy::new(1, 2, 2).expect("policy should be valid");
    let mut manager = ContentReplicationManager::new(policy);
    let result = manager.register_content(&adapter, &head.cid, &["node-a"], 1_716_000_300);
    assert!(matches!(result, Err(ContentReplicationError::Storage(_))));
}

#[test]
fn content_replication_regression_repair_retries_do_not_duplicate_or_corrupt_content() {
    // Regression: #167
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/octet-stream", b"artifact-4")
        .expect("put should succeed");

    let policy = ContentReplicationPolicy::new(1, 2, 2).expect("policy should be valid");
    let mut manager = ContentReplicationManager::new(policy);
    manager
        .register_content(&adapter, &head.cid, &["node-a"], 1_716_000_400)
        .expect("register should succeed");

    let first = manager.plan_repairs();
    assert_eq!(first.len(), 1);
    assert_eq!(manager.plan_repairs().len(), 0);

    manager
        .apply_repair_failure(&head.cid, 1_716_000_401)
        .expect("first failure should be recorded");
    assert_eq!(manager.plan_repairs().len(), 1);

    manager
        .apply_repair_failure(&head.cid, 1_716_000_402)
        .expect("second failure should be recorded");
    let blocked = manager.plan_repairs();
    assert!(blocked.is_empty());

    let stored = adapter
        .get(&head.cid)
        .expect("payload should remain retrievable");
    assert_eq!(stored.payload, b"artifact-4");
}
