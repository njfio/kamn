use kamn_core::{ChannelModelError, ChannelStore, ChannelType};

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
