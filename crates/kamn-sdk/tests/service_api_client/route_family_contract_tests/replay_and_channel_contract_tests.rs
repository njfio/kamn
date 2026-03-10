use super::super::support::*;

pub(super) fn assert_replay_nonce_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 3));
    wait_for_server_ready(bind_addr.as_str());
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-replay").expect("sender did should parse");
    let payload = r#"{"message":"nonce replay"}"#;
    let replay_auth = auth_with_scope(&sender, 11, payload, "messages:write");
    client
        .send_message(payload, &replay_auth)
        .expect("first send should pass");
    assert_replay_error(&client, payload, &replay_auth);
    assert_signature_error(&client, payload, &sender);
    assert_server_result(
        server,
        "test service contract server should satisfy replay request budget",
    );
}

pub(super) fn assert_channel_messages_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 1));
    wait_for_server_ready(bind_addr.as_str());
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-list").expect("sender did should parse");
    let messages = client
        .list_channel_messages(
            "channel-local-123",
            &auth_with_scope(&sender, 1, "", "channels:read"),
        )
        .expect("list channel messages should succeed");
    assert_eq!(messages.channel_id, "channel-local-123");
    assert_eq!(
        messages.messages,
        vec!["msg-local-a".to_owned(), "msg-local-b".to_owned()]
    );
    assert_server_result(
        server,
        "test service contract server should satisfy request budget",
    );
}

fn assert_replay_error(client: &ServiceApiClient, payload: &str, replay_auth: &ServiceRequestAuth) {
    let replay_error = client
        .send_message(payload, replay_auth)
        .expect_err("replayed nonce should fail closed");
    assert!(replay_error
        .to_string()
        .contains("reason_code=service_api_auth_replay_nonce_detected"));
}

fn assert_signature_error(client: &ServiceApiClient, payload: &str, sender: &AgentDid) {
    let invalid_auth = auth_with_scope(
        sender,
        12,
        r#"{"message":"mismatch-signature"}"#,
        "messages:write",
    );
    let unauthorized_error = client
        .send_message(payload, &invalid_auth)
        .expect_err("signature mismatch should fail closed");
    assert!(unauthorized_error
        .to_string()
        .contains("reason_code=service_api_auth_signature_verification_failed"));
}

fn assert_server_result(server: thread::JoinHandle<Result<(), String>>, message: &str) {
    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "{message}");
}
