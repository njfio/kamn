use kamn_core::{
    ChannelAction, ChannelPermissionEngine, ChannelPermissions, ChannelPolicyError, PermissionRule,
    RetentionMessage, RetentionPolicy,
};

fn base_permissions() -> ChannelPermissions {
    ChannelPermissions {
        send: PermissionRule::Members,
        read: PermissionRule::Members,
        invite: PermissionRule::Admins,
        remove: PermissionRule::Admins,
        configure: PermissionRule::Admins,
        retention: RetentionPolicy::Forever,
    }
}

fn register_group(engine: &mut ChannelPermissionEngine, channel_id: &str) {
    engine
        .register_channel(
            channel_id,
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
                "kamn:did:agent:member-2".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
            base_permissions(),
        )
        .expect("channel should register");
}

#[test]
fn admin_and_member_permissions_are_enforced() {
    let mut engine = ChannelPermissionEngine::new();
    register_group(&mut engine, "channel:group:perm-1");

    assert!(engine
        .authorize(
            "channel:group:perm-1",
            "kamn:did:agent:member-1",
            ChannelAction::Send,
        )
        .is_ok());
    assert_eq!(
        engine.authorize(
            "channel:group:perm-1",
            "kamn:did:agent:member-1",
            ChannelAction::Invite,
        ),
        Err(ChannelPolicyError::Unauthorized {
            actor: "kamn:did:agent:member-1".to_owned(),
            action: ChannelAction::Invite,
            rule: PermissionRule::Admins,
        })
    );
    assert!(engine
        .authorize(
            "channel:group:perm-1",
            "kamn:did:agent:owner",
            ChannelAction::Invite,
        )
        .is_ok());
}

#[test]
fn allowlist_rules_restrict_actions() {
    let mut engine = ChannelPermissionEngine::new();
    let mut permissions = base_permissions();
    permissions.send =
        PermissionRule::Allowlist(["kamn:did:agent:member-1".to_owned()].into_iter().collect());
    engine
        .register_channel(
            "channel:group:perm-2",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
                "kamn:did:agent:member-2".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
            permissions,
        )
        .expect("channel should register");

    assert!(engine
        .authorize(
            "channel:group:perm-2",
            "kamn:did:agent:member-1",
            ChannelAction::Send,
        )
        .is_ok());
    assert_eq!(
        engine.authorize(
            "channel:group:perm-2",
            "kamn:did:agent:member-2",
            ChannelAction::Send,
        ),
        Err(ChannelPolicyError::Unauthorized {
            actor: "kamn:did:agent:member-2".to_owned(),
            action: ChannelAction::Send,
            rule: PermissionRule::Allowlist(
                ["kamn:did:agent:member-1".to_owned()].into_iter().collect(),
            ),
        })
    );
}

#[test]
fn retention_message_count_prunes_oldest_deterministically() {
    let mut engine = ChannelPermissionEngine::new();
    let mut permissions = base_permissions();
    permissions.retention = RetentionPolicy::MaxMessageCount(2);
    engine
        .register_channel(
            "channel:group:perm-3",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
            permissions,
        )
        .expect("channel should register");

    let candidates = engine
        .retention_candidates(
            "channel:group:perm-3",
            1000,
            vec![
                RetentionMessage {
                    id: "msg-a".to_owned(),
                    created_at_secs: 100,
                },
                RetentionMessage {
                    id: "msg-b".to_owned(),
                    created_at_secs: 200,
                },
                RetentionMessage {
                    id: "msg-c".to_owned(),
                    created_at_secs: 200,
                },
            ],
        )
        .expect("retention candidates should compute");

    assert_eq!(candidates, vec!["msg-a".to_owned()]);
}

#[test]
fn retention_max_age_prunes_expired_messages() {
    let mut engine = ChannelPermissionEngine::new();
    let mut permissions = base_permissions();
    permissions.retention = RetentionPolicy::MaxAgeSeconds(300);
    engine
        .register_channel(
            "channel:group:perm-4",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
            permissions,
        )
        .expect("channel should register");

    let candidates = engine
        .retention_candidates(
            "channel:group:perm-4",
            1000,
            vec![
                RetentionMessage {
                    id: "msg-old".to_owned(),
                    created_at_secs: 500,
                },
                RetentionMessage {
                    id: "msg-fresh".to_owned(),
                    created_at_secs: 800,
                },
            ],
        )
        .expect("retention candidates should compute");

    assert_eq!(candidates, vec!["msg-old".to_owned()]);
}

#[test]
fn zero_message_count_retention_is_rejected() {
    let mut engine = ChannelPermissionEngine::new();
    let mut permissions = base_permissions();
    permissions.retention = RetentionPolicy::MaxMessageCount(0);

    // Regression: #121
    assert_eq!(
        engine.register_channel(
            "channel:group:perm-5",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
            permissions,
        ),
        Err(ChannelPolicyError::InvalidRetentionPolicy(
            "max message count must be greater than zero".to_owned()
        ))
    );
}
