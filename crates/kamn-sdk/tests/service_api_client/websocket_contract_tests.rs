use super::support::*;

#[test]
fn integration_service_api_client_reads_websocket_event_frame() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 1));
    wait_for_server_ready(bind_addr.as_str());
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-events").expect("sender did should parse");
    let event = client
        .read_event_once(&auth_with_scope(&sender, 9, "", "events:read"))
        .expect("event read should succeed");
    assert_eq!(event.event, "state-transition");
    assert_eq!(event.runtime_mode, "api");
    assert_eq!(event.role, "processor");
    assert_eq!(event.sequence, 1);
    assert_server_result(server);
}

#[test]
fn integration_service_api_client_reads_websocket_event_frame_extended_length() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let note = "x".repeat(200);
    let websocket_payload = format!(
        "{{\"event\":\"state-transition\",\"runtime_mode\":\"api\",\"role\":\"processor\",\"sequence\":1,\"note\":\"{note}\"}}"
    );
    let server = thread::spawn(move || {
        run_service_contract_server_with_websocket_payload(server_addr, 1, websocket_payload)
    });
    wait_for_server_ready(bind_addr.as_str());
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender =
        AgentDid::parse("kamn:did:agent:sdk-events-extended").expect("sender did should parse");
    let event = client
        .read_event_once(&auth_with_scope(&sender, 10, "", "events:read"))
        .expect("event read should succeed");
    assert_eq!(event.event, "state-transition");
    assert_eq!(event.runtime_mode, "api");
    assert_eq!(event.role, "processor");
    assert_eq!(event.sequence, 1);
    assert_server_result(server);
}

fn assert_server_result(server: thread::JoinHandle<Result<(), String>>) {
    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy websocket request budget"
    );
}
