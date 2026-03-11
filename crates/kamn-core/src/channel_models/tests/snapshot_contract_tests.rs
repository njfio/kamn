use super::super::{
    ChannelMetadata, ChannelModelError, ChannelRecordSnapshot, ChannelSnapshot,
    ChannelSnapshotError, ChannelStore, ChannelType,
};
use super::support::group_store;

#[test]
fn functional_channel_snapshot_roundtrip_restores_member_index() {
    let mut store = group_store(
        "channel:group:snapshot-1",
        "kamn:did:agent:owner",
        "kamn:did:agent:member-1",
    );
    invite_snapshot_member(&mut store);
    let restored = restore_snapshot_store(store.export_snapshot());
    assert_restored_members(&restored);
    assert_restored_member_index(&restored);
}

#[test]
fn regression_channel_snapshot_restore_rejects_duplicate_channel_ids() {
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

fn invite_snapshot_member(store: &mut ChannelStore) {
    store
        .invite_member(
            "channel:group:snapshot-1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("invite should succeed");
}

fn restore_snapshot_store(snapshot: ChannelSnapshot) -> ChannelStore {
    let mut restored = ChannelStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
    restored
}

fn assert_restored_members(restored: &ChannelStore) {
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
}

fn assert_restored_member_index(restored: &ChannelStore) {
    assert_eq!(
        restored.channels_for_member("kamn:did:agent:member-2"),
        vec!["channel:group:snapshot-1".to_owned()]
    );
}
