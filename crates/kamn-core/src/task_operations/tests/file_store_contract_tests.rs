use super::super::{
    FileTaskOperationSnapshotStore, TaskOperationEngine, TaskOperationSnapshotStore,
    TaskOperationSnapshotStoreError,
};
use super::support::{
    accepted_task_snapshot, engine_with_submitted_task, remove_snapshot_artifacts,
    roundtrip_snapshot_store, submit_benchmark_tasks, temp_task_operation_snapshot_journal_path,
    temp_task_operation_snapshot_path, write_corrupt_journal_tail, write_stale_snapshot_payload,
};
use std::fs;

#[test]
fn integration_file_task_operation_snapshot_store_roundtrips_snapshot() {
    let path = temp_task_operation_snapshot_path("roundtrip");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    let snapshot = accepted_task_snapshot("task-store-1", "Store snapshot flow");
    let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    assert_eq!(
        roundtrip_snapshot_store(&mut file_store, snapshot.clone()),
        Some(snapshot)
    );
    remove_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn integration_file_task_operation_snapshot_store_replays_journal_when_snapshot_is_stale() {
    let path = temp_task_operation_snapshot_path("journal-replay");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    let mut engine = engine_with_submitted_task("task-journal-1", "first snapshot");
    let first_snapshot = engine.export_snapshot();
    let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    file_store.write(first_snapshot.clone()).unwrap();
    engine
        .submit(
            "task-journal-2",
            "kamn:did:agent:requester-1",
            "second snapshot",
        )
        .unwrap();
    let second_snapshot = engine.export_snapshot();
    file_store.write(second_snapshot.clone()).unwrap();
    write_stale_snapshot_payload(&path, &first_snapshot);
    assert_eq!(
        file_store.read_latest().expect("journal replay should win"),
        Some(second_snapshot)
    );
    remove_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn regression_file_task_operation_snapshot_store_rejects_malformed_payload() {
    let path = temp_task_operation_snapshot_path("malformed");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    assert!(fs::write(&path, "schema|1\ntask|broken\n").is_ok());
    let file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    assert_eq!(
        file_store.read_latest(),
        Err(TaskOperationSnapshotStoreError::InvalidPayload(
            "task|broken".to_owned()
        ))
    );
    remove_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn functional_file_task_operation_snapshot_store_recovery_repairs_corrupt_payload() {
    let path = temp_task_operation_snapshot_path("recover");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    assert!(fs::write(&path, "schema|1\ntask|broken\n").is_ok());
    let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    let recovery = file_store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert!(recovery.latest.is_none());
    assert!(recovery.repaired);
    assert_eq!(fs::read_to_string(&path).unwrap(), "");
    remove_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn regression_file_task_operation_snapshot_store_rejects_corrupt_journal_tail() {
    let path = temp_task_operation_snapshot_path("corrupt-journal-tail");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    let snapshot = engine_with_submitted_task("task-tail", "tail payload").export_snapshot();
    let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    file_store.write(snapshot).unwrap();
    write_corrupt_journal_tail(&journal_path);
    assert_eq!(
        file_store.recover_latest_and_repair(),
        Err(TaskOperationSnapshotStoreError::InvalidPayload(
            "task_operation_snapshot_journal_corrupt_tail:2".to_owned()
        ))
    );
    remove_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn performance_file_task_operation_snapshot_store_roundtrip_stays_within_ci_budget() {
    let path = temp_task_operation_snapshot_path("perf");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    let mut engine = TaskOperationEngine::new();
    submit_benchmark_tasks(
        &mut engine,
        "task-store-perf",
        256,
        "bounded snapshot benchmark",
    );
    let snapshot = engine.export_snapshot();
    let mut store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    let started = std::time::Instant::now();
    store
        .write(snapshot)
        .expect("write should stay within perf budget");
    let _ = store.read_latest().expect("read should pass");
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 250,
        "task operation snapshot store roundtrip exceeded CI budget: {elapsed_millis}ms"
    );
    remove_snapshot_artifacts(&path, &journal_path);
}

#[test]
#[ignore = "scheduled task operation snapshot deep lane"]
fn performance_task_operation_snapshot_store_deep_lane_stress() {
    let path = temp_task_operation_snapshot_path("deep");
    let journal_path = temp_task_operation_snapshot_journal_path(&path);
    remove_snapshot_artifacts(&path, &journal_path);
    let mut engine = TaskOperationEngine::new();
    submit_benchmark_tasks(
        &mut engine,
        "task-store-deep",
        6000,
        "scheduled deep lane benchmark",
    );
    let snapshot = engine.export_snapshot();
    let mut store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
    store.write(snapshot).expect("write should pass");
    let _ = store.read_latest().expect("read should pass");
    remove_snapshot_artifacts(&path, &journal_path);
}
