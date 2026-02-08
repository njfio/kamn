use kamn_core::{
    content_uri_for_cid, ContentCleanupActionKind, ContentLifecycleError, ContentLifecycleManager,
    ContentLifecycleStatus, ContentRetentionClass, ContentStorageAdapter, InMemoryContentAdapter,
    TaskArtifactRegistry, TaskArtifactSubmission,
};

#[test]
fn content_retention_profiles_define_positive_ttl_windows() {
    let short = ContentLifecycleManager::retention_profile(ContentRetentionClass::ShortLived);
    let standard = ContentLifecycleManager::retention_profile(ContentRetentionClass::Standard);
    let compliance = ContentLifecycleManager::retention_profile(ContentRetentionClass::Compliance);

    assert!(short.retain_for_secs > 0);
    assert!(short.tombstone_for_secs > 0);
    assert!(standard.retain_for_secs >= short.retain_for_secs);
    assert!(compliance.retain_for_secs >= standard.retain_for_secs);
}

#[test]
fn content_retention_transitions_active_to_expired_to_tombstoned() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/json", br#"{"v":1}"#)
        .expect("put should succeed");

    let mut manager = ContentLifecycleManager::new();
    let record = manager
        .register(&head.cid, ContentRetentionClass::ShortLived, 1_716_001_000)
        .expect("register should succeed");

    let active = manager
        .lifecycle_status(&head.cid, record.created_at_unix)
        .expect("status lookup should succeed");
    assert_eq!(active, ContentLifecycleStatus::Active);

    let expired = manager
        .lifecycle_status(&head.cid, record.expires_at_unix + 1)
        .expect("status lookup should succeed");
    assert_eq!(expired, ContentLifecycleStatus::Expired);

    manager
        .execute_cleanup(&head.cid, record.expires_at_unix + 1)
        .expect("cleanup should tombstone expired content");
    let tombstoned = manager
        .lifecycle_status(&head.cid, record.expires_at_unix + 1)
        .expect("status lookup should succeed");
    assert_eq!(tombstoned, ContentLifecycleStatus::Tombstoned);
}

#[test]
fn content_retention_cleanup_plan_is_deterministic_and_safe() {
    let mut adapter = InMemoryContentAdapter::new();
    let first = adapter
        .put("application/octet-stream", b"a")
        .expect("put should succeed");
    let second = adapter
        .put("application/octet-stream", b"b")
        .expect("put should succeed");

    let mut manager = ContentLifecycleManager::new();
    manager
        .register(
            &second.cid,
            ContentRetentionClass::ShortLived,
            1_716_002_000,
        )
        .expect("register should succeed");
    manager
        .register(&first.cid, ContentRetentionClass::ShortLived, 1_716_002_000)
        .expect("register should succeed");

    let after_expiry = 1_716_006_000;
    let plan = manager.cleanup_due(after_expiry);
    assert_eq!(plan.len(), 2);
    assert_eq!(plan[0].action, ContentCleanupActionKind::Tombstone);
    assert!(plan[0].cid < plan[1].cid);
}

#[test]
fn content_retention_integration_blocks_tombstoned_task_artifact_reference() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/pdf", b"%PDF-1.7 content")
        .expect("put should succeed");
    let uri = content_uri_for_cid(&head.cid).expect("uri should build");

    let mut registry = TaskArtifactRegistry::new();
    let on_chain_hash =
        TaskArtifactRegistry::integrity_fingerprint("task-901", "kamn:did:agent:builder-1", &uri);
    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-901".to_owned(),
            task_id: "task-901".to_owned(),
            creator: "kamn:did:agent:builder-1".to_owned(),
            created_at_unix: 1_716_002_500,
            on_chain_hash,
            off_chain_uri: uri.clone(),
            content_type: "application/pdf".to_owned(),
        })
        .expect("artifact registration should succeed");

    let mut manager = ContentLifecycleManager::new();
    manager
        .register(&head.cid, ContentRetentionClass::ShortLived, 1_716_002_500)
        .expect("register should succeed");
    manager
        .apply_tombstone(&head.cid, 1_716_003_000)
        .expect("tombstone should succeed");

    assert!(matches!(
        manager.assert_uri_accessible(&uri, 1_716_003_001),
        Err(ContentLifecycleError::Tombstoned(_))
    ));
}

#[test]
fn content_retention_regression_deleted_reference_replay_stays_blocked() {
    // Regression: #163
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/octet-stream", b"to-delete")
        .expect("put should succeed");

    let mut manager = ContentLifecycleManager::new();
    manager
        .register(&head.cid, ContentRetentionClass::ShortLived, 1_716_004_000)
        .expect("register should succeed");
    manager
        .apply_tombstone(&head.cid, 1_716_004_100)
        .expect("tombstone should succeed");

    let profile = ContentLifecycleManager::retention_profile(ContentRetentionClass::ShortLived);
    manager
        .execute_cleanup(&head.cid, 1_716_004_100 + profile.tombstone_for_secs + 1)
        .expect("cleanup should purge tombstone");

    assert!(matches!(
        manager.assert_accessible(&head.cid, 1_716_004_100 + profile.tombstone_for_secs + 2),
        Err(ContentLifecycleError::Purged(_))
    ));
}
