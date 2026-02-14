use kamn_core::{
    migrate_file_snapshots_to_sqlite_parity, ChannelPermissionEngine, ChannelSnapshotStore,
    ChannelStore, DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle,
    FileChannelSnapshotStore, FileDurableGuardSnapshotStore, FileMessageLifecycleSnapshotStore,
    FileRuntimeSnapshotStore, FileTaskOperationSnapshotStore, MessageDeliveryGuards,
    MessageLifecycleSnapshotStore, MessageLifecycleStore, RuntimeSnapshot, RuntimeSnapshotStore,
    SnapshotMigrationError, SnapshotMigrationParityReport, SqliteChannelSnapshotStore,
    SqliteDurableGuardSnapshotStore, SqliteMessageLifecycleSnapshotStore,
    SqliteRuntimeSnapshotStore, SqliteTaskOperationSnapshotStore, TaskOperationEngine,
    TaskOperationSnapshotStore,
};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn temp_dir(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-{tag}-{unique}"))
}

#[test]
fn functional_migration_corpus_replays_file_snapshots_into_sqlite() {
    let root = temp_dir("file-sqlite-migration-corpus");
    fs::create_dir_all(&root).expect("fixture directory should build");
    let sqlite_path = root.join("migration-parity.sqlite");

    let mut channel_file_store =
        FileChannelSnapshotStore::new(root.join("channel.snapshot")).expect("channel file store");
    let channel_snapshot = ChannelStore::new().export_snapshot();
    channel_file_store
        .write(channel_snapshot.clone())
        .expect("channel snapshot should persist");

    let mut message_file_store =
        FileMessageLifecycleSnapshotStore::new(root.join("message-lifecycle.snapshot"))
            .expect("message file store");
    let message_snapshot = MessageLifecycleStore::new().export_snapshot();
    message_file_store
        .write(message_snapshot.clone())
        .expect("message snapshot should persist");

    let mut task_file_store =
        FileTaskOperationSnapshotStore::new(root.join("task-operation.snapshot"))
            .expect("task file store");
    let task_snapshot = TaskOperationEngine::new().export_snapshot();
    task_file_store
        .write(task_snapshot.clone())
        .expect("task snapshot should persist");

    let mut durable_file_store =
        FileDurableGuardSnapshotStore::new(root.join("durable-guard.snapshot"))
            .expect("durable file store");
    let durable_bundle = DurableGuardSnapshotBundle::capture(
        &MessageDeliveryGuards::new(),
        &ChannelPermissionEngine::new(),
    );
    durable_file_store
        .save_bundle(durable_bundle.clone())
        .expect("durable bundle should persist");

    let mut runtime_file_store = FileRuntimeSnapshotStore::new(root.join("runtime.snapshot"))
        .expect("runtime file store should initialize");
    let runtime_first = RuntimeSnapshot::new(1, "runtime-state-v1").expect("runtime snapshot");
    let runtime_second =
        RuntimeSnapshot::with_cursor(2, "runtime-state-v2", 2).expect("runtime snapshot");
    runtime_file_store
        .write(runtime_first.clone())
        .expect("runtime first snapshot should persist");
    runtime_file_store
        .write(runtime_second.clone())
        .expect("runtime second snapshot should persist");

    let report = migrate_file_snapshots_to_sqlite_parity(root.as_path(), sqlite_path.as_path())
        .expect("migration parity should pass");
    assert_eq!(
        report,
        SnapshotMigrationParityReport {
            reason_code: "snapshot_migration_parity_pass",
            migrated_domains: vec![
                "channel-snapshot-store",
                "message-lifecycle-snapshot-store",
                "task-operation-snapshot-store",
                "durable-guard-snapshot-store",
                "runtime-snapshot-store",
            ],
            migrated_snapshot_count: 6,
        }
    );

    let channel_sqlite_store =
        SqliteChannelSnapshotStore::new(sqlite_path.clone()).expect("sqlite channel");
    assert_eq!(
        channel_sqlite_store
            .read_latest()
            .expect("sqlite channel read should pass"),
        Some(channel_snapshot)
    );

    let message_sqlite_store =
        SqliteMessageLifecycleSnapshotStore::new(sqlite_path.clone()).expect("sqlite message");
    assert_eq!(
        message_sqlite_store
            .read_latest()
            .expect("sqlite message read should pass"),
        Some(message_snapshot)
    );

    let task_sqlite_store =
        SqliteTaskOperationSnapshotStore::new(sqlite_path.clone()).expect("sqlite task");
    assert_eq!(
        task_sqlite_store
            .read_latest()
            .expect("sqlite task read should pass"),
        Some(task_snapshot)
    );

    let durable_sqlite_store =
        SqliteDurableGuardSnapshotStore::new(sqlite_path.clone()).expect("sqlite durable");
    assert_eq!(
        durable_sqlite_store
            .load_bundle()
            .expect("sqlite durable read should pass"),
        Some(durable_bundle)
    );

    let runtime_sqlite_store =
        SqliteRuntimeSnapshotStore::new(sqlite_path.clone()).expect("sqlite runtime");
    assert_eq!(
        runtime_sqlite_store
            .list()
            .expect("sqlite runtime list should pass"),
        vec![runtime_first, runtime_second]
    );
}

#[test]
fn integration_migration_checker_fails_closed_on_corrupt_legacy_payload() {
    let root = temp_dir("file-sqlite-migration-corrupt");
    fs::create_dir_all(&root).expect("fixture directory should build");
    fs::write(root.join("channel.snapshot"), "schema|1\nbroken\n")
        .expect("corrupt fixture should persist");
    let sqlite_path = root.join("migration-corrupt.sqlite");

    let error = migrate_file_snapshots_to_sqlite_parity(root.as_path(), sqlite_path.as_path())
        .expect_err("corrupt fixture must fail closed");
    match error {
        SnapshotMigrationError::LegacyStoreLoad {
            domain,
            reason_code,
            ..
        } => {
            assert_eq!(domain, "channel-snapshot-store");
            assert_eq!(reason_code, "snapshot_migration_legacy_store_load_failed");
        }
        other => panic!("unexpected migration error variant: {other:?}"),
    }
}

#[test]
fn performance_migration_corpus_replay_stays_within_local_budget() {
    let root = temp_dir("file-sqlite-migration-budget");
    fs::create_dir_all(&root).expect("fixture directory should build");
    let sqlite_path = root.join("migration-budget.sqlite");

    let mut runtime_file_store = FileRuntimeSnapshotStore::new(root.join("runtime.snapshot"))
        .expect("runtime file store should initialize");
    for version in 1..=64_u64 {
        let state_hash = format!("runtime-state-{version}");
        let snapshot =
            RuntimeSnapshot::with_cursor(version, state_hash.as_str(), version).expect("snapshot");
        runtime_file_store
            .write(snapshot)
            .expect("runtime snapshot should persist");
    }

    let started = Instant::now();
    let report = migrate_file_snapshots_to_sqlite_parity(root.as_path(), sqlite_path.as_path())
        .expect("migration parity should pass");
    let elapsed = started.elapsed();
    assert_eq!(report.reason_code, "snapshot_migration_parity_pass");
    assert!(
        elapsed <= Duration::from_secs(2),
        "migration corpus replay should remain bounded (elapsed={elapsed:?})"
    );
}
