use kamn_sdk::{
    AgentDid, AgentMetadata, InMemoryKamnClient, KamnAgent, KamnTransport, LiveTransportConfig,
    LiveTransportKamnClient, Message, SdkError, TransportMode,
};
use std::time::Instant;

fn metadata(agent_type: &str, model: &str, capabilities: &[&str]) -> AgentMetadata {
    AgentMetadata {
        agent_type: agent_type.to_owned(),
        model_family: model.to_owned(),
        capabilities: capabilities.iter().map(|cap| (*cap).to_owned()).collect(),
    }
}

fn valid_did(identifier: &str) -> AgentDid {
    match AgentDid::parse(format!("kamn:did:agent:{identifier}")) {
        Ok(did) => did,
        Err(error) => panic!("did parse failed: {error}"),
    }
}

#[test]
fn unit_live_transport_config_rejects_non_live_endpoint() {
    assert_eq!(
        LiveTransportConfig::new("http://localhost:7000"),
        Err(SdkError::InvalidInput {
            field: "transport.endpoint",
            reason: "must start with https:// or wss://",
        })
    );
}

#[test]
fn functional_live_transport_register_and_receive_round_trip() {
    let mut client = match LiveTransportKamnClient::connect("https://live.kamn.testnet/functional")
    {
        Ok(value) => value,
        Err(error) => panic!("connect failed: {error}"),
    };
    let sender = match client.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match client.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    if let Err(error) = client.send(Message {
        from: sender,
        to: recipient.clone(),
        body: "live hello".to_owned(),
        channel: None,
    }) {
        panic!("send failed: {error}");
    }

    let messages = match client.receive(&recipient) {
        Ok(value) => value,
        Err(error) => panic!("receive failed: {error}"),
    };
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].message.body, "live hello");
}

#[test]
fn integration_live_transport_clients_share_endpoint_state() {
    let endpoint = "https://live.kamn.testnet/integration";
    let mut publisher = match LiveTransportKamnClient::connect(endpoint) {
        Ok(value) => value,
        Err(error) => panic!("connect publisher failed: {error}"),
    };
    let mut consumer = match LiveTransportKamnClient::connect(endpoint) {
        Ok(value) => value,
        Err(error) => panic!("connect consumer failed: {error}"),
    };

    let sender = match publisher.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match publisher.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    if let Err(error) = publisher.send(Message {
        from: sender,
        to: recipient.clone(),
        body: "shared endpoint message".to_owned(),
        channel: None,
    }) {
        panic!("publish failed: {error}");
    }

    let messages = match consumer.receive(&recipient) {
        Ok(value) => value,
        Err(error) => panic!("consume failed: {error}"),
    };
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].message.body, "shared endpoint message");
}

#[test]
fn regression_transport_mode_mismatch_is_rejected() {
    // Regression: #620
    let memory = InMemoryKamnClient::new();
    assert_eq!(
        memory.assert_transport_mode(TransportMode::Live),
        Err(SdkError::TransportModeMismatch {
            expected: "live",
            found: "in-memory",
        })
    );

    let live = match LiveTransportKamnClient::connect("https://live.kamn.testnet/mismatch") {
        Ok(value) => value,
        Err(error) => panic!("connect live failed: {error}"),
    };
    assert_eq!(
        live.assert_transport_mode(TransportMode::InMemory),
        Err(SdkError::TransportModeMismatch {
            expected: "in-memory",
            found: "live",
        })
    );
}

#[test]
fn performance_live_transport_contract_lane_stays_within_budget() {
    let endpoint = "https://live.kamn.testnet/perf";
    let mut client = match LiveTransportKamnClient::connect(endpoint) {
        Ok(value) => value,
        Err(error) => panic!("connect failed: {error}"),
    };
    let sender = match client.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match client.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    let start = Instant::now();
    for nonce in 1..=256 {
        if let Err(error) = client.send(Message {
            from: sender.clone(),
            to: recipient.clone(),
            body: format!("live-perf-message-{nonce}"),
            channel: None,
        }) {
            panic!("send failed at nonce {nonce}: {error}");
        }
    }

    let messages = match client.receive(&recipient) {
        Ok(value) => value,
        Err(error) => panic!("receive failed: {error}"),
    };
    assert_eq!(messages.len(), 256);

    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 300,
        "rust sdk live transport contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled live transport provider lane"]
fn performance_live_transport_multi_client_deep_lane() {
    let endpoint = "https://live.kamn.testnet/deep";
    let mut publisher = match LiveTransportKamnClient::connect(endpoint) {
        Ok(value) => value,
        Err(error) => panic!("connect publisher failed: {error}"),
    };
    let mut consumer = match LiveTransportKamnClient::connect(endpoint) {
        Ok(value) => value,
        Err(error) => panic!("connect consumer failed: {error}"),
    };

    let sender = match publisher.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match publisher.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    for nonce in 1..=5000 {
        if let Err(error) = publisher.send(Message {
            from: sender.clone(),
            to: recipient.clone(),
            body: format!("live-deep-message-{nonce}"),
            channel: Some(kamn_sdk::ChannelId("deep-lane".to_owned())),
        }) {
            panic!("send failed at nonce {nonce}: {error}");
        }
    }

    let messages = match consumer.receive(&recipient) {
        Ok(value) => value,
        Err(error) => panic!("consume failed: {error}"),
    };
    assert_eq!(messages.len(), 5000);
    assert_eq!(
        messages[0].message.channel,
        Some(kamn_sdk::ChannelId("deep-lane".to_owned()))
    );
}

#[test]
fn unit_live_transport_resolve_rejects_unregistered_did() {
    let client = match LiveTransportKamnClient::connect("https://live.kamn.testnet/not-found") {
        Ok(value) => value,
        Err(error) => panic!("connect failed: {error}"),
    };
    let unknown = valid_did("unknown-live-agent");
    assert_eq!(
        client.resolve(&unknown),
        Err(SdkError::NotFound {
            entity: "agent",
            id: "kamn:did:agent:unknown-live-agent".to_owned(),
        })
    );
}
