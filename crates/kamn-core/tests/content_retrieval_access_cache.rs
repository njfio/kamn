use kamn_core::{
    ChannelAction, ChannelPermissionEngine, ChannelPermissions, ContentRetrievalConfig,
    ContentRetrievalEngine, ContentRetrievalError, ContentRetrievalOutcome,
    ContentRetrievalRequest, ContentRetrievalScope, ContentStorageAdapter, InMemoryContentAdapter,
    PermissionRule, RetentionPolicy,
};

fn member_permissions() -> ChannelPermissions {
    ChannelPermissions {
        send: PermissionRule::Members,
        read: PermissionRule::Members,
        invite: PermissionRule::Admins,
        remove: PermissionRule::Admins,
        configure: PermissionRule::Admins,
        retention: RetentionPolicy::Forever,
    }
}

#[test]
fn content_retrieval_config_rejects_zero_cache_ttl() {
    assert_eq!(
        ContentRetrievalConfig::new(0),
        Err(ContentRetrievalError::InvalidConfig("cache_ttl_secs"))
    );
}

#[test]
fn content_retrieval_allows_authorized_task_reader_and_uses_cache() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("text/plain", b"cached-payload")
        .expect("put should succeed");

    let mut engine = ContentRetrievalEngine::new(
        ContentRetrievalConfig::new(60).expect("config should be valid"),
    );
    engine
        .grant_task_read("task-100", "kamn:did:agent:reader-1")
        .expect("grant should succeed");

    let request = ContentRetrievalRequest::new(
        &head.cid,
        "kamn:did:agent:reader-1",
        ContentRetrievalScope::Task("task-100".to_owned()),
        1_716_000_500,
    )
    .expect("request should be valid");

    let first = engine
        .retrieve(&adapter, &request, None)
        .expect("first retrieval should pass");
    assert_eq!(first.payload, b"cached-payload");
    assert!(!first.from_cache);

    let second = engine
        .retrieve(&adapter, &request, None)
        .expect("second retrieval should pass");
    assert!(second.from_cache);
}

#[test]
fn content_retrieval_denied_access_is_auditable() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("text/plain", b"private-payload")
        .expect("put should succeed");

    let mut engine = ContentRetrievalEngine::new(
        ContentRetrievalConfig::new(60).expect("config should be valid"),
    );
    let request = ContentRetrievalRequest::new(
        &head.cid,
        "kamn:did:agent:reader-2",
        ContentRetrievalScope::Task("task-200".to_owned()),
        1_716_000_600,
    )
    .expect("request should be valid");

    let result = engine.retrieve(&adapter, &request, None);
    assert!(matches!(
        result,
        Err(ContentRetrievalError::Unauthorized { .. })
    ));

    let audits = engine.audit_events();
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].outcome, ContentRetrievalOutcome::Denied);
}

#[test]
fn content_retrieval_integration_enforces_channel_read_permissions() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/json", br#"{"ok":true}"#)
        .expect("put should succeed");

    let mut policy_engine = ChannelPermissionEngine::new();
    policy_engine
        .register_channel(
            "channel:ops:1",
            vec![
                "kamn:did:agent:member-1".to_owned(),
                "kamn:did:agent:admin-1".to_owned(),
            ],
            vec!["kamn:did:agent:admin-1".to_owned()],
            member_permissions(),
        )
        .expect("channel should register");
    policy_engine
        .authorize(
            "channel:ops:1",
            "kamn:did:agent:member-1",
            ChannelAction::Read,
        )
        .expect("member should have read permission");

    let mut engine = ContentRetrievalEngine::new(
        ContentRetrievalConfig::new(45).expect("config should be valid"),
    );
    let request = ContentRetrievalRequest::new(
        &head.cid,
        "kamn:did:agent:member-1",
        ContentRetrievalScope::Channel("channel:ops:1".to_owned()),
        1_716_000_700,
    )
    .expect("request should be valid");

    let result = engine
        .retrieve(&adapter, &request, Some(&policy_engine))
        .expect("authorized member retrieval should pass");
    assert_eq!(result.media_type, "application/json");
}

#[test]
fn content_retrieval_regression_cache_does_not_bypass_authorization() {
    // Regression: #165
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("text/plain", b"scope-sensitive")
        .expect("put should succeed");

    let mut engine = ContentRetrievalEngine::new(
        ContentRetrievalConfig::new(120).expect("config should be valid"),
    );
    engine
        .grant_task_read("task-300", "kamn:did:agent:reader-ok")
        .expect("grant should succeed");

    let allowed = ContentRetrievalRequest::new(
        &head.cid,
        "kamn:did:agent:reader-ok",
        ContentRetrievalScope::Task("task-300".to_owned()),
        1_716_000_800,
    )
    .expect("request should be valid");
    engine
        .retrieve(&adapter, &allowed, None)
        .expect("authorized reader should warm cache");

    let denied = ContentRetrievalRequest::new(
        &head.cid,
        "kamn:did:agent:reader-nope",
        ContentRetrievalScope::Task("task-300".to_owned()),
        1_716_000_801,
    )
    .expect("request should be valid");
    assert!(matches!(
        engine.retrieve(&adapter, &denied, None),
        Err(ContentRetrievalError::Unauthorized { .. })
    ));
}
