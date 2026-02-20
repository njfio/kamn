use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    evaluate_cross_store_replay_consistency, ChannelSnapshot, ChannelSnapshotStore, ChannelStore,
    CrossStoreReplayConsistencyStatus, CrossStoreReplayDivergenceClass, FileChannelSnapshotStore,
    FileMessageLifecycleSnapshotStore, FileRuntimeSnapshotStore, FileTaskOperationSnapshotStore,
    MessageLifecycleSnapshot, MessageLifecycleSnapshotStore, MessageLifecycleStore,
    RuntimeSnapshot, RuntimeSnapshotStore, TaskOperationEngine, TaskOperationSnapshot,
    TaskOperationSnapshotStore,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn build_channel_snapshot() -> ChannelSnapshot {
    let mut store = ChannelStore::new();
    store
        .create_direct(
            "channel-alpha",
            "kamn:did:agent:sender-a",
            "kamn:did:agent:recipient-a",
        )
        .expect("channel should be valid");
    store.export_snapshot()
}

fn build_message_snapshot() -> MessageLifecycleSnapshot {
    let mut store = MessageLifecycleStore::new();
    store
        .register(
            "message-alpha",
            "kamn:did:agent:sender-a",
            vec!["kamn:did:agent:recipient-a".to_owned()],
            "2026-02-20T00:00:00Z",
            "2026-02-20T00:10:00Z",
        )
        .expect("message should register");
    store.export_snapshot()
}

fn build_task_snapshot() -> TaskOperationSnapshot {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit(
            "task-alpha",
            "kamn:did:agent:requester-a",
            "replay consistency checker fixture task",
        )
        .expect("task should submit");
    engine.export_snapshot()
}

fn temp_snapshot_path(label: &str) -> PathBuf {
    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-4013-cross-store-replay-{label}-{}-{entropy}.snapshot",
        std::process::id()
    ))
}

fn journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

fn cleanup_snapshot_files(paths: &[&Path]) {
    for path in paths {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path(path));
    }
}

#[test]
fn spec_c01_cross_store_replay_consistency_reports_consistent_when_all_snapshots_align() {
    let runtime_snapshot = RuntimeSnapshot::with_cursor(6, "state-6", 6).expect("valid runtime");
    let channel_snapshot = build_channel_snapshot();
    let message_snapshot = build_message_snapshot();
    let task_snapshot = build_task_snapshot();

    let report = evaluate_cross_store_replay_consistency(
        Some(runtime_snapshot),
        Some(channel_snapshot),
        Some(message_snapshot),
        Some(task_snapshot),
    );

    assert_eq!(
        report.status(),
        CrossStoreReplayConsistencyStatus::Consistent
    );
    assert_eq!(report.reason_code(), "none");
    assert_eq!(
        report.divergence_class(),
        CrossStoreReplayDivergenceClass::Consistent
    );
}

#[test]
fn spec_c02_cross_store_replay_consistency_flags_presence_drift_when_runtime_snapshot_missing() {
    let report = evaluate_cross_store_replay_consistency(
        None,
        Some(build_channel_snapshot()),
        Some(build_message_snapshot()),
        Some(build_task_snapshot()),
    );

    assert_eq!(
        report.status(),
        CrossStoreReplayConsistencyStatus::Divergent
    );
    assert_eq!(
        report.reason_code(),
        "cross_store_replay_divergence_runtime_snapshot_missing"
    );
    assert_eq!(
        report.divergence_class(),
        CrossStoreReplayDivergenceClass::PresenceDrift
    );
}

#[test]
fn spec_c03_cross_store_replay_consistency_flags_schema_drift_when_channel_schema_mismatch() {
    let runtime_snapshot = RuntimeSnapshot::with_cursor(6, "state-6", 6).expect("valid runtime");
    let mut channel_snapshot = build_channel_snapshot();
    channel_snapshot.schema_version += 1;

    let report = evaluate_cross_store_replay_consistency(
        Some(runtime_snapshot),
        Some(channel_snapshot),
        Some(build_message_snapshot()),
        Some(build_task_snapshot()),
    );

    assert_eq!(
        report.status(),
        CrossStoreReplayConsistencyStatus::Divergent
    );
    assert_eq!(
        report.reason_code(),
        "cross_store_replay_divergence_channel_schema_version_mismatch"
    );
    assert_eq!(
        report.divergence_class(),
        CrossStoreReplayDivergenceClass::SchemaDrift
    );
}

#[test]
fn spec_c04_cross_store_replay_consistency_flags_runtime_continuity_when_cursor_regresses_state_version(
) {
    let runtime_snapshot = RuntimeSnapshot::with_cursor(7, "state-7", 6).expect("valid runtime");
    let report = evaluate_cross_store_replay_consistency(
        Some(runtime_snapshot),
        Some(build_channel_snapshot()),
        Some(build_message_snapshot()),
        Some(build_task_snapshot()),
    );

    assert_eq!(
        report.status(),
        CrossStoreReplayConsistencyStatus::Divergent
    );
    assert_eq!(
        report.reason_code(),
        "cross_store_replay_divergence_runtime_cursor_state_version_mismatch"
    );
    assert_eq!(
        report.divergence_class(),
        CrossStoreReplayDivergenceClass::RuntimeContinuityDrift
    );
}

#[test]
fn spec_c05_cross_store_replay_consistency_flags_cardinality_drift_when_aggregate_records_exceed_cursor(
) {
    let runtime_snapshot = RuntimeSnapshot::with_cursor(2, "state-2", 2).expect("valid runtime");
    let report = evaluate_cross_store_replay_consistency(
        Some(runtime_snapshot),
        Some(build_channel_snapshot()),
        Some(build_message_snapshot()),
        Some(build_task_snapshot()),
    );

    assert_eq!(
        report.status(),
        CrossStoreReplayConsistencyStatus::Divergent
    );
    assert_eq!(
        report.reason_code(),
        "cross_store_replay_divergence_aggregate_records_exceed_runtime_cursor"
    );
    assert_eq!(
        report.divergence_class(),
        CrossStoreReplayDivergenceClass::CardinalityDrift
    );
}

#[test]
fn spec_c06_cross_store_replay_consistency_taxonomy_markers_are_stable() {
    assert_eq!(
        cross_store_replay_reason_taxonomy_version(),
        "kamn.runtime.cross-store-replay-consistency-reason-taxonomy.v1"
    );
    assert_eq!(
        cross_store_replay_reason_codes_csv(),
        "none,cross_store_replay_divergence_all_snapshots_missing,cross_store_replay_divergence_runtime_snapshot_missing,cross_store_replay_divergence_channel_snapshot_missing,cross_store_replay_divergence_message_snapshot_missing,cross_store_replay_divergence_task_snapshot_missing,cross_store_replay_divergence_channel_schema_version_mismatch,cross_store_replay_divergence_message_schema_version_mismatch,cross_store_replay_divergence_task_schema_version_mismatch,cross_store_replay_divergence_runtime_cursor_state_version_mismatch,cross_store_replay_divergence_aggregate_records_missing_for_advanced_runtime_state,cross_store_replay_divergence_aggregate_records_exceed_runtime_cursor"
    );
}

#[test]
fn spec_c07_cross_store_replay_consistency_composes_with_file_backed_snapshot_stores() {
    let channel_path = temp_snapshot_path("channel");
    let message_path = temp_snapshot_path("message");
    let task_path = temp_snapshot_path("task");
    let runtime_path = temp_snapshot_path("runtime");

    let mut channel_store =
        FileChannelSnapshotStore::new(channel_path.clone()).expect("channel store should build");
    let mut message_store = FileMessageLifecycleSnapshotStore::new(message_path.clone())
        .expect("message store should build");
    let mut task_store =
        FileTaskOperationSnapshotStore::new(task_path.clone()).expect("task store should build");
    let mut runtime_store =
        FileRuntimeSnapshotStore::new(runtime_path.clone()).expect("runtime store should build");

    channel_store
        .write(build_channel_snapshot())
        .expect("channel snapshot should persist");
    message_store
        .write(build_message_snapshot())
        .expect("message snapshot should persist");
    task_store
        .write(build_task_snapshot())
        .expect("task snapshot should persist");
    runtime_store
        .write(RuntimeSnapshot::with_cursor(6, "state-6", 6).expect("runtime should be valid"))
        .expect("runtime snapshot should persist");

    let report = evaluate_cross_store_replay_consistency(
        runtime_store
            .read_latest()
            .expect("runtime read should pass"),
        channel_store
            .read_latest()
            .expect("channel read should pass"),
        message_store
            .read_latest()
            .expect("message read should pass"),
        task_store.read_latest().expect("task read should pass"),
    );

    assert_eq!(
        report.status(),
        CrossStoreReplayConsistencyStatus::Consistent
    );
    assert_eq!(report.reason_code(), "none");
    assert_eq!(
        report.divergence_class(),
        CrossStoreReplayDivergenceClass::Consistent
    );

    cleanup_snapshot_files(&[
        channel_path.as_path(),
        message_path.as_path(),
        task_path.as_path(),
        runtime_path.as_path(),
    ]);
}

#[test]
fn regression_cross_store_replay_consistency_projection_markers_remain_stable() {
    // Regression: #4013
    let report = evaluate_cross_store_replay_consistency(
        Some(RuntimeSnapshot::with_cursor(6, "state-6", 6).expect("runtime should be valid")),
        Some(build_channel_snapshot()),
        Some(build_message_snapshot()),
        Some(build_task_snapshot()),
    );
    assert_eq!(
        report.source_marker(),
        "cross_store_replay_consistency_checker"
    );
    assert_eq!(
        report.reason_taxonomy_version(),
        cross_store_replay_reason_taxonomy_version()
    );
    assert_eq!(
        report.consistency_fingerprint(),
        "runtime:6:6|channel:1:1|message:1:1|task:1:1|aggregate:3|reason:none"
    );
}
