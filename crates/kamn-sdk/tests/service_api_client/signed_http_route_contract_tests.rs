use super::support::*;

#[path = "signed_http_route_contract_tests/agent_content_metrics_contract_tests.rs"]
mod agent_content_metrics_contract_tests;
#[path = "signed_http_route_contract_tests/message_channel_contract_tests.rs"]
mod message_channel_contract_tests;

#[test]
fn functional_service_api_client_executes_signed_http_route_contracts() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 14));
    wait_for_server_ready(bind_addr.as_str());
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-client").expect("sender did should parse");
    message_channel_contract_tests::assert_message_channel_routes(&client, &sender);
    agent_content_metrics_contract_tests::assert_agent_content_routes(&client, &sender);
    agent_content_metrics_contract_tests::assert_service_health_and_metrics(&client);
    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}
