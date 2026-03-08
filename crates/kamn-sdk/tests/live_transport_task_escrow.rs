#[path = "support/live_transport_task_escrow.rs"]
mod support;

use kamn_sdk::{
    Artifact, ArtifactId, ArtifactStatus, EscrowConfig, EscrowId, KamnAgent,
    LiveTransportKamnClient, SdkError, TaskDefinition, TaskId, TokenAmount,
};
use std::thread;

use support::{
    did, ensure_live_test_env, expected_request, reserve_loopback_addr, run_contract_server,
    wait_for_server_ready, ExpectedRequest,
};

fn deterministic_u64_tag(value: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

#[test]
fn spec_c06_live_transport_task_and_escrow_routes_execute_network_contract() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server = spawn_contract_server(bind_addr.clone());
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    assert_task_flow(&mut client);
    let escrow_id = assert_escrow_flow(&mut client);
    client
        .release_escrow(&escrow_id)
        .expect("release_escrow should succeed");
    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "task/escrow route server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_unknown_task_and_escrow_aliases_fail_closed() {
    ensure_live_test_env();
    let mut client = live_client("127.0.0.1:65535");
    assert_unknown_task_aliases(&mut client);
    assert_unknown_escrow_alias(&mut client);
    assert_balance_route_fails_closed(&client);
}

fn spawn_contract_server(bind_addr: String) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || run_contract_server(bind_addr, task_and_escrow_requests()))
}

fn task_and_escrow_requests() -> Vec<ExpectedRequest> {
    vec![
        create_task_request(),
        accept_task_request(),
        submit_artifact_request(),
        get_artifact_status_request(),
        complete_task_request(),
        create_escrow_request(),
        release_escrow_request(),
    ]
}

fn create_task_request() -> ExpectedRequest {
    ExpectedRequest {
        sender_did: "kamn:did:agent:creator-live".to_owned(),
        scope: "tasks:write",
        response_status: 201,
        response_body: r#"{"task_id":"task-local-abc","state":"submitted"}"#.to_owned(),
        ..expected_request(
            "POST",
            "/v1/tasks/create",
            r#"{"creator":"kamn:did:agent:creator-live","task_type":"triage","description":"triage contract"}"#,
        )
    }
}

fn accept_task_request() -> ExpectedRequest {
    ExpectedRequest {
        sender_did: "kamn:did:agent:assignee-live".to_owned(),
        scope: "tasks:write",
        response_body: r#"{"task_id":"task-local-abc","state":"accepted"}"#.to_owned(),
        ..expected_request("POST", "/v1/tasks/task-local-abc/accept", "{}")
    }
}

fn complete_task_request() -> ExpectedRequest {
    ExpectedRequest {
        sender_did: "kamn:did:agent:assignee-live".to_owned(),
        scope: "tasks:write",
        response_body: r#"{"task_id":"task-local-abc","state":"completed"}"#.to_owned(),
        ..expected_request("POST", "/v1/tasks/task-local-abc/complete", "{}")
    }
}

fn submit_artifact_request() -> ExpectedRequest {
    ExpectedRequest {
        sender_did: "kamn:did:agent:assignee-live".to_owned(),
        scope: "content:write",
        response_status: 201,
        response_body: r#"{"content_id":"content-local-artifact-abc","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}"#.to_owned(),
        ..expected_request(
            "POST",
            "/v1/content/register",
            r#"{"task_id":"task-local-abc","artifact_name":"artifact.bin","artifact_bytes_hex":"61727469666163742d6279746573"}"#,
        )
    }
}

fn create_escrow_request() -> ExpectedRequest {
    ExpectedRequest {
        sender_did: "kamn:did:agent:payer-live".to_owned(),
        scope: "escrow:write",
        response_body: r#"{"escrow_id":"escrow-local-xyz","state":"funded"}"#.to_owned(),
        ..expected_request(
            "POST",
            "/v1/escrow/fund",
            r#"{"payer":"kamn:did:agent:payer-live","payee":"kamn:did:agent:payee-live","amount":7}"#,
        )
    }
}

fn get_artifact_status_request() -> ExpectedRequest {
    ExpectedRequest {
        method: "GET",
        path: "/v1/content/content-local-artifact-abc".to_owned(),
        body: String::new(),
        sender_did: "kamn:did:agent:live-requester".to_owned(),
        scope: "content:read",
        response_body: r#"{"content_id":"content-local-artifact-abc","lifecycle_state":"retained","redaction_status":"none"}"#.to_owned(),
        ..Default::default()
    }
}

fn release_escrow_request() -> ExpectedRequest {
    ExpectedRequest {
        sender_did: "kamn:did:agent:payer-live".to_owned(),
        scope: "escrow:write",
        response_body: r#"{"escrow_id":"escrow-local-xyz","state":"released"}"#.to_owned(),
        ..expected_request("POST", "/v1/escrow/escrow-local-xyz/release", "{}")
    }
}

fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}

fn assert_task_flow(client: &mut LiveTransportKamnClient) {
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    assert_eq!(task_id, TaskId(deterministic_u64_tag("task-local-abc")));
    client
        .accept_task(&task_id, &did("assignee-live"))
        .expect("accept_task should succeed");
    let artifact_id = client
        .submit_artifact(&task_id, live_artifact())
        .expect("submit_artifact should succeed");
    assert_eq!(
        artifact_id,
        ArtifactId(deterministic_u64_tag("content-local-artifact-abc"))
    );
    let artifact_status = client
        .get_artifact_status(&artifact_id)
        .expect("get_artifact_status should succeed");
    assert_eq!(
        artifact_status,
        ArtifactStatus {
            artifact_id,
            lifecycle_state: "retained".to_owned(),
            redaction_status: "none".to_owned(),
        }
    );
    client
        .complete_task(&task_id)
        .expect("complete_task should succeed");
}

fn live_task() -> TaskDefinition {
    TaskDefinition {
        creator: did("creator-live"),
        task_type: "triage".to_owned(),
        description: "triage contract".to_owned(),
    }
}

fn assert_escrow_flow(client: &mut LiveTransportKamnClient) -> EscrowId {
    let escrow_id = client
        .create_escrow(live_escrow())
        .expect("create_escrow should succeed");
    assert_eq!(
        escrow_id,
        EscrowId(deterministic_u64_tag("escrow-local-xyz"))
    );
    escrow_id
}

fn live_escrow() -> EscrowConfig {
    EscrowConfig {
        payer: did("payer-live"),
        payee: did("payee-live"),
        amount: TokenAmount(7),
    }
}

fn live_artifact() -> Artifact {
    Artifact {
        name: "artifact.bin".to_owned(),
        bytes: b"artifact-bytes".to_vec(),
    }
}

fn assert_unknown_task_aliases(client: &mut LiveTransportKamnClient) {
    assert_eq!(
        client.accept_task(&TaskId(77), &did("assignee-live")),
        Err(SdkError::NotFound {
            entity: "task",
            id: "77".to_owned(),
        })
    );
    assert_eq!(
        client.complete_task(&TaskId(77)),
        Err(SdkError::NotFound {
            entity: "task",
            id: "77".to_owned(),
        })
    );
    assert_eq!(
        client.submit_artifact(&TaskId(77), live_artifact()),
        Err(SdkError::NotFound {
            entity: "task",
            id: "77".to_owned(),
        })
    );
    assert_eq!(
        client.get_artifact_status(&ArtifactId(77)),
        Err(SdkError::NotFound {
            entity: "artifact",
            id: "77".to_owned(),
        })
    );
}

#[test]
fn regression_live_transport_submit_artifact_requires_accepted_task() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![create_task_request()];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    assert_eq!(
        client.submit_artifact(&task_id, live_artifact()),
        Err(SdkError::Conflict(
            "task must be accepted before artifact submission"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "unaccepted task artifact submission should not emit an extra network request"
    );
}

#[test]
fn regression_live_transport_artifact_status_rejects_malformed_service_payload() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![
        create_task_request(),
        accept_task_request(),
        submit_artifact_request(),
        ExpectedRequest {
            method: "GET",
            path: "/v1/content/content-local-artifact-abc".to_owned(),
            body: String::new(),
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "content:read",
            response_body: r#"{"content_id":"content-local-artifact-abc","redaction_status":"none"}"#.to_owned(),
            ..Default::default()
        },
    ];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    client
        .accept_task(&task_id, &did("assignee-live"))
        .expect("accept_task should succeed");
    let artifact_id = client
        .submit_artifact(&task_id, live_artifact())
        .expect("submit_artifact should succeed");
    assert_eq!(
        client.get_artifact_status(&artifact_id),
        Err(SdkError::TransportFailure(
            "service response missing required field"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "malformed content status should still satisfy request budget"
    );
}

fn assert_unknown_escrow_alias(client: &mut LiveTransportKamnClient) {
    assert_eq!(
        client.release_escrow(&EscrowId(88)),
        Err(SdkError::NotFound {
            entity: "escrow",
            id: "88".to_owned(),
        })
    );
}

fn assert_balance_route_fails_closed(client: &LiveTransportKamnClient) {
    assert_eq!(
        client.balance(&did("payer-live")),
        Err(SdkError::TransportFailure(
            "failed to connect to service endpoint"
        ))
    );
}
