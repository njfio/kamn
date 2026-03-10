use super::support::*;

#[test]
fn get_artifact_status_returns_retained_status_for_known_artifact() {
    let mut client = InMemoryKamnClient::new();
    let (_, artifact_id) = prepare_task_with_artifact(&mut client);
    assert_artifact_status(&client, &artifact_id, "retained", "none");
}

#[test]
fn get_artifact_status_rejects_unknown_artifact() {
    let client = InMemoryKamnClient::new();
    assert_not_found(client.get_artifact_status(&ArtifactId(42)), "artifact", "42");
}

#[test]
fn expire_artifact_returns_expired_status_for_known_artifact() {
    let mut client = InMemoryKamnClient::new();
    let (_, artifact_id) = prepare_task_with_artifact(&mut client);
    let status = client.expire_artifact(&artifact_id).expect("artifact expire should succeed");
    assert_eq!(status.lifecycle_state, "expired");
    assert_eq!(status.redaction_status, "none");
    assert_artifact_status(&client, &artifact_id, "expired", "none");
}

#[test]
fn expire_artifact_rejects_unknown_artifact() {
    let mut client = InMemoryKamnClient::new();
    assert_not_found(client.expire_artifact(&ArtifactId(43)), "artifact", "43");
}

#[test]
fn tombstone_artifact_returns_tombstoned_status_for_known_artifact() {
    let mut client = InMemoryKamnClient::new();
    let (_, artifact_id) = prepare_task_with_artifact(&mut client);
    let status = client
        .tombstone_artifact(&artifact_id)
        .expect("artifact tombstone should succeed");
    assert_eq!(status.lifecycle_state, "tombstoned");
    assert_eq!(status.redaction_status, "redacted");
    assert_artifact_status(&client, &artifact_id, "tombstoned", "redacted");
}

#[test]
fn tombstone_artifact_rejects_unknown_artifact() {
    let mut client = InMemoryKamnClient::new();
    assert_not_found(client.tombstone_artifact(&ArtifactId(44)), "artifact", "44");
}
