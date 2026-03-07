#[path = "support/live_transport_task_escrow.rs"]
mod support;

use kamn_sdk::{
    EscrowConfig, EscrowId, KamnAgent, LiveTransportKamnClient, SdkError, TaskDefinition, TaskId,
    TokenAmount,
};
use std::thread;

use support::{
    did, ensure_live_test_env, reserve_loopback_addr, run_contract_server, wait_for_server_ready,
    ExpectedRequest,
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
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || {
        run_contract_server(
            server_addr,
            vec![
                ExpectedRequest {
                    method: "POST",
                    path: "/v1/tasks/create".to_owned(),
                    body: r#"{"creator":"kamn:did:agent:creator-live","task_type":"triage","description":"triage contract"}"#.to_owned(),
                    sender_did: "kamn:did:agent:creator-live".to_owned(),
                    scope: "tasks:write",
                    response_status: 201,
                    response_body: r#"{"task_id":"task-local-abc","state":"submitted"}"#.to_owned(),
                },
                ExpectedRequest {
                    method: "POST",
                    path: "/v1/tasks/task-local-abc/accept".to_owned(),
                    body: "{}".to_owned(),
                    sender_did: "kamn:did:agent:assignee-live".to_owned(),
                    scope: "tasks:write",
                    response_status: 200,
                    response_body: r#"{"task_id":"task-local-abc","state":"accepted"}"#.to_owned(),
                },
                ExpectedRequest {
                    method: "POST",
                    path: "/v1/tasks/task-local-abc/complete".to_owned(),
                    body: "{}".to_owned(),
                    sender_did: "kamn:did:agent:assignee-live".to_owned(),
                    scope: "tasks:write",
                    response_status: 200,
                    response_body: r#"{"task_id":"task-local-abc","state":"completed"}"#.to_owned(),
                },
                ExpectedRequest {
                    method: "POST",
                    path: "/v1/escrow/fund".to_owned(),
                    body: r#"{"payer":"kamn:did:agent:payer-live","payee":"kamn:did:agent:payee-live","amount":7}"#.to_owned(),
                    sender_did: "kamn:did:agent:payer-live".to_owned(),
                    scope: "escrow:write",
                    response_status: 200,
                    response_body: r#"{"escrow_id":"escrow-local-xyz","state":"funded"}"#.to_owned(),
                },
                ExpectedRequest {
                    method: "POST",
                    path: "/v1/escrow/escrow-local-xyz/release".to_owned(),
                    body: "{}".to_owned(),
                    sender_did: "kamn:did:agent:payer-live".to_owned(),
                    scope: "escrow:write",
                    response_status: 200,
                    response_body: r#"{"escrow_id":"escrow-local-xyz","state":"released"}"#.to_owned(),
                },
            ],
        )
    });
    wait_for_server_ready();

    let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
        .expect("live client should connect");
    let task_id = client
        .create_task(TaskDefinition {
            creator: did("creator-live"),
            task_type: "triage".to_owned(),
            description: "triage contract".to_owned(),
        })
        .expect("create_task should succeed");
    assert_eq!(task_id, TaskId(deterministic_u64_tag("task-local-abc")));

    client
        .accept_task(&task_id, &did("assignee-live"))
        .expect("accept_task should succeed");
    client
        .complete_task(&task_id)
        .expect("complete_task should succeed");

    let escrow_id = client
        .create_escrow(EscrowConfig {
            payer: did("payer-live"),
            payee: did("payee-live"),
            amount: TokenAmount(7),
        })
        .expect("create_escrow should succeed");
    assert_eq!(escrow_id, EscrowId(deterministic_u64_tag("escrow-local-xyz")));

    client
        .release_escrow(&escrow_id)
        .expect("release_escrow should succeed");

    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "task/escrow route server should satisfy request budget");
}

#[test]
fn regression_live_transport_unknown_task_and_escrow_aliases_fail_closed() {
    ensure_live_test_env();
    let mut client = LiveTransportKamnClient::connect("http://127.0.0.1:65535")
        .expect("endpoint format should be accepted");

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
        client.release_escrow(&EscrowId(88)),
        Err(SdkError::NotFound {
            entity: "escrow",
            id: "88".to_owned(),
        })
    );
    assert_eq!(
        client.balance(&did("payer-live")),
        Err(SdkError::NotImplemented(
            "live transport balance route is not available via service api"
        ))
    );
}
