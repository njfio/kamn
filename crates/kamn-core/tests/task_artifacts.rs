use kamn_core::{
    TaskArtifactError, TaskArtifactRecord, TaskArtifactRegistry, TaskArtifactSubmission,
};

#[test]
fn register_artifact_persists_integrity_and_provenance() {
    let mut registry = TaskArtifactRegistry::new();
    let hash = TaskArtifactRegistry::integrity_fingerprint(
        "task-1",
        "kamn:did:agent:builder-1",
        "ipfs://bafy-artifact-1",
    );

    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-1".to_owned(),
            task_id: "task-1".to_owned(),
            creator: "kamn:did:agent:builder-1".to_owned(),
            created_at_unix: 1_738_000_000,
            on_chain_hash: hash.clone(),
            off_chain_uri: "ipfs://bafy-artifact-1".to_owned(),
            content_type: "application/json".to_owned(),
        })
        .expect("artifact should register");

    assert_eq!(
        registry
            .artifact("artifact-1")
            .expect("artifact should exist"),
        &TaskArtifactRecord {
            artifact_id: "artifact-1".to_owned(),
            task_id: "task-1".to_owned(),
            creator: "kamn:did:agent:builder-1".to_owned(),
            created_at_unix: 1_738_000_000,
            on_chain_hash: hash,
            off_chain_uri: "ipfs://bafy-artifact-1".to_owned(),
            content_type: "application/json".to_owned(),
        }
    );
}

#[test]
fn duplicate_artifact_id_is_rejected() {
    let mut registry = TaskArtifactRegistry::new();
    let hash = TaskArtifactRegistry::integrity_fingerprint(
        "task-2",
        "kamn:did:agent:builder-2",
        "https://example.com/artifacts/2",
    );

    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-2".to_owned(),
            task_id: "task-2".to_owned(),
            creator: "kamn:did:agent:builder-2".to_owned(),
            created_at_unix: 1_738_000_010,
            on_chain_hash: hash.clone(),
            off_chain_uri: "https://example.com/artifacts/2".to_owned(),
            content_type: "text/plain".to_owned(),
        })
        .expect("artifact should register");

    assert_eq!(
        registry.register(TaskArtifactSubmission {
            artifact_id: "artifact-2".to_owned(),
            task_id: "task-2".to_owned(),
            creator: "kamn:did:agent:builder-2".to_owned(),
            created_at_unix: 1_738_000_011,
            on_chain_hash: hash,
            off_chain_uri: "https://example.com/artifacts/2".to_owned(),
            content_type: "text/plain".to_owned(),
        }),
        Err(TaskArtifactError::DuplicateArtifactId(
            "artifact-2".to_owned()
        ))
    );
}

#[test]
fn integration_indexes_by_task_and_creator() {
    let mut registry = TaskArtifactRegistry::new();

    let hash_a = TaskArtifactRegistry::integrity_fingerprint(
        "task-3",
        "kamn:did:agent:creator-a",
        "https://example.com/a",
    );
    let hash_b = TaskArtifactRegistry::integrity_fingerprint(
        "task-3",
        "kamn:did:agent:creator-b",
        "https://example.com/b",
    );

    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-a".to_owned(),
            task_id: "task-3".to_owned(),
            creator: "kamn:did:agent:creator-a".to_owned(),
            created_at_unix: 1_738_000_020,
            on_chain_hash: hash_a,
            off_chain_uri: "https://example.com/a".to_owned(),
            content_type: "application/pdf".to_owned(),
        })
        .expect("artifact a should register");
    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-b".to_owned(),
            task_id: "task-3".to_owned(),
            creator: "kamn:did:agent:creator-b".to_owned(),
            created_at_unix: 1_738_000_021,
            on_chain_hash: hash_b,
            off_chain_uri: "https://example.com/b".to_owned(),
            content_type: "application/pdf".to_owned(),
        })
        .expect("artifact b should register");

    assert_eq!(
        registry.artifacts_for_task("task-3"),
        vec!["artifact-a".to_owned(), "artifact-b".to_owned()]
    );
    assert_eq!(
        registry.artifacts_for_creator("kamn:did:agent:creator-b"),
        vec!["artifact-b".to_owned()]
    );
}

#[test]
fn regression_tampered_integrity_hash_is_rejected() {
    let mut registry = TaskArtifactRegistry::new();
    let expected = TaskArtifactRegistry::integrity_fingerprint(
        "task-4",
        "kamn:did:agent:builder-4",
        "ipfs://bafy-artifact-4",
    );

    // Regression: #225
    assert_eq!(
        registry.register(TaskArtifactSubmission {
            artifact_id: "artifact-4".to_owned(),
            task_id: "task-4".to_owned(),
            creator: "kamn:did:agent:builder-4".to_owned(),
            created_at_unix: 1_738_000_030,
            on_chain_hash: "deadbeef".to_owned(),
            off_chain_uri: "ipfs://bafy-artifact-4".to_owned(),
            content_type: "application/octet-stream".to_owned(),
        }),
        Err(TaskArtifactError::IntegrityMismatch {
            expected,
            provided: "deadbeef".to_owned(),
        })
    );
}
