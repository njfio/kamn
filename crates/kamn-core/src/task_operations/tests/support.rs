use super::super::{
    snapshot_codec::serialize_task_operation_snapshot, FileTaskOperationSnapshotStore,
    TaskOperationEngine, TaskOperationSnapshot, TaskOperationSnapshotStore,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_task_operation_snapshot_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-task-operation-snapshot-{tag}-{nonce}.log"))
}

pub(super) fn temp_task_operation_snapshot_journal_path(path: &std::path::Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

pub(super) fn remove_snapshot_artifacts(path: &PathBuf, journal_path: &PathBuf) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

pub(super) fn accepted_snapshot(task_id: &str, description: &str) -> TaskOperationSnapshot {
    let mut engine = submitted_engine(task_id, description);
    engine
        .accept(task_id, "kamn:did:agent:worker-1")
        .expect("accept should succeed");
    engine.export_snapshot()
}

pub(super) fn submitted_engine(task_id: &str, description: &str) -> TaskOperationEngine {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit(task_id, "kamn:did:agent:requester-1", description)
        .expect("submit should succeed");
    engine
}

pub(super) fn write_stale_snapshot_payload(
    path: &PathBuf,
    snapshot: &TaskOperationSnapshot,
) {
    let payload = serialize_task_operation_snapshot(snapshot).expect("serialize should succeed");
    assert!(fs::write(path, payload).is_ok());
}

pub(super) fn write_corrupt_journal_tail(journal_path: &PathBuf) {
    let mut journal = OpenOptions::new()
        .append(true)
        .open(journal_path)
        .expect("journal should exist");
    assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
}

pub(super) fn write_and_read_snapshot(
    store: &mut FileTaskOperationSnapshotStore,
    snapshot: TaskOperationSnapshot,
) -> Option<TaskOperationSnapshot> {
    store.write(snapshot).expect("write should pass");
    store.read_latest().expect("read should pass")
}

pub(super) fn submit_benchmark_tasks(
    engine: &mut TaskOperationEngine,
    prefix: &str,
    count: usize,
    description: &str,
) {
    for index in 0..count {
        engine
            .submit(
                &format!("{prefix}-{index}"),
                "kamn:did:agent:requester-1",
                description,
            )
            .expect("benchmark submission should succeed");
    }
}
