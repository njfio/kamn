use super::support::*;

#[test]
fn search_agents_filters_by_capability_and_reputation_exists() {
    let mut client = InMemoryKamnClient::new();
    let did_a = register_agent(&mut client, "autonomous", "claude-4", &["text", "code"]);
    let _did_b = register_agent(&mut client, "assistant", "gpt-5", &["text"]);
    let results = client
        .search_agents(AgentQuery {
            capability: Some("code".to_owned()),
            model_family: None,
        })
        .expect("search agents failed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].did, did_a.clone());
    let reputation = client
        .get_reputation(&did_a)
        .expect("get reputation failed");
    assert_eq!(reputation.did, did_a);
    assert!(reputation.score > 0);
}

#[test]
fn create_channel_returns_deterministic_channel_id() {
    let mut client = InMemoryKamnClient::new();
    assert_channel_id(
        client
            .create_channel("ops")
            .expect("first create should succeed"),
        "channel-local-1",
    );
    assert_channel_id(
        client
            .create_channel("ops")
            .expect("second create should succeed"),
        "channel-local-2",
    );
}

#[test]
fn create_channel_rejects_empty_or_whitespace_name() {
    let mut client = InMemoryKamnClient::new();
    let error = client
        .create_channel("   ")
        .expect_err("whitespace-only channel name must fail");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "channel_name",
            reason: "must not be empty",
        }
    );
}

#[test]
fn did_parse_rejects_wrong_prefix() {
    let invalid = AgentDid::parse("did:example:abc").map_err(SdkError::from);
    assert_eq!(
        invalid,
        Err(SdkError::InvalidInput {
            field: "did",
            reason: "must start with kamn:did:agent:",
        })
    );
    let parsed = valid_did("alpha-1");
    assert_eq!(parsed.as_str(), "kamn:did:agent:alpha-1");
}

#[test]
fn register_rejects_empty_capability_entries() {
    let mut client = InMemoryKamnClient::new();
    let result = client.register(metadata("autonomous", "claude-4", &["text", ""]));
    assert_eq!(
        result,
        Err(SdkError::InvalidInput {
            field: "capabilities",
            reason: "must not include empty capability entries",
        })
    );
}
