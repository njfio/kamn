#[path = "support/live_transport_task_escrow.rs"]
mod support;

use kamn_sdk::{BridgeId, KamnAgent, LiveTransportKamnClient, Message, SdkError};
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
fn spec_c09_live_transport_bridge_routes_execute_network_contract() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, bridge_requests()));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let source_message_id = client
        .send(Message {
            from: did("sender-bridge-live"),
            to: did("recipient-bridge-live"),
            body: "bridge contract payload".to_owned(),
            channel: None,
        })
        .expect("send should succeed");

    let submitted = client
        .submit_bridge(&source_message_id, "testnet")
        .expect("submit_bridge should succeed");
    assert_eq!(submitted.bridge_id, BridgeId(deterministic_u64_tag("bridge-local-abc")));
    assert_eq!(submitted.bridge_status, "submitted");
    assert_eq!(submitted.target_message_id, None);
    assert_eq!(submitted.forward_tx_hash, None);

    let queried_submitted = client
        .get_bridge_status(&submitted.bridge_id)
        .expect("submitted bridge should be queryable");
    assert_eq!(queried_submitted, submitted);

    let forwarded = client
        .forward_bridge(&submitted.bridge_id)
        .expect("forward_bridge should succeed");
    assert_eq!(forwarded.bridge_status, "forwarded");
    assert_eq!(
        forwarded.target_message_id,
        Some(kamn_sdk::MessageId(deterministic_u64_tag(
            "msg-bridge-target-bridge-local-abc"
        )))
    );
    assert_eq!(
        forwarded.forward_tx_hash,
        Some("sha256:bridge-forwarded-bridge-local-abc".to_owned())
    );

    let queried_forwarded = client
        .get_bridge_status(&submitted.bridge_id)
        .expect("forwarded bridge should be queryable");
    assert_eq!(queried_forwarded, forwarded);

    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "bridge route server should satisfy request budget");
}

#[test]
fn regression_live_transport_bridge_status_rejects_unknown_alias_before_network() {
    ensure_live_test_env();
    let client = live_client("127.0.0.1:65535");

    assert_eq!(
        client.get_bridge_status(&BridgeId(404)),
        Err(SdkError::NotFound {
            entity: "bridge",
            id: "404".to_owned(),
        })
    );
    assert_eq!(
        client.forward_bridge(&BridgeId(404)),
        Err(SdkError::NotFound {
            entity: "bridge",
            id: "404".to_owned(),
        })
    );
}

#[test]
fn regression_live_transport_bridge_status_rejects_malformed_service_payload() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server =
        thread::spawn(move || run_contract_server(server_addr, malformed_bridge_requests()));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let source_message_id = client
        .send(Message {
            from: did("sender-bridge-live"),
            to: did("recipient-bridge-live"),
            body: "bridge contract payload".to_owned(),
            channel: None,
        })
        .expect("send should succeed");
    let submitted = client
        .submit_bridge(&source_message_id, "testnet")
        .expect("submit_bridge should succeed");

    assert_eq!(
        client.get_bridge_status(&submitted.bridge_id),
        Err(SdkError::TransportFailure(
            "service response missing required field"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "malformed bridge status should satisfy request budget");
}

fn bridge_requests() -> Vec<ExpectedRequest> {
    vec![
        ExpectedRequest {
            sender_did: "kamn:did:agent:sender-bridge-live".to_owned(),
            scope: "messages:write",
            response_status: 202,
            response_body:
                r#"{"message_id":"msg-live-bridge-source","status":"created","runtime_mode":"api"}"#
                    .to_owned(),
            ..expected_request(
                "POST",
                "/v1/messages/send",
                r#"{"from":"kamn:did:agent:sender-bridge-live","to":"kamn:did:agent:recipient-bridge-live","body":"bridge contract payload"}"#,
            )
        },
        ExpectedRequest {
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "bridge:write",
            response_status: 202,
            response_body:
                r#"{"bridge_id":"bridge-local-abc","source_message_id":"msg-live-bridge-source","bridge_status":"submitted"}"#
                    .to_owned(),
            ..expected_request(
                "POST",
                "/v1/bridge/submit",
                r#"{"source_message_id":"msg-live-bridge-source","target_network":"testnet"}"#,
            )
        },
        ExpectedRequest {
            method: "GET",
            path: "/v1/bridge/bridge-local-abc".to_owned(),
            body: String::new(),
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "bridge:read",
            response_body:
                r#"{"bridge_id":"bridge-local-abc","bridge_status":"submitted","target_message_id":"","forward_tx_hash":""}"#
                    .to_owned(),
            ..Default::default()
        },
        ExpectedRequest {
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "bridge:write",
            response_body:
                r#"{"bridge_id":"bridge-local-abc","bridge_status":"forwarded","target_message_id":"msg-bridge-target-bridge-local-abc","forward_tx_hash":"sha256:bridge-forwarded-bridge-local-abc"}"#
                    .to_owned(),
            ..expected_request("POST", "/v1/bridge/bridge-local-abc/forward", "{}")
        },
        ExpectedRequest {
            method: "GET",
            path: "/v1/bridge/bridge-local-abc".to_owned(),
            body: String::new(),
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "bridge:read",
            response_body:
                r#"{"bridge_id":"bridge-local-abc","bridge_status":"forwarded","target_message_id":"msg-bridge-target-bridge-local-abc","forward_tx_hash":"sha256:bridge-forwarded-bridge-local-abc"}"#
                    .to_owned(),
            ..Default::default()
        },
    ]
}

fn malformed_bridge_requests() -> Vec<ExpectedRequest> {
    vec![
        ExpectedRequest {
            sender_did: "kamn:did:agent:sender-bridge-live".to_owned(),
            scope: "messages:write",
            response_status: 202,
            response_body:
                r#"{"message_id":"msg-live-bridge-source","status":"created","runtime_mode":"api"}"#
                    .to_owned(),
            ..expected_request(
                "POST",
                "/v1/messages/send",
                r#"{"from":"kamn:did:agent:sender-bridge-live","to":"kamn:did:agent:recipient-bridge-live","body":"bridge contract payload"}"#,
            )
        },
        ExpectedRequest {
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "bridge:write",
            response_status: 202,
            response_body:
                r#"{"bridge_id":"bridge-local-abc","source_message_id":"msg-live-bridge-source","bridge_status":"submitted"}"#
                    .to_owned(),
            ..expected_request(
                "POST",
                "/v1/bridge/submit",
                r#"{"source_message_id":"msg-live-bridge-source","target_network":"testnet"}"#,
            )
        },
        ExpectedRequest {
            method: "GET",
            path: "/v1/bridge/bridge-local-abc".to_owned(),
            body: String::new(),
            sender_did: "kamn:did:agent:live-requester".to_owned(),
            scope: "bridge:read",
            response_body: r#"{"bridge_id":"bridge-local-abc"}"#.to_owned(),
            ..Default::default()
        },
    ]
}

fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}
