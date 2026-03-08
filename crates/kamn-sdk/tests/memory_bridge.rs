use kamn_sdk::{AgentMetadata, BridgeId, InMemoryKamnClient, KamnAgent, Message, SdkError};

fn metadata(agent_type: &str, model: &str, capabilities: &[&str]) -> AgentMetadata {
    AgentMetadata {
        agent_type: agent_type.to_owned(),
        model_family: model.to_owned(),
        capabilities: capabilities.iter().map(|cap| (*cap).to_owned()).collect(),
    }
}

#[test]
fn bridge_submit_forward_and_query_round_trip() {
    let mut client = InMemoryKamnClient::new();
    let sender = client
        .register(metadata("autonomous", "claude-4", &["text"]))
        .expect("register sender should succeed");
    let recipient = client
        .register(metadata("assistant", "gpt-5", &["text"]))
        .expect("register recipient should succeed");

    let source_message_id = client
        .send(Message {
            from: sender,
            to: recipient,
            body: "bridge me".to_owned(),
            channel: None,
        })
        .expect("send should succeed");

    let submitted = client
        .submit_bridge(&source_message_id, "testnet")
        .expect("submit_bridge should succeed");
    assert_eq!(submitted.bridge_status, "submitted");
    assert_eq!(submitted.target_message_id, None);
    assert_eq!(submitted.forward_tx_hash, None);

    let queried_submitted = client
        .get_bridge_status(&submitted.bridge_id)
        .expect("submitted bridge should be queryable");
    assert_eq!(queried_submitted, submitted);

    let forwarded = client
        .forward_bridge(&submitted.bridge_id)
        .expect("forward_bridge should succeed");
    assert_eq!(forwarded.bridge_status, "forwarded");
    assert!(forwarded.target_message_id.is_some());
    assert!(forwarded.forward_tx_hash.is_some());

    let queried_forwarded = client
        .get_bridge_status(&submitted.bridge_id)
        .expect("forwarded bridge should be queryable");
    assert_eq!(queried_forwarded, forwarded);
}

#[test]
fn bridge_submit_rejects_unknown_source_message() {
    let mut client = InMemoryKamnClient::new();

    assert_eq!(
        client.submit_bridge(&kamn_sdk::MessageId(41), "testnet"),
        Err(SdkError::NotFound {
            entity: "message",
            id: "41".to_owned(),
        })
    );
}

#[test]
fn bridge_forward_and_query_reject_unknown_bridge() {
    let mut client = InMemoryKamnClient::new();
    let bridge_id = BridgeId(52);

    assert_eq!(
        client.get_bridge_status(&bridge_id),
        Err(SdkError::NotFound {
            entity: "bridge",
            id: "52".to_owned(),
        })
    );
    assert_eq!(
        client.forward_bridge(&bridge_id),
        Err(SdkError::NotFound {
            entity: "bridge",
            id: "52".to_owned(),
        })
    );
}
