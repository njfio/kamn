use super::super::support::*;

pub(super) fn assert_registration_surface_contract() {
    let route_source = std::fs::read_to_string("src/service_client_bridge_misc_routes.rs")
        .expect("route source should be readable");
    assert!(route_source.contains("pub fn register_agent("));
    assert!(route_source.contains("pub fn search_agents("));
    let model_source = std::fs::read_to_string("src/service_models.rs")
        .expect("service models should be readable");
    assert!(model_source.contains("pub agent_type: String"));
    assert!(model_source.contains("pub model_family: String"));
    assert!(model_source.contains("pub capabilities: Vec<String>"));
}

pub(super) fn assert_task_and_escrow_routes() {
    let listener = bind_loopback_listener();
    let bind_addr = listener
        .local_addr()
        .expect("listener address should resolve")
        .to_string();
    let server = thread::spawn(move || run_bound_service_contract_server(listener, 4));
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender =
        AgentDid::parse("kamn:did:agent:sdk-task-escrow").expect("sender did should parse");
    assert_task_routes(&client, &sender);
    assert_escrow_routes(&client, &sender);
    assert_server_result(
        server,
        "test service contract server should satisfy request budget",
    );
}

pub(super) fn assert_bridge_routes() {
    let listener = bind_loopback_listener();
    let bind_addr = listener
        .local_addr()
        .expect("listener address should resolve")
        .to_string();
    let server = thread::spawn(move || run_bound_service_contract_server(listener, 3));
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-bridge").expect("sender did should parse");
    let submitted = submit_bridge_message(&client, &sender);
    let forwarded = forward_bridge_message(&client, &sender, submitted.bridge_id.as_str());
    let queried = query_bridge_message(&client, &sender, submitted.bridge_id.as_str());
    assert_eq!(queried.bridge_id, submitted.bridge_id);
    assert_eq!(queried.bridge_status, forwarded.bridge_status);
    assert_server_result(
        server,
        "test service contract server should satisfy request budget",
    );
}

fn submit_bridge_message(
    client: &ServiceApiClient,
    sender: &AgentDid,
) -> kamn_sdk::ServiceBridgeSubmission {
    let submit_payload = r#"{"source_message_id":"msg-sdk","target_network":"testnet"}"#;
    let submitted = client
        .submit_bridge_message(
            submit_payload,
            &auth_with_scope(sender, 1, submit_payload, "bridge:write"),
        )
        .expect("submit bridge should succeed");
    assert!(submitted.bridge_id.starts_with("bridge-local-"));
    submitted
}

fn forward_bridge_message(
    client: &ServiceApiClient,
    sender: &AgentDid,
    bridge_id: &str,
) -> kamn_sdk::ServiceBridgeStatus {
    client
        .forward_bridge_message(bridge_id, &auth_with_scope(sender, 2, "{}", "bridge:write"))
        .expect("forward bridge should succeed")
}

fn query_bridge_message(
    client: &ServiceApiClient,
    sender: &AgentDid,
    bridge_id: &str,
) -> kamn_sdk::ServiceBridgeStatus {
    client
        .get_bridge_message(bridge_id, &auth_with_scope(sender, 3, "", "bridge:read"))
        .expect("query bridge should succeed")
}

fn assert_task_routes(client: &ServiceApiClient, sender: &AgentDid) {
    let accepted = client
        .accept_task(
            "task-local-123",
            &auth_with_scope(sender, 1, "{}", "tasks:write"),
        )
        .expect("accept task should succeed");
    assert_eq!(accepted.state, "accepted");
    assert_service_authority(&accepted.receipt_id, &accepted.receipt_digest);
    let completed = client
        .complete_task(
            "task-local-123",
            &auth_with_scope(sender, 2, "{}", "tasks:write"),
        )
        .expect("complete task should succeed");
    assert_eq!(completed.state, "completed");
    assert_service_authority(&completed.receipt_id, &completed.receipt_digest);
}

fn assert_escrow_routes(client: &ServiceApiClient, sender: &AgentDid) {
    let fund_payload = r#"{"task_id":"task-local-123","amount":100}"#;
    let funded = client
        .fund_escrow(
            fund_payload,
            &auth_with_scope(sender, 3, fund_payload, "escrow:write"),
        )
        .expect("fund escrow should succeed");
    assert!(funded.escrow_id.starts_with("escrow-local-"));
    assert_service_authority(&funded.receipt_id, &funded.receipt_digest);
    let released = client
        .release_escrow(
            funded.escrow_id.as_str(),
            &auth_with_scope(sender, 4, "{}", "escrow:write"),
        )
        .expect("release escrow should succeed");
    assert_eq!(released.escrow_id, funded.escrow_id);
    assert_eq!(released.state, "released");
    assert_service_authority(&released.receipt_id, &released.receipt_digest);
}

fn assert_service_authority(receipt_id: &str, receipt_digest: &str) {
    assert!(!receipt_id.is_empty());
    assert!(receipt_digest.starts_with("sha256:"));
    assert_eq!(receipt_digest.len(), 71);
}

fn assert_server_result(server: thread::JoinHandle<Result<(), String>>, message: &str) {
    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "{message}");
}
