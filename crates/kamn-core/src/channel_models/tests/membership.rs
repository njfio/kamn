use super::support::*;
#[test]
fn group_creator_must_be_member() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_group(
            "channel:group:1",
            "kamn:did:agent:owner",
            vec!["kamn:did:agent:member-1".to_owned()],
            vec!["kamn:did:agent:member-1".to_owned()],
        ),
        Err(ChannelModelError::CreatorNotMember(
            "kamn:did:agent:owner".to_owned()
        ))
    );
}

#[test]
fn direct_channels_require_distinct_participants() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_direct(
            "channel:direct:1",
            "kamn:did:agent:alice",
            "kamn:did:agent:alice",
        ),
        Err(ChannelModelError::InvalidDirectParticipants)
    );
}

#[test]
fn governance_channels_require_three_members() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_governance_channel(
            "channel:gov:1",
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

#[test]
fn broadcast_metadata_requires_non_empty_topic() {
    let mut store = ChannelStore::new();
    assert_eq!(
        store.create_broadcast(
            "channel:broadcast:1",
            "kamn:did:agent:owner",
            "",
            vec!["kamn:did:agent:owner".to_owned()],
            vec!["kamn:did:agent:owner".to_owned()],
        ),
        Err(ChannelModelError::InvalidMetadata(
            "topic must not be empty".to_owned()
        ))
    );

    store
        .create_broadcast(
            "channel:broadcast:2",
            "kamn:did:agent:owner",
            "announcements",
            vec!["kamn:did:agent:owner".to_owned()],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("broadcast should be created");

    assert_eq!(
        store
            .metadata("channel:broadcast:2")
            .expect("metadata should resolve"),
        ChannelMetadata::Broadcast {
            topic: "announcements".to_owned(),
        }
    );
}

#[test]
fn functional_channel_snapshot_roundtrip_restores_member_index() {
    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:snapshot-1",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    store
        .invite_member(
            "channel:group:snapshot-1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("invite should succeed");

    let snapshot = store.export_snapshot();
    let mut restored = ChannelStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");

    assert_eq!(
        restored
            .members("channel:group:snapshot-1")
            .expect("members should exist"),
        vec![
            "kamn:did:agent:member-1".to_owned(),
            "kamn:did:agent:member-2".to_owned(),
            "kamn:did:agent:owner".to_owned(),
        ]
    );
    assert_eq!(
        restored.channels_for_member("kamn:did:agent:member-2"),
        vec!["channel:group:snapshot-1".to_owned()]
    );
}

#[test]
fn regression_channel_snapshot_restore_rejects_duplicate_channel_ids() {
    // Regression: #617
    let mut store = ChannelStore::new();
    store
        .create_direct(
            "channel:direct:snapshot-2",
            "kamn:did:agent:alice",
            "kamn:did:agent:bob",
        )
        .expect("direct should be created");

    let mut snapshot = store.export_snapshot();
    snapshot.records.push(snapshot.records[0].clone());

    let mut restored = ChannelStore::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(ChannelSnapshotError::DuplicateChannelId(
            "channel:direct:snapshot-2".to_owned()
        ))
    );
}

#[test]
fn regression_channel_snapshot_restore_rejects_admin_not_member_state() {
    // Regression: #617
    let snapshot = ChannelSnapshot {
        schema_version: 1,
        records: vec![ChannelRecordSnapshot {
            channel_id: "channel:group:snapshot-3".to_owned(),
            channel_type: ChannelType::Group,
            metadata: ChannelMetadata::Group,
            members: vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            admins: vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:ghost-admin".to_owned(),
            ],
        }],
    };

    let mut restored = ChannelStore::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(ChannelSnapshotError::Model(
            ChannelModelError::AdminNotMember("kamn:did:agent:ghost-admin".to_owned())
        ))
    );
}
