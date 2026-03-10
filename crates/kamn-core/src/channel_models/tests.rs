use super::{
    snapshot_codec::serialize_channel_snapshot, ChannelMetadata, ChannelModelError,
    ChannelRecordSnapshot, ChannelSnapshot, ChannelSnapshotError, ChannelSnapshotStore,
    ChannelSnapshotStoreError, ChannelStore, ChannelType, FileChannelSnapshotStore,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

#[test]
fn integration_file_channel_snapshot_store_roundtrips_snapshot() {
    let path = temp_channel_snapshot_path("roundtrip");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);

    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:snapshot-4",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    let snapshot = store.export_snapshot();

    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    file_store
        .write(snapshot.clone())
        .expect("write should succeed");
    assert_eq!(
        file_store.read_latest().expect("read should succeed"),
        Some(snapshot)
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn integration_file_channel_snapshot_store_replays_journal_when_snapshot_is_stale() {
    let path = temp_channel_snapshot_path("journal-replay");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);

    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:journal-1",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    let first_snapshot = store.export_snapshot();

    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    file_store
        .write(first_snapshot.clone())
        .expect("write should succeed");

    store
        .invite_member(
            "channel:group:journal-1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("invite should succeed");
    let second_snapshot = store.export_snapshot();
    file_store
        .write(second_snapshot.clone())
        .expect("second write should succeed");

    let stale_payload =
        serialize_channel_snapshot(&first_snapshot).expect("first snapshot should serialize");
    assert!(fs::write(&path, stale_payload).is_ok());
    assert_eq!(
        file_store.read_latest().expect("journal replay should win"),
        Some(second_snapshot)
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn regression_file_channel_snapshot_store_rejects_malformed_payload() {
    // Regression: #617
    let path = temp_channel_snapshot_path("malformed");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

    let file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    assert_eq!(
        file_store.read_latest(),
        Err(ChannelSnapshotStoreError::InvalidPayload(
            "record|broken".to_owned()
        ))
    );

    let _ = fs::remove_file(path);
}

#[test]
fn functional_file_channel_snapshot_store_recovery_repairs_corrupt_payload() {
    let path = temp_channel_snapshot_path("recover");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    let recovery = file_store
        .recover_latest_and_repair()
        .expect("recovery should succeed");
    assert!(recovery.latest.is_none());
    assert!(recovery.repaired);
    assert_eq!(
        fs::read_to_string(&path).expect("repaired file should be readable"),
        ""
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn regression_file_channel_snapshot_store_rejects_corrupt_journal_tail() {
    // Regression: #2690
    let path = temp_channel_snapshot_path("corrupt-journal-tail");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);

    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:journal-tail",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    let snapshot = store.export_snapshot();
    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    file_store.write(snapshot).expect("write should succeed");

    let mut journal = OpenOptions::new()
        .append(true)
        .open(&journal_path)
        .expect("journal should exist");
    assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
    assert_eq!(
        file_store.recover_latest_and_repair(),
        Err(ChannelSnapshotStoreError::InvalidPayload(
            "channel_snapshot_journal_corrupt_tail:2".to_owned()
        ))
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn performance_channel_snapshot_roundtrip_stays_within_ci_budget() {
    let mut store = ChannelStore::new();
    for index in 0..256 {
        store
            .create_group(
                &format!("channel:group:perf-{index}"),
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    format!("kamn:did:agent:member-{index}"),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
    }

    let snapshot = store.export_snapshot();
    let mut restored = ChannelStore::new();
    let start = Instant::now();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 300,
        "channel snapshot roundtrip exceeded CI budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled channel snapshot deep lane"]
fn performance_channel_snapshot_deep_lane_stress() {
    let mut store = ChannelStore::new();
    for index in 0..6000 {
        store
            .create_group(
                &format!("channel:group:deep-{index}"),
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    format!("kamn:did:agent:member-{index}"),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
    }

    let snapshot = store.export_snapshot();
    let mut restored = ChannelStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
}

fn temp_channel_snapshot_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-channel-snapshot-{tag}-{nonce}.log"))
}

fn temp_channel_snapshot_journal_path(path: &std::path::Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}
