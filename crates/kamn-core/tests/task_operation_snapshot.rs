use kamn_core::{
    EscrowLifecycle, PaymentOffer, SwarmTaskDraft, TaskOperationEngine, TaskOperationError,
    TaskPaymentWorkflow, TaskState,
};

fn build_dependency_engine() -> TaskOperationEngine {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit_swarm_tasks(vec![
            SwarmTaskDraft {
                task_id: "snapshot-root".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "Root".to_owned(),
                dependencies: vec![],
            },
            SwarmTaskDraft {
                task_id: "snapshot-child".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "Child".to_owned(),
                dependencies: vec!["snapshot-root".to_owned()],
            },
        ])
        .expect("swarm DAG should submit");
    engine
        .accept("snapshot-root", "kamn:did:agent:worker-1")
        .expect("root accept should pass");
    engine
        .accept("snapshot-child", "kamn:did:agent:worker-1")
        .expect("child accept should pass");
    engine
}

#[test]
fn task_operation_snapshot_functional_restore_resumes_dependency_gates() {
    let mut engine = build_dependency_engine();
    engine
        .start_work("snapshot-root", "kamn:did:agent:worker-1")
        .expect("root start should pass");
    engine
        .complete("snapshot-root", "kamn:did:agent:worker-1")
        .expect("root complete should pass");

    let snapshot = engine.export_snapshot();
    let mut restored = TaskOperationEngine::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
    assert_eq!(restored.ready_tasks(), vec!["snapshot-child".to_owned()]);
    restored
        .start_work("snapshot-child", "kamn:did:agent:worker-1")
        .expect("child should start after restore");
}

#[test]
fn task_operation_snapshot_rejects_schema_version_mismatch() {
    let engine = TaskOperationEngine::new();
    let mut snapshot = engine.export_snapshot();
    snapshot.schema_version = 99;

    let mut restored = TaskOperationEngine::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(TaskOperationError::SnapshotVersionMismatch {
            expected: 1,
            found: 99,
        })
    );
}

#[test]
fn task_operation_snapshot_integration_supports_payment_flow_after_restore() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit("snapshot-pay", "kamn:did:agent:requester-1", "Payable task")
        .expect("submit should pass");
    engine
        .accept("snapshot-pay", "kamn:did:agent:worker-1")
        .expect("accept should pass");
    engine
        .start_work("snapshot-pay", "kamn:did:agent:worker-1")
        .expect("start should pass");
    engine
        .complete("snapshot-pay", "kamn:did:agent:worker-1")
        .expect("complete should pass");

    let snapshot = engine.export_snapshot();
    let mut restored = TaskOperationEngine::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
    assert_eq!(
        restored
            .task("snapshot-pay")
            .expect("restored task should exist")
            .lifecycle
            .state(),
        TaskState::Completed
    );

    let escrow = EscrowLifecycle::new(10).expect("escrow should initialize");
    let mut payments = TaskPaymentWorkflow::new();
    payments
        .submit_offer(
            PaymentOffer {
                task_id: "snapshot-pay".to_owned(),
                escrow_id: "escrow-snapshot-1".to_owned(),
                payer_did: "kamn:did:agent:requester-1".to_owned(),
                payee_did: "kamn:did:agent:worker-1".to_owned(),
                amount: 10,
            },
            &restored,
            &escrow,
        )
        .expect("offer submission should pass against restored state");
}

#[test]
fn task_operation_snapshot_regression_rejects_tampered_dependency_completion_state() {
    // Regression: #502
    let mut engine = build_dependency_engine();
    engine
        .start_work("snapshot-root", "kamn:did:agent:worker-1")
        .expect("root start should pass");
    engine
        .complete("snapshot-root", "kamn:did:agent:worker-1")
        .expect("root complete should pass");
    engine
        .start_work("snapshot-child", "kamn:did:agent:worker-1")
        .expect("child start should pass");

    let mut snapshot = engine.export_snapshot();
    let root = snapshot
        .tasks
        .iter_mut()
        .find(|task| task.task_id == "snapshot-root")
        .expect("root snapshot should exist");
    root.lifecycle_history = vec![TaskState::Submitted];

    let mut restored = TaskOperationEngine::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(TaskOperationError::SnapshotDependencyNotCompleted {
            task_id: "snapshot-child".to_owned(),
            dependency_id: "snapshot-root".to_owned(),
            dependency_state: TaskState::Submitted,
        })
    );
}
