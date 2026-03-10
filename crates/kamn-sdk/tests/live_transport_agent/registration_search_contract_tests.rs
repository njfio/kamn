use super::support::*;

#[test]
fn spec_c06_live_transport_register_and_resolve_use_service_profile_metadata() {
    with_env_lock(|| {
        let (bind_addr, server) = start_contract_server(2, "kamn:did:agent:live-tester", None);

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let did = client
            .register(metadata("assistant", "gpt-5", &["text", "code"]))
            .expect("register should succeed over live transport");
        let resolved = client.resolve(&did).expect("resolve should succeed");

        assert_eq!(resolved.id, did);
        assert_eq!(resolved.metadata.agent_type, "assistant");
        assert_eq!(resolved.metadata.model_family, "gpt-5");
        assert_eq!(
            resolved.metadata.capabilities,
            vec!["text".to_owned(), "code".to_owned()]
        );

        assert_server_result(
            server,
            "live transport register/resolve server should satisfy request budget",
        );
    });
}

#[test]
fn spec_c07_live_transport_search_agents_uses_service_route() {
    with_env_lock(|| {
        let (bind_addr, server) = start_contract_server(1, DEFAULT_LIVE_REQUESTER_DID, None);

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let results = search_code_agents(&client);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].did.as_str(), "kamn:did:agent:alpha");
        assert_eq!(results[0].agent_type, "assistant");
        assert_eq!(results[0].model_family, "gpt-5");
        assert_eq!(
            results[0].capabilities,
            vec!["text".to_owned(), "code".to_owned()]
        );

        assert_server_result(
            server,
            "live transport search server should satisfy request budget",
        );
    });
}

fn search_code_agents(client: &LiveTransportKamnClient) -> Vec<kamn_sdk::AgentSummary> {
    client
        .search_agents(AgentQuery {
            capability: Some("code".to_owned()),
            model_family: Some("gpt-5".to_owned()),
        })
        .expect("live search_agents should succeed")
}
