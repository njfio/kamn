const DOC: &str = include_str!("../../../docs/foundation/task-operations.md");

#[test]
fn doc_contains_task_operation_core_types_and_commands() {
    assert!(DOC.contains("## Core Types"));
    assert!(DOC.contains("TaskOperationEngine"));
    assert!(DOC.contains("SwarmTaskDraft"));
    assert!(DOC.contains("## Command Behavior"));
    assert!(DOC.contains("submit(task_id, requester, description)"));
    assert!(DOC.contains("request_input(task_id, actor, reason)"));
}

#[test]
fn doc_contains_snapshot_persistence_and_restore_contract_rules() {
    assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
    assert!(DOC.contains("export_snapshot()"));
    assert!(DOC.contains("restore_snapshot(snapshot)"));
    assert!(DOC.contains("TaskOperationSnapshotStore"));
    assert!(DOC.contains("recover_latest_and_repair()"));
    assert!(DOC.contains("TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane_commands() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --lib task_operations::tests::"));
    assert!(DOC.contains("cargo test -p kamn-core --test task_operations"));
    assert!(DOC.contains("cargo test -p kamn-core --test task_operation_snapshot"));
    assert!(DOC.contains("cargo test -p kamn-core --test task_operations_docs"));
    assert!(DOC.contains("bash scripts/task/run_task_operation_snapshot_contract_lane.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --lib task_operations::tests::performance_task_operation_snapshot_store_deep_lane_stress -- --ignored"
    ));
    assert!(DOC.contains("bash scripts/task/run_task_operation_snapshot_deep_lane.sh"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_task_snapshot_restore_guard_rules() {
    // Regression: #617
    assert!(DOC.contains("duplicate task IDs on restore are rejected (`Regression: #617`)"));
    assert!(DOC.contains("malformed snapshot payloads are rejected (`Regression: #617`)"));
    assert!(DOC.contains(
        "dependency-completion tampering remains rejected during restore (`Regression: #502`)"
    ));
}
