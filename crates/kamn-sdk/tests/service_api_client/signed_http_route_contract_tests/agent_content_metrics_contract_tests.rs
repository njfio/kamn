use super::super::support::*;

pub(super) fn assert_agent_content_routes(client: &ServiceApiClient, sender: &AgentDid) {
    assert_profile_route(client, sender);
    assert_registration_and_search(client, sender);
    assert_content_routes(client, sender);
}

pub(super) fn assert_service_health_and_metrics(client: &ServiceApiClient) {
    let health = client.health().expect("health route should succeed");
    assert_eq!(health.status, "ok");
    assert_eq!(health.runtime_mode, "api");
    let metrics = client.metrics().expect("metrics route should succeed");
    assert!(
        metrics.contains("kamn_service_api_health{runtime_mode=\"api\"} 1"),
        "metrics contract should include service health gauge"
    );
}

fn assert_profile_route(client: &ServiceApiClient, sender: &AgentDid) {
    let profile = client
        .get_agent_profile(
            sender.as_str(),
            &auth_with_scope(sender, 6, "", "agents:read"),
        )
        .expect("agent profile should resolve");
    assert_eq!(profile.did, sender.as_str());
    assert_eq!(profile.reputation_score, 500);
    assert_eq!(profile.agent_type, "service-agent");
    assert_eq!(profile.model_family, "service-api");
    assert_eq!(profile.capabilities, vec!["profile:read".to_owned()]);
}

fn assert_registration_and_search(client: &ServiceApiClient, sender: &AgentDid) {
    let registration = register_agent(client, sender);
    assert_eq!(registration.did, "kamn:did:agent:sdk-register");
    let search_results = search_agents(client, sender);
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].did, "kamn:did:agent:sdk-register");
}

fn register_agent(client: &ServiceApiClient, sender: &AgentDid) -> kamn_sdk::ServiceAgentProfile {
    let registration_payload = serde_json::json!({
        "agent_type": "assistant",
        "model_family": "gpt-5",
        "capabilities": ["text", "code"],
    })
    .to_string();
    client
        .register_agent(
            &kamn_sdk::AgentMetadata {
                agent_type: "assistant".to_owned(),
                model_family: "gpt-5".to_owned(),
                capabilities: vec!["text".to_owned(), "code".to_owned()],
            },
            &auth_with_scope(sender, 7, registration_payload.as_str(), "agents:write"),
        )
        .expect("agent registration should succeed")
}

fn search_agents(
    client: &ServiceApiClient,
    sender: &AgentDid,
) -> Vec<kamn_sdk::ServiceAgentProfile> {
    client
        .search_agents(
            &kamn_sdk::AgentQuery {
                capability: Some("code".to_owned()),
                model_family: Some("gpt-5".to_owned()),
            },
            &auth_with_scope(
                sender,
                8,
                r#"{"capability":"code","model_family":"gpt-5"}"#,
                "agents:read",
            ),
        )
        .expect("agent search should succeed")
}

fn assert_content_routes(client: &ServiceApiClient, sender: &AgentDid) {
    let payload = r#"{"task_id":"task-local-sdk","artifact_name":"artifact.bin","artifact_bytes_hex":"616263"}"#;
    let content_registration = client
        .register_content(
            payload,
            &auth_with_scope(sender, 9, payload, "content:write"),
        )
        .expect("content registration should succeed");
    assert_eq!(content_registration.content_id, "content-local-sdk");
    let content_status = client
        .get_content(
            content_registration.content_id.as_str(),
            &auth_with_scope(sender, 10, "", "content:read"),
        )
        .expect("content status should succeed");
    assert_eq!(content_status.lifecycle_state, "retained");
    assert_content_mutations(client, sender, content_registration.content_id.as_str());
}

fn assert_content_mutations(client: &ServiceApiClient, sender: &AgentDid, content_id: &str) {
    let expired_content = client
        .expire_content(
            content_id,
            &auth_with_scope(sender, 11, "{}", "content:write"),
        )
        .expect("content expire should succeed");
    assert_eq!(expired_content.lifecycle_state, "expired");
    let tombstoned_content = client
        .tombstone_content(
            content_id,
            &auth_with_scope(sender, 12, "{}", "content:write"),
        )
        .expect("content tombstone should succeed");
    assert_eq!(tombstoned_content.lifecycle_state, "tombstoned");
    assert_eq!(tombstoned_content.redaction_status, "redacted");
}
