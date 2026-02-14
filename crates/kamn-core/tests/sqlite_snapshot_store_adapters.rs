use kamn_core::{
    bootstrap, ChannelPermissionEngine, ChannelSnapshotStore, ChannelStore,
    DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle, MessageDeliveryGuards,
    MessageLifecycleSnapshotStore, MessageLifecycleStore, NodeConfig, NodeRole, RuntimeSnapshot,
    RuntimeSnapshotStore, SqliteChannelSnapshotStore, SqliteDurableGuardSnapshotStore,
    SqliteMessageLifecycleSnapshotStore, SqliteRuntimeSnapshotStore,
    SqliteTaskOperationSnapshotStore, SyncMode, TaskOperationEngine, TaskOperationSnapshotStore,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-{tag}-{unique}"))
}

#[test]
fn functional_sqlite_snapshot_adapters_roundtrip_all_store_types() {
    let root = temp_dir("sqlite-snapshot-adapters");
    fs::create_dir_all(&root).expect("fixture directory should build");
    let sqlite_path = root.join("runtime-snapshot-stores.sqlite");

    let mut channel_store = SqliteChannelSnapshotStore::new(sqlite_path.clone())
        .expect("sqlite channel snapshot store should initialize");
    let channel_snapshot = ChannelStore::new().export_snapshot();
    channel_store
        .write(channel_snapshot.clone())
        .expect("sqlite channel snapshot write should succeed");
    assert_eq!(
        channel_store
            .read_latest()
            .expect("sqlite channel snapshot read should succeed"),
        Some(channel_snapshot)
    );

    let mut message_store = SqliteMessageLifecycleSnapshotStore::new(sqlite_path.clone())
        .expect("sqlite message lifecycle snapshot store should initialize");
    let message_snapshot = MessageLifecycleStore::new().export_snapshot();
    message_store
        .write(message_snapshot.clone())
        .expect("sqlite message lifecycle snapshot write should succeed");
    assert_eq!(
        message_store
            .read_latest()
            .expect("sqlite message lifecycle snapshot read should succeed"),
        Some(message_snapshot)
    );

    let mut task_store = SqliteTaskOperationSnapshotStore::new(sqlite_path.clone())
        .expect("sqlite task-operation snapshot store should initialize");
    let task_snapshot = TaskOperationEngine::new().export_snapshot();
    task_store
        .write(task_snapshot.clone())
        .expect("sqlite task-operation snapshot write should succeed");
    assert_eq!(
        task_store
            .read_latest()
            .expect("sqlite task-operation snapshot read should succeed"),
        Some(task_snapshot)
    );

    let mut durable_store = SqliteDurableGuardSnapshotStore::new(sqlite_path.clone())
        .expect("sqlite durable-guard snapshot store should initialize");
    let durable_bundle = DurableGuardSnapshotBundle::capture(
        &MessageDeliveryGuards::new(),
        &ChannelPermissionEngine::new(),
    );
    durable_store
        .save_bundle(durable_bundle.clone())
        .expect("sqlite durable-guard bundle write should succeed");
    assert_eq!(
        durable_store
            .load_bundle()
            .expect("sqlite durable-guard bundle read should succeed"),
        Some(durable_bundle)
    );

    let mut runtime_store = SqliteRuntimeSnapshotStore::new(sqlite_path.clone())
        .expect("sqlite runtime snapshot store should initialize");
    let first = RuntimeSnapshot::new(1, "state-hash-v1").expect("first snapshot should build");
    let second =
        RuntimeSnapshot::with_cursor(2, "state-hash-v2", 2).expect("second snapshot should build");
    runtime_store
        .write(first.clone())
        .expect("sqlite runtime snapshot first write should succeed");
    runtime_store
        .write(second.clone())
        .expect("sqlite runtime snapshot second write should succeed");
    assert_eq!(
        runtime_store
            .read_latest()
            .expect("sqlite runtime snapshot read_latest should succeed"),
        Some(second)
    );
    assert_eq!(
        runtime_store
            .list()
            .expect("sqlite runtime snapshot list should succeed"),
        vec![
            first,
            RuntimeSnapshot::with_cursor(2, "state-hash-v2", 2).expect("snapshot")
        ]
    );

    let _ = fs::remove_file(sqlite_path);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn integration_bootstrap_selects_sqlite_snapshot_components() {
    let root = temp_dir("bootstrap-sqlite-components");
    fs::create_dir_all(&root).expect("fixture directory should build");
    let sqlite_path = root.join("runtime-stores.sqlite");
    let storage_dir = format!("sqlite://{}", sqlite_path.to_string_lossy());

    let config = NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role: NodeRole::Processor,
        storage_dir,
        enable_gossip: true,
        sync_mode: SyncMode::Fast,
    };
    let plan = bootstrap(config).expect("bootstrap should support sqlite storage dir");
    let components = plan.wiring.all_components();
    assert!(components.contains(&"content-storage:file-default"));
    assert!(components.contains(&"did-registry:file-default"));
    assert!(components.contains(&"task-operation-snapshot-store:sqlite-default"));
    assert!(components.contains(&"durable-guard-snapshot-store:sqlite-default"));
    assert!(components.contains(&"channel-snapshot-store:sqlite-default"));
    assert!(components.contains(&"message-lifecycle-snapshot-store:sqlite-default"));
    assert!(components.contains(&"runtime-snapshot-store:sqlite-default"));

    let _ = fs::remove_file(sqlite_path);
    let _ = fs::remove_dir_all(root);
}
