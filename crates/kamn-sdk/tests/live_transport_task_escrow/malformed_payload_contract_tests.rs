use crate::support::{
    accept_task_request, create_task_request, did, ensure_live_test_env, expire_artifact_request,
    live_artifact, live_client, live_task, spawn_expected_server, submit_artifact_request,
    ExpectedRequest,
};
use kamn_sdk::{KamnAgent, SdkError};

#[test]
fn regression_live_transport_artifact_status_rejects_malformed_service_payload() {
    assert_content_route_failure(malformed_content_status_request(), |client, artifact_id| {
        client.get_artifact_status(artifact_id)
    });
}

#[test]
fn regression_live_transport_task_status_rejects_malformed_service_payload() {
    ensure_live_test_env();
    let (bind_addr, server) =
        spawn_expected_server(vec![create_task_request(), malformed_task_status_request()]);

    let mut client = live_client(bind_addr.as_str());
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    assert_eq!(
        client.get_task_status(&task_id),
        Err(SdkError::TransportFailure(
            "service response missing required field"
        ))
    );

    assert!(server.join().expect("server thread should join").is_ok());
}

#[test]
fn regression_live_transport_artifact_expire_rejects_malformed_service_payload() {
    assert_content_route_failure(malformed_expire_request(), |client, artifact_id| {
        client.expire_artifact(artifact_id)
    });
}

#[test]
fn regression_live_transport_artifact_tombstone_rejects_malformed_service_payload() {
    assert_content_route_failure(malformed_tombstone_request(), |client, artifact_id| {
        client.tombstone_artifact(artifact_id)
    });
}

fn assert_content_route_failure<F, T>(response: ExpectedRequest, action: F)
where
    F: Fn(&mut kamn_sdk::LiveTransportKamnClient, &kamn_sdk::ArtifactId) -> Result<T, SdkError>,
    T: std::fmt::Debug,
{
    ensure_live_test_env();
    let (bind_addr, server) = spawn_expected_server(content_setup_requests(response));

    let mut client = live_client(bind_addr.as_str());
    let artifact_id = setup_live_artifact(&mut client);
    assert_missing_field_failure(action(&mut client, &artifact_id));

    assert!(server.join().expect("server thread should join").is_ok());
}

fn setup_live_artifact(client: &mut kamn_sdk::LiveTransportKamnClient) -> kamn_sdk::ArtifactId {
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    client
        .accept_task(&task_id, &did("assignee-live"))
        .expect("accept_task should succeed");
    client
        .submit_artifact(&task_id, live_artifact())
        .expect("submit_artifact should succeed")
}

fn assert_missing_field_failure<T: std::fmt::Debug>(result: Result<T, SdkError>) {
    match result {
        Err(SdkError::TransportFailure(message)) => {
            assert_eq!(message, "service response missing required field");
        }
        other => panic!("expected malformed payload failure, got {other:?}"),
    }
}

fn content_setup_requests(response: ExpectedRequest) -> Vec<ExpectedRequest> {
    vec![
        create_task_request(),
        accept_task_request(),
        submit_artifact_request(),
        response,
    ]
}

fn malformed_content_status_request() -> ExpectedRequest {
    ExpectedRequest {
        method: "GET",
        path: "/v1/content/content-local-artifact-abc".to_owned(),
        body: String::new(),
        sender_did: "kamn:did:agent:live-requester".to_owned(),
        scope: "content:read",
        response_status: 200,
        response_body: r#"{"content_id":"content-local-artifact-abc","redaction_status":"none"}"#
            .to_owned(),
    }
}

fn malformed_task_status_request() -> ExpectedRequest {
    ExpectedRequest {
        method: "GET",
        path: "/v1/tasks/task-local-abc".to_owned(),
        body: String::new(),
        sender_did: "kamn:did:agent:live-requester".to_owned(),
        scope: "tasks:read",
        response_status: 200,
        response_body: r#"{"task_id":"task-local-abc"}"#.to_owned(),
    }
}

fn malformed_expire_request() -> ExpectedRequest {
    ExpectedRequest {
        response_body: r#"{"content_id":"content-local-artifact-abc","redaction_status":"none"}"#
            .to_owned(),
        ..expire_artifact_request()
    }
}

fn malformed_tombstone_request() -> ExpectedRequest {
    ExpectedRequest {
        response_body:
            r#"{"content_id":"content-local-artifact-abc","redaction_status":"redacted"}"#
                .to_owned(),
        ..crate::support::tombstone_artifact_request()
    }
}
