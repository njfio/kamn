use kamn_core::{TaskOperationEngine, TaskOperationError, TaskOperationNoticeKind, TaskState};

#[test]
fn submit_creates_task_and_emits_notice() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit(
            "task-op-1",
            "kamn:did:agent:requester-1",
            "Collect requirements",
        )
        .expect("submit should succeed");

    let task = engine.task("task-op-1").expect("task should exist");
    assert_eq!(task.lifecycle.state(), TaskState::Submitted);
    assert_eq!(task.requester, "kamn:did:agent:requester-1");
    assert_eq!(
        engine.notices("task-op-1"),
        vec![TaskOperationNoticeKind::Submitted]
    );
}

#[test]
fn accept_then_delegate_updates_assignee_and_state() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit(
            "task-op-2",
            "kamn:did:agent:requester-1",
            "Draft architecture",
        )
        .expect("submit should succeed");
    engine
        .accept("task-op-2", "kamn:did:agent:worker-1")
        .expect("accept should succeed");
    engine
        .delegate(
            "task-op-2",
            "kamn:did:agent:worker-1",
            "kamn:did:agent:worker-2",
        )
        .expect("delegate should succeed");

    let task = engine.task("task-op-2").expect("task should exist");
    assert_eq!(task.lifecycle.state(), TaskState::Delegated);
    assert_eq!(task.assignee.as_deref(), Some("kamn:did:agent:worker-2"));
}

#[test]
fn block_and_complete_flow_is_legal_for_assignee() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit("task-op-3", "kamn:did:agent:requester-1", "Compile report")
        .expect("submit should succeed");
    engine
        .accept("task-op-3", "kamn:did:agent:worker-1")
        .expect("accept should succeed");
    engine
        .start_work("task-op-3", "kamn:did:agent:worker-1")
        .expect("start should succeed");
    engine
        .block("task-op-3", "kamn:did:agent:worker-1", "Need access token")
        .expect("block should succeed");
    engine
        .start_work("task-op-3", "kamn:did:agent:worker-1")
        .expect("restart should succeed");
    engine
        .complete("task-op-3", "kamn:did:agent:worker-1")
        .expect("complete should succeed");

    let task = engine.task("task-op-3").expect("task should exist");
    assert_eq!(task.lifecycle.state(), TaskState::Completed);
}

#[test]
fn unauthorized_actor_cannot_delegate_task() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit("task-op-4", "kamn:did:agent:requester-1", "Run tests")
        .expect("submit should succeed");
    engine
        .accept("task-op-4", "kamn:did:agent:worker-1")
        .expect("accept should succeed");

    assert_eq!(
        engine.delegate(
            "task-op-4",
            "kamn:did:agent:worker-x",
            "kamn:did:agent:worker-2",
        ),
        Err(TaskOperationError::UnauthorizedActor {
            actor: "kamn:did:agent:worker-x".to_owned(),
            required: "assignee",
        })
    );
}

#[test]
fn cancelled_task_cannot_be_accepted_again() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit("task-op-5", "kamn:did:agent:requester-1", "Write changelog")
        .expect("submit should succeed");
    engine
        .cancel("task-op-5", "kamn:did:agent:requester-1")
        .expect("cancel should succeed");

    // Regression: #129
    assert_eq!(
        engine.accept("task-op-5", "kamn:did:agent:worker-1"),
        Err(TaskOperationError::Lifecycle(
            "task is in terminal state: Cancelled".to_owned()
        ))
    );
}
