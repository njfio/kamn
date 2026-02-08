use kamn_core::{ChannelMetadata, ChannelModelError, ChannelStore, ChannelType};

#[test]
fn direct_channel_registers_membership_and_admins() {
    let mut store = ChannelStore::new();
    store
        .create_direct(
            "channel:direct:1",
            "kamn:did:agent:alice",
            "kamn:did:agent:bob",
        )
        .expect("direct channel should be created");

    assert_eq!(
        store
            .channel_type("channel:direct:1")
            .expect("type should exist"),
        ChannelType::Direct
    );
    assert_eq!(
        store
            .members("channel:direct:1")
            .expect("members should exist"),
        vec![
            "kamn:did:agent:alice".to_owned(),
            "kamn:did:agent:bob".to_owned()
        ]
    );
    assert_eq!(
        store
            .admins("channel:direct:1")
            .expect("admins should exist"),
        vec![
            "kamn:did:agent:alice".to_owned(),
            "kamn:did:agent:bob".to_owned()
        ]
    );
    assert_eq!(
        store.channels_for_member("kamn:did:agent:alice"),
        vec!["channel:direct:1".to_owned()]
    );
}

#[test]
fn group_channel_admin_can_invite_and_remove_members() {
    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:1",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group channel should be created");

    store
        .invite_member(
            "channel:group:1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("admin should invite member");
    assert!(store
        .is_member("channel:group:1", "kamn:did:agent:member-2")
        .expect("membership query should succeed"));

    store
        .remove_member(
            "channel:group:1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("admin should remove member");
    assert!(!store
        .is_member("channel:group:1", "kamn:did:agent:member-2")
        .expect("membership query should succeed"));
}

#[test]
fn non_admin_cannot_invite_member() {
    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:2",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group channel should be created");

    assert_eq!(
        store.invite_member(
            "channel:group:2",
            "kamn:did:agent:member-1",
            "kamn:did:agent:member-2",
        ),
        Err(ChannelModelError::UnauthorizedActor {
            actor: "kamn:did:agent:member-1".to_owned(),
            required: "admin",
        })
    );
}

#[test]
fn direct_channel_member_removal_is_rejected() {
    let mut store = ChannelStore::new();
    store
        .create_direct(
            "channel:direct:2",
            "kamn:did:agent:alice",
            "kamn:did:agent:bob",
        )
        .expect("direct channel should be created");

    assert_eq!(
        store.remove_member(
            "channel:direct:2",
            "kamn:did:agent:alice",
            "kamn:did:agent:bob",
        ),
        Err(ChannelModelError::UnsupportedOperation {
            channel_type: ChannelType::Direct,
            action: "remove_member",
        })
    );
}

#[test]
fn removing_last_admin_is_rejected() {
    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:3",
            "kamn:did:agent:owner",
            vec!["kamn:did:agent:owner".to_owned()],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group channel should be created");

    // Regression: #119
    assert_eq!(
        store.remove_admin(
            "channel:group:3",
            "kamn:did:agent:owner",
            "kamn:did:agent:owner",
        ),
        Err(ChannelModelError::LastAdminRemoval(
            "channel:group:3".to_owned()
        ))
    );
}

#[test]
fn broadcast_channel_exposes_topic_metadata() {
    let mut store = ChannelStore::new();
    store
        .create_broadcast(
            "channel:broadcast:1",
            "kamn:did:agent:owner",
            "protocol-updates",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("broadcast channel should be created");

    assert_eq!(
        store
            .channel_type("channel:broadcast:1")
            .expect("type should exist"),
        ChannelType::Broadcast
    );
    assert_eq!(
        store
            .metadata("channel:broadcast:1")
            .expect("metadata should exist"),
        ChannelMetadata::Broadcast {
            topic: "protocol-updates".to_owned(),
        }
    );
}

#[test]
fn specialized_task_and_marketplace_channels_preserve_metadata() {
    let mut store = ChannelStore::new();
    store
        .create_task_channel(
            "channel:task:1",
            "kamn:did:agent:owner",
            "task-42",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:assignee-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("task channel should be created");
    store
        .create_marketplace_channel(
            "channel:market:1",
            "kamn:did:agent:owner",
            "service-market-v1",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:buyer-1".to_owned(),
                "kamn:did:agent:seller-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("marketplace channel should be created");

    assert_eq!(
        store
            .metadata("channel:task:1")
            .expect("metadata should exist"),
        ChannelMetadata::Task {
            task_id: "task-42".to_owned(),
        }
    );
    assert_eq!(
        store
            .metadata("channel:market:1")
            .expect("metadata should exist"),
        ChannelMetadata::Marketplace {
            market_scope: "service-market-v1".to_owned(),
        }
    );
}

#[test]
fn integration_governance_channel_requires_quorum_ready_membership() {
    let mut store = ChannelStore::new();
    store
        .create_governance_channel(
            "channel:gov:1",
            "kamn:did:agent:owner",
            "core-protocol",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:validator-1".to_owned(),
                "kamn:did:agent:validator-2".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("governance channel should be created");

    assert_eq!(
        store
            .metadata("channel:gov:1")
            .expect("metadata should exist"),
        ChannelMetadata::Governance {
            proposal_scope: "core-protocol".to_owned(),
        }
    );
    assert_eq!(
        store.channels_for_member("kamn:did:agent:validator-1"),
        vec!["channel:gov:1".to_owned()]
    );
}

#[test]
fn regression_governance_channel_rejects_under_quorum_membership() {
    let mut store = ChannelStore::new();

    // Regression: #229
    assert_eq!(
        store.create_governance_channel(
            "channel:gov:2",
            "kamn:did:agent:owner",
            "core-protocol",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:validator-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        ),
        Err(ChannelModelError::InsufficientMembers {
            channel_type: ChannelType::Governance,
            minimum: 3,
            actual: 2,
        })
    );
}
