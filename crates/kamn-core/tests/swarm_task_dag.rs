use kamn_core::{
    SwarmTaskDraft, TaskArtifactRegistry, TaskArtifactSubmission, TaskOperationEngine,
    TaskOperationError, TaskState,
};

#[test]
fn swarm_dag_rejects_cyclic_dependency_graph_submission() {
    let mut engine = TaskOperationEngine::new();

    // Regression: #472
    assert!(matches!(
        engine.submit_swarm_tasks(vec![
            SwarmTaskDraft {
                task_id: "swarm-a".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "A".to_owned(),
                dependencies: vec!["swarm-b".to_owned()],
            },
            SwarmTaskDraft {
                task_id: "swarm-b".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "B".to_owned(),
                dependencies: vec!["swarm-a".to_owned()],
            },
        ]),
        Err(TaskOperationError::CyclicDependency { .. })
    ));
}

#[test]
fn swarm_dag_functional_dependency_gates_task_start() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit_swarm_tasks(vec![
            SwarmTaskDraft {
                task_id: "swarm-root".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "Root task".to_owned(),
                dependencies: vec![],
            },
            SwarmTaskDraft {
                task_id: "swarm-child".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "Child task".to_owned(),
                dependencies: vec!["swarm-root".to_owned()],
            },
        ])
        .expect("swarm DAG should submit");

    engine
        .accept("swarm-root", "kamn:did:agent:worker-1")
        .expect("root accept should pass");
    engine
        .accept("swarm-child", "kamn:did:agent:worker-1")
        .expect("child accept should pass");

    assert_eq!(engine.ready_tasks(), vec!["swarm-root".to_owned()]);
    assert_eq!(
        engine.start_work("swarm-child", "kamn:did:agent:worker-1"),
        Err(TaskOperationError::DependencyNotSatisfied {
            task_id: "swarm-child".to_owned(),
            dependency_id: "swarm-root".to_owned(),
        })
    );

    engine
        .start_work("swarm-root", "kamn:did:agent:worker-1")
        .expect("root start should pass");
    engine
        .complete("swarm-root", "kamn:did:agent:worker-1")
        .expect("root complete should pass");

    assert_eq!(engine.ready_tasks(), vec!["swarm-child".to_owned()]);
    engine
        .start_work("swarm-child", "kamn:did:agent:worker-1")
        .expect("child start should pass once dependency is complete");
    let child = engine.task("swarm-child").expect("child task should exist");
    assert_eq!(child.lifecycle.state(), TaskState::InProgress);
}

#[test]
fn swarm_dag_integration_interops_with_task_artifacts_after_dependency_completion() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit_swarm_tasks(vec![
            SwarmTaskDraft {
                task_id: "swarm-art-root".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "Root task".to_owned(),
                dependencies: vec![],
            },
            SwarmTaskDraft {
                task_id: "swarm-art-child".to_owned(),
                requester: "kamn:did:agent:requester-1".to_owned(),
                description: "Child task".to_owned(),
                dependencies: vec!["swarm-art-root".to_owned()],
            },
        ])
        .expect("swarm DAG should submit");
    engine
        .accept("swarm-art-root", "kamn:did:agent:worker-1")
        .expect("root accept should pass");
    engine
        .accept("swarm-art-child", "kamn:did:agent:worker-1")
        .expect("child accept should pass");
    engine
        .start_work("swarm-art-root", "kamn:did:agent:worker-1")
        .expect("root start should pass");
    engine
        .complete("swarm-art-root", "kamn:did:agent:worker-1")
        .expect("root complete should pass");
    engine
        .start_work("swarm-art-child", "kamn:did:agent:worker-1")
        .expect("child start should pass");
    engine
        .complete("swarm-art-child", "kamn:did:agent:worker-1")
        .expect("child complete should pass");

    let mut artifacts = TaskArtifactRegistry::new();
    let integrity_hash = TaskArtifactRegistry::integrity_fingerprint(
        "swarm-art-child",
        "kamn:did:agent:worker-1",
        "ipfs://artifact/swarm-child",
    );
    artifacts
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-swarm-1".to_owned(),
            task_id: "swarm-art-child".to_owned(),
            creator: "kamn:did:agent:worker-1".to_owned(),
            off_chain_uri: "ipfs://artifact/swarm-child".to_owned(),
            on_chain_hash: integrity_hash,
            content_type: "application/json".to_owned(),
            created_at_unix: 1_716_700_100,
        })
        .expect("artifact registration should pass");

    assert_eq!(
        artifacts.artifacts_for_task("swarm-art-child"),
        vec!["artifact-swarm-1".to_owned()]
    );
}

#[test]
fn swarm_dag_regression_rejects_replayed_dependency_completion() {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit_swarm_tasks(vec![SwarmTaskDraft {
            task_id: "swarm-single".to_owned(),
            requester: "kamn:did:agent:requester-1".to_owned(),
            description: "Single task".to_owned(),
            dependencies: vec![],
        }])
        .expect("swarm DAG should submit");
    engine
        .accept("swarm-single", "kamn:did:agent:worker-1")
        .expect("accept should pass");
    engine
        .start_work("swarm-single", "kamn:did:agent:worker-1")
        .expect("start should pass");
    engine
        .complete("swarm-single", "kamn:did:agent:worker-1")
        .expect("first complete should pass");

    // Regression: #472
    assert_eq!(
        engine.complete("swarm-single", "kamn:did:agent:worker-1"),
        Err(TaskOperationError::Lifecycle(
            "task is in terminal state: Completed".to_owned()
        ))
    );
}
