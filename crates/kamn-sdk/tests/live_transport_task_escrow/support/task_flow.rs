use crate::support::{deterministic_u64_tag, did, live_artifact, live_escrow, live_task};
use kamn_sdk::{
    ArtifactId, ArtifactStatus, EscrowId, KamnAgent, LiveTransportKamnClient, SdkError, TaskId,
};

pub(crate) fn assert_task_flow(client: &mut LiveTransportKamnClient) {
    let task_id = create_and_accept_task(client);
    let artifact_id = submit_and_verify_artifact(client, &task_id);
    verify_expire_roundtrip(client, &artifact_id);
    verify_tombstone_roundtrip(client, &artifact_id);
    client
        .complete_task(&task_id)
        .expect("complete_task should succeed");
    assert_task_status_state(client, &task_id, "completed");
}

pub(crate) fn assert_escrow_flow(client: &mut LiveTransportKamnClient) -> EscrowId {
    let escrow_id = client
        .create_escrow(live_escrow())
        .expect("create_escrow should succeed");
    assert_eq!(
        escrow_id,
        EscrowId(deterministic_u64_tag("escrow-local-xyz"))
    );
    escrow_id
}

pub(crate) fn assert_unknown_task_aliases(client: &mut LiveTransportKamnClient) {
    assert_task_not_found(client.accept_task(&TaskId(77), &did("assignee-live")));
    assert_task_not_found(client.get_task_status(&TaskId(77)));
    assert_task_not_found(client.complete_task(&TaskId(77)));
    assert_task_not_found(client.submit_artifact(&TaskId(77), live_artifact()));
    assert_artifact_not_found(client.get_artifact_status(&ArtifactId(77)));
    assert_artifact_not_found(client.expire_artifact(&ArtifactId(77)));
    assert_artifact_not_found(client.tombstone_artifact(&ArtifactId(77)));
}

pub(crate) fn assert_unknown_escrow_alias(client: &mut LiveTransportKamnClient) {
    assert_eq!(
        client.release_escrow(&EscrowId(88)),
        Err(SdkError::NotFound {
            entity: "escrow",
            id: "88".to_owned(),
        })
    );
}

pub(crate) fn assert_balance_route_fails_closed(client: &LiveTransportKamnClient) {
    assert_eq!(
        client.balance(&did("payer-live")),
        Err(SdkError::TransportFailure(
            "failed to connect to service endpoint"
        ))
    );
}

fn create_and_accept_task(client: &mut LiveTransportKamnClient) -> TaskId {
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    assert_eq!(task_id, TaskId(deterministic_u64_tag("task-local-abc")));
    assert_task_status_state(client, &task_id, "submitted");
    client
        .accept_task(&task_id, &did("assignee-live"))
        .expect("accept_task should succeed");
    assert_task_status_state(client, &task_id, "accepted");
    task_id
}

fn submit_and_verify_artifact(
    client: &mut LiveTransportKamnClient,
    task_id: &TaskId,
) -> ArtifactId {
    let artifact_id = client
        .submit_artifact(task_id, live_artifact())
        .expect("submit_artifact should succeed");
    assert_eq!(
        artifact_id,
        ArtifactId(deterministic_u64_tag("content-local-artifact-abc"))
    );
    let artifact_status = client
        .get_artifact_status(&artifact_id)
        .expect("get_artifact_status should succeed");
    assert_artifact_status(&artifact_status, &artifact_id, "retained", "none");
    artifact_id
}

fn verify_expire_roundtrip(client: &mut LiveTransportKamnClient, artifact_id: &ArtifactId) {
    let expired = client
        .expire_artifact(artifact_id)
        .expect("expire_artifact should succeed");
    assert_artifact_status(&expired, artifact_id, "expired", "none");
    let reread = client
        .get_artifact_status(&expired.artifact_id)
        .expect("get_artifact_status after expire should succeed");
    assert_eq!(reread, expired);
}

fn verify_tombstone_roundtrip(client: &mut LiveTransportKamnClient, artifact_id: &ArtifactId) {
    let tombstoned = client
        .tombstone_artifact(artifact_id)
        .expect("tombstone_artifact should succeed");
    assert_artifact_status(&tombstoned, artifact_id, "tombstoned", "redacted");
    let reread = client
        .get_artifact_status(&tombstoned.artifact_id)
        .expect("get_artifact_status after tombstone should succeed");
    assert_eq!(reread, tombstoned);
}

fn assert_task_status_state(
    client: &LiveTransportKamnClient,
    task_id: &TaskId,
    expected_state: &str,
) {
    assert_eq!(
        client
            .get_task_status(task_id)
            .expect("task status should succeed")
            .state,
        expected_state
    );
}

fn assert_artifact_status(
    status: &ArtifactStatus,
    artifact_id: &ArtifactId,
    lifecycle_state: &str,
    redaction_status: &str,
) {
    assert_eq!(
        status,
        &ArtifactStatus {
            artifact_id: artifact_id.clone(),
            lifecycle_state: lifecycle_state.to_owned(),
            redaction_status: redaction_status.to_owned(),
        }
    );
}

fn assert_task_not_found<T: std::fmt::Debug>(result: Result<T, SdkError>) {
    match result {
        Err(SdkError::NotFound { entity, id }) => {
            assert_eq!(entity, "task");
            assert_eq!(id, "77");
        }
        other => panic!("expected task not found error, got {other:?}"),
    }
}

fn assert_artifact_not_found<T: std::fmt::Debug>(result: Result<T, SdkError>) {
    match result {
        Err(SdkError::NotFound { entity, id }) => {
            assert_eq!(entity, "artifact");
            assert_eq!(id, "77");
        }
        other => panic!("expected artifact not found error, got {other:?}"),
    }
}
