const OPERATIONS_DOC: &str = include_str!("../../../docs/foundation/task-operations.md");
const STATE_MACHINE_DOC: &str = include_str!("../../../docs/foundation/task-state-machine.md");

#[test]
fn docs_define_swarm_dag_command_surface() {
    assert!(OPERATIONS_DOC.contains("SwarmTaskDraft"));
    assert!(OPERATIONS_DOC.contains("submit_swarm_tasks(drafts)"));
    assert!(OPERATIONS_DOC.contains("ready_tasks()"));
    assert!(OPERATIONS_DOC.contains("DependencyNotSatisfied"));
}

#[test]
fn docs_define_dependency_aware_transition_gates() {
    assert!(STATE_MACHINE_DOC.contains("## Dependency-Aware Transition Gates"));
    assert!(STATE_MACHINE_DOC.contains("TaskOperationEngine::start_work"));
    assert!(STATE_MACHINE_DOC
        .contains("all declared dependencies must already be in `Completed` state."));
}

#[test]
fn regression_requires_cyclic_and_premature_transition_guards() {
    // Regression: #472
    assert!(OPERATIONS_DOC.contains("Regression: #472"));
    assert!(STATE_MACHINE_DOC.contains("Regression: #472"));
}

#[test]
fn docs_define_bounded_graph_benchmark_lane() {
    assert!(OPERATIONS_DOC.contains("bounded graph benchmark"));
    assert!(OPERATIONS_DOC.contains("cargo test -p kamn-core --test swarm_task_dag"));
}

#[test]
fn docs_define_snapshot_recovery_validation_rules() {
    assert!(OPERATIONS_DOC.contains("export_snapshot()"));
    assert!(OPERATIONS_DOC.contains("restore_snapshot(snapshot)"));
    assert!(OPERATIONS_DOC.contains("schema version mismatch is rejected."));
    assert!(STATE_MACHINE_DOC.contains("Snapshot restore invariants"));
}

#[test]
fn regression_requires_tampered_snapshot_rejection_rule() {
    // Regression: #502
    assert!(OPERATIONS_DOC.contains("Regression: #502"));
    assert!(STATE_MACHINE_DOC.contains("Regression: #502"));
}

#[test]
fn docs_define_snapshot_roundtrip_benchmark_lane() {
    assert!(OPERATIONS_DOC.contains("snapshot roundtrip benchmark"));
    assert!(OPERATIONS_DOC.contains("cargo test -p kamn-core --test task_operation_snapshot"));
}
