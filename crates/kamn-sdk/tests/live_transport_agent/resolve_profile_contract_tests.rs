use super::support::*;

#[test]
fn spec_c03_live_transport_resolve_and_reputation_use_network_contract() {
    with_env_lock(|| {
        let (bind_addr, server) = start_contract_server(2, "kamn:did:agent:live-tester", None);

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let target = did("agent-profile-target");

        let document = client.resolve(&target).expect("resolve should succeed");
        assert_resolved_document(&document, &target, bind_addr.as_str());

        let reputation = client
            .get_reputation(&did("agent-profile-target"))
            .expect("reputation query should succeed");
        assert_reputation(&reputation);

        assert_server_result(
            server,
            "test service contract server should satisfy request budget",
        );
    });
}

fn assert_resolved_document(document: &kamn_sdk::DidDocument, target: &AgentDid, bind_addr: &str) {
    assert_eq!(document.id, *target);
    assert_eq!(document.metadata.agent_type, "service-agent");
    assert_eq!(document.metadata.model_family, "service-api");
    assert_eq!(document.service_endpoint, format!("http://{bind_addr}"));
}

fn assert_reputation(reputation: &kamn_sdk::AgentReputation) {
    assert_eq!(reputation.did, did("agent-profile-target"));
    assert_eq!(reputation.score, 777);
}

#[test]
fn regression_live_transport_whitespace_requester_did_falls_back_to_default() {
    with_env_lock(|| {
        ensure_live_test_env();
        std::env::set_var(LIVE_REQUESTER_DID_ENV, "   ");
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, DEFAULT_LIVE_REQUESTER_DID, None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should use default requester did when env is whitespace");
        let _ = client
            .resolve(&did("agent-profile-target"))
            .expect("resolve should succeed with default requester did");

        assert_server_result(
            server,
            "whitespace requester did env should fallback to default requester did",
        );
    });
}
