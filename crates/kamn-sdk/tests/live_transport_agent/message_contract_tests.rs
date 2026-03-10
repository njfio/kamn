use super::support::*;

#[path = "message_contract_tests/malformed_status_support.rs"]
mod malformed_status_support;

#[test]
fn spec_c02_live_transport_send_executes_network_contract() {
    with_env_lock(|| {
        let (bind_addr, server) = start_contract_server(2, "kamn:did:agent:live-tester", None);
        let mut client = connect_live_client(bind_addr.as_str());
        let message_id = client
            .send(Message {
                from: did("sender-live-contract"),
                to: did("recipient-live-contract"),
                body: "live contract payload".to_owned(),
                channel: None,
            })
            .expect("live send should succeed");
        assert_eq!(
            message_id.0,
            deterministic_message_id("msg-live-contract-001")
        );
        assert_eq!(
            client
                .get_message_status(&message_id)
                .expect("message status should succeed")
                .status,
            "created"
        );
        malformed_status_support::assert_server_result(
            server,
            "test service contract server should satisfy request budget",
        );
    });
}

#[test]
fn regression_live_transport_duplicate_service_message_id_reuses_alias() {
    with_env_lock(|| {
        let (bind_addr, server) = start_contract_server(2, "kamn:did:agent:live-tester", None);
        let mut client = connect_live_client(bind_addr.as_str());
        let first = send_contract_message(&mut client, "live contract payload one");
        let second = send_contract_message(&mut client, "live contract payload two");
        assert_eq!(first, second, "same service id must map to same sdk alias");
        malformed_status_support::assert_server_result(
            server,
            "test service contract server should satisfy request budget",
        );
    });
}

#[test]
fn regression_live_transport_message_status_rejects_malformed_service_payload() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            malformed_status_support::run_malformed_message_status_server(server_addr)
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = connect_live_client(bind_addr.as_str());
        let message_id = send_contract_message(&mut client, "live contract payload");
        assert_eq!(
            client.get_message_status(&message_id),
            Err(SdkError::TransportFailure(
                "service response missing required field"
            ))
        );

        malformed_status_support::assert_server_result(
            server,
            "malformed message status should still satisfy request budget",
        );
    });
}

#[test]
fn regression_live_transport_message_status_rejects_unknown_alias_before_network() {
    with_env_lock(|| {
        ensure_live_test_env();
        let endpoint = format!("http://{}", reserve_loopback_addr());
        let client = LiveTransportKamnClient::connect(endpoint.as_str())
            .expect("live transport client should construct");
        assert_eq!(
            client.get_message_status(&MessageId(404)),
            Err(SdkError::NotFound {
                entity: "message",
                id: "404".to_owned(),
            })
        );
    });
}

#[test]
fn spec_c04_live_transport_send_escapes_json_payload_contract() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let expected_payload = "{\"from\":\"kamn:did:agent:sender-escape\",\"to\":\"kamn:did:agent:recipient-escape\",\"body\":\"line\\n\\t\\\"slash\\\\bell\\u0007\",\"channel_id\":\"ops\\\"lane\"}".to_owned();
        let expected_for_server = expected_payload.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(
                server_addr,
                1,
                "kamn:did:agent:live-tester",
                Some(expected_for_server),
            )
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = connect_live_client(bind_addr.as_str());
        let message_id = client
            .send(Message {
                from: did("sender-escape"),
                to: did("recipient-escape"),
                body: "line\n\t\"slash\\bell\u{0007}".to_owned(),
                channel: Some(kamn_sdk::ChannelId("ops\"lane".to_owned())),
            })
            .expect("send should succeed");
        assert_eq!(
            message_id.0,
            deterministic_message_id("msg-live-contract-001")
        );

        malformed_status_support::assert_server_result(
            server,
            "message payload must match json-escaped contract",
        );
        assert!(
            expected_payload.contains("\\u0007"),
            "expected payload fixture should include control-char escape marker"
        );
    });
}

fn start_contract_server(
    request_budget: u64,
    expected_agent_sender_did: &str,
    expected_message_body: Option<String>,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let expected_sender = expected_agent_sender_did.to_owned();
    let server = thread::spawn(move || {
        run_live_transport_contract_server(
            server_addr,
            request_budget,
            expected_sender.as_str(),
            expected_message_body,
        )
    });
    wait_for_server_ready(bind_addr.as_str());
    (bind_addr, server)
}

fn connect_live_client(bind_addr: &str) -> LiveTransportKamnClient {
    LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
        .expect("live client should connect")
}

fn send_contract_message(client: &mut LiveTransportKamnClient, body: &str) -> MessageId {
    client
        .send(Message {
            from: did("sender-live-contract"),
            to: did("recipient-live-contract"),
            body: body.to_owned(),
            channel: None,
        })
        .expect("live send should succeed")
}
