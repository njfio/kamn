use super::super::{SwarmTaskDraft, TaskOperationEngine, TaskOperationError};
use crate::TaskState;

#[test]
fn submit_rejects_duplicate_task_id() {
    let mut engine = TaskOperationEngine::new();
    assert!(engine
        .submit("task-1", "kamn:did:agent:req-1", "desc")
        .is_ok());
    assert_eq!(
        engine.submit("task-1", "kamn:did:agent:req-1", "desc"),
        Err(TaskOperationError::DuplicateTaskId("task-1".to_owned()))
    );
}

#[test]
fn cancel_allowed_for_assignee() {
    let mut engine = TaskOperationEngine::new();
    assert!(engine
        .submit("task-2", "kamn:did:agent:req-1", "desc")
        .is_ok());
    assert!(engine.accept("task-2", "kamn:did:agent:worker-1").is_ok());
    assert!(engine.cancel("task-2", "kamn:did:agent:worker-1").is_ok());
    let state = engine
        .task("task-2")
        .expect("task lookup failed")
        .lifecycle
        .state();
    assert_eq!(state, TaskState::Cancelled);
}

#[test]
fn submit_swarm_rejects_cyclic_dependencies() {
    let mut engine = TaskOperationEngine::new();
    assert!(matches!(
        engine.submit_swarm_tasks(vec![
            SwarmTaskDraft {
                task_id: "cycle-a".to_owned(),
                requester: "kamn:did:agent:req-1".to_owned(),
                description: "a".to_owned(),
                dependencies: vec!["cycle-b".to_owned()],
            },
            SwarmTaskDraft {
                task_id: "cycle-b".to_owned(),
                requester: "kamn:did:agent:req-1".to_owned(),
                description: "b".to_owned(),
                dependencies: vec!["cycle-a".to_owned()],
            },
        ]),
        Err(TaskOperationError::CyclicDependency { .. })
    ));
}

#[test]
fn start_work_rejects_unsatisfied_dependency() {
    let mut engine = dependency_block_case_engine();
    assert_eq!(
        engine.start_work("dep-child", "kamn:did:agent:worker-1"),
        Err(TaskOperationError::DependencyNotSatisfied {
            task_id: "dep-child".to_owned(),
            dependency_id: "dep-root".to_owned(),
        })
    );
}

fn dependency_block_case_engine() -> TaskOperationEngine {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit_swarm_tasks(dependency_block_case_drafts())
        .expect("swarm submission should succeed");
    accept_dependency_case_tasks(&mut engine);
    engine
}

fn dependency_block_case_drafts() -> Vec<SwarmTaskDraft> {
    vec![
        swarm_task("dep-root", "root", vec![]),
        swarm_task("dep-child", "child", vec!["dep-root".to_owned()]),
    ]
}

fn swarm_task(task_id: &str, description: &str, dependencies: Vec<String>) -> SwarmTaskDraft {
    SwarmTaskDraft {
        task_id: task_id.to_owned(),
        requester: "kamn:did:agent:req-1".to_owned(),
        description: description.to_owned(),
        dependencies,
    }
}

fn accept_dependency_case_tasks(engine: &mut TaskOperationEngine) {
    engine
        .accept("dep-root", "kamn:did:agent:worker-1")
        .expect("root accept should succeed");
    engine
        .accept("dep-child", "kamn:did:agent:worker-1")
        .expect("child accept should succeed");
}
