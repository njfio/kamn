#[path = "support/live_transport_task_escrow.rs"]
mod support;

use kamn_sdk::{KamnAgent, LiveTransportKamnClient, Message, MessageId, MessageRecord, SdkError};
use std::thread;

use support::{
    did, ensure_live_test_env, expected_request, reserve_loopback_addr, run_contract_server,
    wait_for_server_ready, ExpectedRequest,
};

const REQUESTER_DID: &str = "kamn:did:agent:live-requester";
const RECIPIENT_DID: &str = "kamn:did:agent:receiver-live";
const SENDER_DID: &str = "kamn:did:agent:sender-live";

fn deterministic_u64_tag(value: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

#[test]
fn spec_c07_live_transport_receive_routes_execute_network_contract() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![
        ExpectedRequest {
            sender_did: REQUESTER_DID.to_owned(),
            scope: "agents:read",
            response_body: format!(
                r#"{{"channel_id":"recipient:{RECIPIENT_DID}","messages":["msg-live-1"]}}"#
            ),
            ..expected_request(
                "GET",
                format!("/v1/channels/recipient:{RECIPIENT_DID}/messages").as_str(),
                "",
            )
        },
        ExpectedRequest {
            sender_did: REQUESTER_DID.to_owned(),
            scope: "agents:read",
            response_body: format!(
                r#"{{"message_id":"msg-live-1","status":"delivered","sender_did":"{SENDER_DID}","recipient_did":"{RECIPIENT_DID}","body":"hello-live"}}"#
            ),
            ..expected_request("GET", "/v1/messages/msg-live-1", "")
        },
    ];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let messages = client
        .receive(&did("receiver-live"))
        .expect("live receive should succeed");

    assert_eq!(
        messages,
        vec![MessageRecord {
            id: MessageId(deterministic_u64_tag("msg-live-1")),
            message: Message {
                from: did("sender-live"),
                to: did("receiver-live"),
                body: "hello-live".to_owned(),
                channel: None,
            },
        }]
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "live receive contract server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_receive_stream_uses_real_receive_path() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![
        ExpectedRequest {
            sender_did: REQUESTER_DID.to_owned(),
            scope: "agents:read",
            response_body: format!(
                r#"{{"channel_id":"recipient:{RECIPIENT_DID}","messages":["msg-stream-1"]}}"#
            ),
            ..expected_request(
                "GET",
                format!("/v1/channels/recipient:{RECIPIENT_DID}/messages").as_str(),
                "",
            )
        },
        ExpectedRequest {
            sender_did: REQUESTER_DID.to_owned(),
            scope: "agents:read",
            response_body: format!(
                r#"{{"message_id":"msg-stream-1","status":"delivered","sender_did":"{SENDER_DID}","recipient_did":"{RECIPIENT_DID}","body":"stream-live"}}"#
            ),
            ..expected_request("GET", "/v1/messages/msg-stream-1", "")
        },
    ];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let records: Vec<_> = client
        .receive_stream(&did("receiver-live"))
        .expect("live receive stream should succeed")
        .collect();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].message.body, "stream-live");

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "live receive stream server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_empty_mailbox_returns_no_records() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![ExpectedRequest {
        sender_did: REQUESTER_DID.to_owned(),
        scope: "agents:read",
        response_body: format!(r#"{{"channel_id":"recipient:{RECIPIENT_DID}","messages":[]}}"#),
        ..expected_request(
            "GET",
            format!("/v1/channels/recipient:{RECIPIENT_DID}/messages").as_str(),
            "",
        )
    }];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    assert_eq!(
        client
            .receive(&did("receiver-live"))
            .expect("empty mailbox should still succeed"),
        Vec::<MessageRecord>::new()
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "empty mailbox server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_malformed_message_payload_fails_closed() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![
        ExpectedRequest {
            sender_did: REQUESTER_DID.to_owned(),
            scope: "agents:read",
            response_body: format!(
                r#"{{"channel_id":"recipient:{RECIPIENT_DID}","messages":["msg-bad-1"]}}"#
            ),
            ..expected_request(
                "GET",
                format!("/v1/channels/recipient:{RECIPIENT_DID}/messages").as_str(),
                "",
            )
        },
        ExpectedRequest {
            sender_did: REQUESTER_DID.to_owned(),
            scope: "agents:read",
            response_body: format!(
                r#"{{"message_id":"msg-bad-1","status":"delivered","sender_did":"{SENDER_DID}","recipient_did":"{RECIPIENT_DID}"}}"#
            ),
            ..expected_request("GET", "/v1/messages/msg-bad-1", "")
        },
    ];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    assert_eq!(
        client.receive(&did("receiver-live")),
        Err(SdkError::TransportFailure(
            "service message response missing required body"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "malformed payload server should satisfy request budget"
    );
}

fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}
