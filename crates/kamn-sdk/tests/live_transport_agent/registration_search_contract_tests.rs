use super::support::*;

#[test]
fn spec_c06_live_transport_register_and_resolve_use_service_profile_metadata() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 2, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

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

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "live transport register/resolve server should satisfy request budget"
        );
    });
}

#[test]
fn spec_c07_live_transport_search_agents_uses_service_route() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, DEFAULT_LIVE_REQUESTER_DID, None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let results = client
            .search_agents(AgentQuery {
                capability: Some("code".to_owned()),
                model_family: Some("gpt-5".to_owned()),
            })
            .expect("live search_agents should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].did.as_str(), "kamn:did:agent:alpha");
        assert_eq!(results[0].agent_type, "assistant");
        assert_eq!(results[0].model_family, "gpt-5");
        assert_eq!(
            results[0].capabilities,
            vec!["text".to_owned(), "code".to_owned()]
        );

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "live transport search server should satisfy request budget"
        );
    });
}
