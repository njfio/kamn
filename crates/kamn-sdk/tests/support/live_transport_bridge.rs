#[path = "live_transport_task_escrow.rs"]
mod shared;

use kamn_sdk::LiveTransportKamnClient;

pub(crate) use self::shared::{
    ExpectedRequest, did, ensure_live_test_env, expected_request, reserve_loopback_addr,
    run_contract_server, wait_for_server_ready,
};

pub(crate) fn deterministic_u64_tag(value: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

pub(crate) fn bridge_requests() -> Vec<ExpectedRequest> {
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

pub(crate) fn malformed_bridge_requests() -> Vec<ExpectedRequest> {
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

pub(crate) fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}
