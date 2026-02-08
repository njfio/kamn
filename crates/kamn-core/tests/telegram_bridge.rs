use kamn_core::{
    BridgeInboundEnvelope, TelegramBridgeConfig, TelegramBridgeEngine, TelegramBridgeError,
    TelegramInboundRequest,
};
use std::collections::{BTreeMap, BTreeSet};

const VALID_WEBHOOK_TOKEN: &str = "telegram-webhook-token-valid";

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
        webhook_token: VALID_WEBHOOK_TOKEN.to_owned(),
        channel_routes,
    }
}

fn inbound(target: &str, external_message_id: &str) -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: external_message_id.to_owned(),
        external_sender_id: "tg-user-9".to_owned(),
        external_channel_id: "telegram:channel:ops".to_owned(),
        target_agent_did: target.to_owned(),
        body: "run diagnostics".to_owned(),
        received_at: "2026-02-07T21:00:00Z".to_owned(),
        received_at_unix: 1_707_340_800,
    }
}

fn inbound_request(
    target: &str,
    external_message_id: &str,
    webhook_token: &str,
    checkpoint: u64,
) -> TelegramInboundRequest {
    TelegramInboundRequest {
        listener_did: "kamn:did:agent:listener-1".to_owned(),
        webhook_token: webhook_token.to_owned(),
        checkpoint,
        observed_at_unix: 1_707_340_900,
        inbound: inbound(target, external_message_id),
    }
}

#[test]
fn authorized_listener_can_process_inbound() {
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    let normalized = engine
        .process_inbound(&inbound_request(
            "kamn:did:agent:listener-target-1",
            "tg-msg-1",
            VALID_WEBHOOK_TOKEN,
            41,
        ))
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
            &inbound_request(
                "kamn:did:agent:listener-target-1",
                "tg-msg-2",
                VALID_WEBHOOK_TOKEN,
                42,
            ),
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
    let request = inbound_request(
        "kamn:did:agent:listener-target-1",
        "tg-msg-replay",
        VALID_WEBHOOK_TOKEN,
        50,
    );

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
            &TelegramInboundRequest {
                checkpoint: 51,
                ..request.clone()
            },
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-07T21:15:00Z",
            43,
        ),
        Err(TelegramBridgeError::Bridge(
            "duplicate inbound message id: telegram:tg-msg-replay".to_owned()
        ))
    );
}

#[test]
fn route_target_mismatch_is_rejected() {
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    assert_eq!(
        engine.process_inbound(&inbound_request(
            "kamn:did:agent:different-target",
            "tg-msg-3",
            VALID_WEBHOOK_TOKEN,
            60,
        )),
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
            webhook_token: VALID_WEBHOOK_TOKEN.to_owned(),
            checkpoint: 70,
            observed_at_unix: 1_707_340_900,
            inbound: inbound("kamn:did:agent:listener-target-1", "tg-msg-4"),
        }),
        Err(TelegramBridgeError::UnauthorizedListener(
            "kamn:did:agent:listener-x".to_owned()
        ))
    );
}

#[test]
fn regression_rejects_forged_webhook_token() {
    // Regression: #621
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");
    assert_eq!(
        engine.process_inbound(&inbound_request(
            "kamn:did:agent:listener-target-1",
            "tg-msg-5",
            "forged-token",
            80,
        )),
        Err(TelegramBridgeError::InvalidWebhookToken)
    );
}

#[test]
fn regression_rejects_replayed_or_out_of_order_checkpoint() {
    // Regression: #621
    let engine = TelegramBridgeEngine::new(config()).expect("engine should build");

    engine
        .process_inbound(&inbound_request(
            "kamn:did:agent:listener-target-1",
            "tg-msg-6",
            VALID_WEBHOOK_TOKEN,
            100,
        ))
        .expect("initial checkpoint should succeed");

    assert_eq!(
        engine.process_inbound(&inbound_request(
            "kamn:did:agent:listener-target-1",
            "tg-msg-7",
            VALID_WEBHOOK_TOKEN,
            100,
        )),
        Err(TelegramBridgeError::NonMonotonicCheckpoint {
            external_channel_id: "telegram:channel:ops".to_owned(),
            last_checkpoint: 100,
            received_checkpoint: 100,
        })
    );

    assert_eq!(
        engine.process_inbound(&inbound_request(
            "kamn:did:agent:listener-target-1",
            "tg-msg-8",
            VALID_WEBHOOK_TOKEN,
            99,
        )),
        Err(TelegramBridgeError::NonMonotonicCheckpoint {
            external_channel_id: "telegram:channel:ops".to_owned(),
            last_checkpoint: 100,
            received_checkpoint: 99,
        })
    );
}
