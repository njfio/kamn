use kamn_core::{
    BridgeInboundEnvelope, TelegramBridgeConfig, TelegramBridgeEngine, TelegramBridgeError,
    TelegramInboundRequest,
};
use std::collections::{BTreeMap, BTreeSet};

fn config() -> TelegramBridgeConfig {
    let mut channel_routes = BTreeMap::new();
    channel_routes.insert(
        "telegram:channel:ops".to_owned(),
        "kamn:did:agent:listener-target-1".to_owned(),
    );

    TelegramBridgeConfig {
        bridge_agent_did: "kamn:did:agent:bridge-telegram-1".to_owned(),
        authorized_listener_dids: ["kamn:did:agent:listener-1".to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        channel_routes,
    }
}

fn inbound(target: &str) -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: "tg-msg-1".to_owned(),
        external_sender_id: "tg-user-9".to_owned(),
        external_channel_id: "telegram:channel:ops".to_owned(),
        target_agent_did: target.to_owned(),
        body: "run diagnostics".to_owned(),
        received_at: "2026-02-07T21:00:00Z".to_owned(),
    }
}

#[test]
fn authorized_listener_can_process_inbound() {
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    let normalized = engine
        .process_inbound(&TelegramInboundRequest {
            listener_did: "kamn:did:agent:listener-1".to_owned(),
            inbound: inbound("kamn:did:agent:listener-target-1"),
        })
        .expect("inbound should process");

    assert_eq!(normalized.sender_handle, "tg-user-9");
    assert_eq!(
        normalized.target_agent_did,
        "kamn:did:agent:listener-target-1"
    );
}

#[test]
fn integration_processes_inbound_to_canonical_envelope() {
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    let envelope = engine
        .process_inbound_to_envelope(
            &TelegramInboundRequest {
                listener_did: "kamn:did:agent:listener-1".to_owned(),
                inbound: inbound("kamn:did:agent:listener-target-1"),
            },
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-07T21:15:00Z",
            41,
        )
        .expect("envelope conversion should succeed");

    assert_eq!(
        envelope.envelope.from,
        "kamn:did:agent:bridge-telegram-1".to_owned()
    );
    assert_eq!(
        envelope.envelope.to,
        vec!["kamn:did:agent:listener-target-1".to_owned()]
    );
}

#[test]
fn regression_rejects_replayed_telegram_inbound_projection_event() {
    // Regression: #438
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");
    let request = TelegramInboundRequest {
        listener_did: "kamn:did:agent:listener-1".to_owned(),
        inbound: inbound("kamn:did:agent:listener-target-1"),
    };

    engine
        .process_inbound_to_envelope(
            &request,
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-07T21:15:00Z",
            42,
        )
        .expect("first envelope projection should succeed");
    assert_eq!(
        engine.process_inbound_to_envelope(
            &request,
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-07T21:15:00Z",
            43,
        ),
        Err(TelegramBridgeError::Bridge(
            "duplicate inbound message id: telegram:tg-msg-1".to_owned()
        ))
    );
}

#[test]
fn route_target_mismatch_is_rejected() {
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    assert_eq!(
        engine.process_inbound(&TelegramInboundRequest {
            listener_did: "kamn:did:agent:listener-1".to_owned(),
            inbound: inbound("kamn:did:agent:different-target"),
        }),
        Err(TelegramBridgeError::RouteTargetMismatch {
            external_channel_id: "telegram:channel:ops".to_owned(),
            expected_target_did: "kamn:did:agent:listener-target-1".to_owned(),
            provided_target_did: "kamn:did:agent:different-target".to_owned(),
        })
    );
}

#[test]
fn regression_unauthorized_listener_is_rejected() {
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    // Regression: #223
    assert_eq!(
        engine.process_inbound(&TelegramInboundRequest {
            listener_did: "kamn:did:agent:listener-x".to_owned(),
            inbound: inbound("kamn:did:agent:listener-target-1"),
        }),
        Err(TelegramBridgeError::UnauthorizedListener(
            "kamn:did:agent:listener-x".to_owned()
        ))
    );
}
