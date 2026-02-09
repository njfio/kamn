use kamn_core::{
    BridgeInboundEnvelope, BridgePlatform, CanonicalMessageEnvelope, DiscordBridgeConfig,
    DiscordBridgeEngine, DiscordBridgeError, DiscordInboundRequest, TelegramBridgeConfig,
    TelegramBridgeEngine, TelegramBridgeError, TelegramInboundRequest,
};
use std::collections::{BTreeMap, BTreeSet};

const TELEGRAM_BRIDGE_DID: &str = "kamn:did:agent:bridge-telegram-1";
const TELEGRAM_LISTENER_DID: &str = "kamn:did:agent:listener-telegram-1";
const TELEGRAM_TARGET_DID: &str = "kamn:did:agent:listener-target-telegram-1";
const TELEGRAM_CHANNEL_ID: &str = "telegram:channel:ops";
const TELEGRAM_WEBHOOK_TOKEN: &str = "telegram-relay-contract-token-valid";

const DISCORD_BRIDGE_DID: &str = "kamn:did:agent:bridge-discord-1";
const DISCORD_LISTENER_DID: &str = "kamn:did:agent:listener-discord-1";
const DISCORD_TARGET_DID: &str = "kamn:did:agent:listener-target-discord-1";
const DISCORD_CHANNEL_ID: &str = "discord:channel:ops";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngressPlatformCase {
    Telegram,
    Discord,
}

impl IngressPlatformCase {
    fn label(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Discord => "discord",
        }
    }

    fn bridge_agent_did(self) -> &'static str {
        match self {
            Self::Telegram => TELEGRAM_BRIDGE_DID,
            Self::Discord => DISCORD_BRIDGE_DID,
        }
    }

    fn bridge_platform(self) -> BridgePlatform {
        match self {
            Self::Telegram => BridgePlatform::Telegram,
            Self::Discord => BridgePlatform::Discord,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ValidIngressFixtureCase {
    case_id: &'static str,
    platform: IngressPlatformCase,
    external_message_id: &'static str,
    external_sender_id: &'static str,
    external_channel_id: &'static str,
    target_agent_did: &'static str,
    body: &'static str,
    received_at: &'static str,
    received_at_unix: u64,
    observed_at_unix: u64,
    expires: &'static str,
    nonce: u64,
    checkpoint: u64,
    recipient_key: &'static str,
}

const VALID_INGRESS_FIXTURE_MATRIX: [ValidIngressFixtureCase; 2] = [
    ValidIngressFixtureCase {
        case_id: "telegram-valid-inbound-relay",
        platform: IngressPlatformCase::Telegram,
        external_message_id: "tg-ingress-contract-1",
        external_sender_id: "tg-user-contract-1",
        external_channel_id: TELEGRAM_CHANNEL_ID,
        target_agent_did: TELEGRAM_TARGET_DID,
        body: "relay inbound diagnostics",
        received_at: "2026-02-09T09:00:00Z",
        received_at_unix: 1_707_469_200,
        observed_at_unix: 1_707_469_260,
        expires: "2026-02-09T09:30:00Z",
        nonce: 301,
        checkpoint: 701,
        recipient_key: "kamn:did:agent:listener-target-telegram-1#key-agreement-1",
    },
    ValidIngressFixtureCase {
        case_id: "discord-valid-inbound-relay",
        platform: IngressPlatformCase::Discord,
        external_message_id: "discord-ingress-contract-1",
        external_sender_id: "discord-user-contract-1",
        external_channel_id: DISCORD_CHANNEL_ID,
        target_agent_did: DISCORD_TARGET_DID,
        body: "relay inbound triage",
        received_at: "2026-02-09T09:05:00Z",
        received_at_unix: 1_707_469_500,
        observed_at_unix: 1_707_469_540,
        expires: "2026-02-09T09:35:00Z",
        nonce: 302,
        checkpoint: 0,
        recipient_key: "kamn:did:agent:listener-target-discord-1#key-agreement-1",
    },
];

const MALFORMED_INGRESS_CLASS_MATRIX: [&str; 4] = [
    "telegram-forged-webhook-token",
    "telegram-empty-body",
    "discord-unauthorized-listener",
    "discord-route-target-mismatch",
];

fn telegram_config() -> TelegramBridgeConfig {
    let mut channel_routes = BTreeMap::new();
    channel_routes.insert(
        TELEGRAM_CHANNEL_ID.to_owned(),
        TELEGRAM_TARGET_DID.to_owned(),
    );

    TelegramBridgeConfig {
        bridge_agent_did: TELEGRAM_BRIDGE_DID.to_owned(),
        authorized_listener_dids: [TELEGRAM_LISTENER_DID.to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        webhook_token: TELEGRAM_WEBHOOK_TOKEN.to_owned(),
        channel_routes,
    }
}

fn discord_config() -> DiscordBridgeConfig {
    let mut channel_routes = BTreeMap::new();
    channel_routes.insert(DISCORD_CHANNEL_ID.to_owned(), DISCORD_TARGET_DID.to_owned());

    DiscordBridgeConfig {
        bridge_agent_did: DISCORD_BRIDGE_DID.to_owned(),
        authorized_listener_dids: [DISCORD_LISTENER_DID.to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        authorized_approver_dids: [
            "kamn:did:agent:approver-discord-1".to_owned(),
            "kamn:did:agent:approver-discord-2".to_owned(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>(),
        required_approvals: 2,
        channel_routes,
    }
}

fn inbound_envelope(case: &ValidIngressFixtureCase) -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: case.external_message_id.to_owned(),
        external_sender_id: case.external_sender_id.to_owned(),
        external_channel_id: case.external_channel_id.to_owned(),
        target_agent_did: case.target_agent_did.to_owned(),
        body: case.body.to_owned(),
        received_at: case.received_at.to_owned(),
        received_at_unix: case.received_at_unix,
    }
}

fn project_inbound_to_envelope(case: &ValidIngressFixtureCase) -> CanonicalMessageEnvelope {
    match case.platform {
        IngressPlatformCase::Telegram => {
            let engine = TelegramBridgeEngine::new(telegram_config()).expect("engine should build");
            engine
                .process_inbound_to_envelope(
                    &TelegramInboundRequest {
                        listener_did: TELEGRAM_LISTENER_DID.to_owned(),
                        webhook_token: TELEGRAM_WEBHOOK_TOKEN.to_owned(),
                        checkpoint: case.checkpoint,
                        observed_at_unix: case.observed_at_unix,
                        inbound: inbound_envelope(case),
                    },
                    vec![case.recipient_key.to_owned()],
                    case.expires,
                    case.nonce,
                )
                .expect("telegram inbound fixture should project into canonical envelope")
        }
        IngressPlatformCase::Discord => {
            let engine = DiscordBridgeEngine::new(discord_config()).expect("engine should build");
            engine
                .process_inbound_to_envelope(
                    &DiscordInboundRequest {
                        listener_did: DISCORD_LISTENER_DID.to_owned(),
                        observed_at_unix: case.observed_at_unix,
                        inbound: inbound_envelope(case),
                    },
                    vec![case.recipient_key.to_owned()],
                    case.expires,
                    case.nonce,
                )
                .expect("discord inbound fixture should project into canonical envelope")
        }
    }
}

#[test]
fn unit_ingress_fixture_matrix_covers_valid_and_malformed_classes() {
    assert_eq!(VALID_INGRESS_FIXTURE_MATRIX.len(), 2);
    assert_eq!(MALFORMED_INGRESS_CLASS_MATRIX.len(), 4);
    assert!(VALID_INGRESS_FIXTURE_MATRIX
        .iter()
        .any(|case| case.platform == IngressPlatformCase::Telegram));
    assert!(VALID_INGRESS_FIXTURE_MATRIX
        .iter()
        .any(|case| case.platform == IngressPlatformCase::Discord));
    assert!(MALFORMED_INGRESS_CLASS_MATRIX.contains(&"telegram-forged-webhook-token"));
    assert!(MALFORMED_INGRESS_CLASS_MATRIX.contains(&"discord-route-target-mismatch"));
}

#[test]
fn functional_ingress_fixture_matrix_projects_deterministic_envelopes() {
    for case in VALID_INGRESS_FIXTURE_MATRIX {
        let first = project_inbound_to_envelope(&case);
        let second = project_inbound_to_envelope(&case);

        assert_eq!(
            first, second,
            "fixture {} must be deterministic",
            case.case_id
        );

        let expected_message_id = format!("{}:{}", case.platform.label(), case.external_message_id);
        assert_eq!(first.envelope.id, expected_message_id);
        assert_eq!(
            first.envelope.from,
            case.platform.bridge_agent_did().to_owned(),
            "fixture {} must preserve bridge DID sender binding",
            case.case_id
        );
        assert_eq!(
            first.envelope.to,
            vec![case.target_agent_did.to_owned()],
            "fixture {} must preserve target DID binding",
            case.case_id
        );
        assert_eq!(
            first.proof.verification_method,
            format!("{}#bridge-key-1", case.platform.bridge_agent_did()),
            "fixture {} must preserve verification method binding",
            case.case_id
        );
        assert_eq!(
            first.proof.proof_value,
            format!("proof:{}", first.envelope.id),
            "fixture {} must preserve proof/message-id binding",
            case.case_id
        );
        assert_eq!(
            first.body.get("message").map(String::as_str),
            Some(case.body),
            "fixture {} must preserve body payload",
            case.case_id
        );
        assert_eq!(
            first.body.get("external_sender").map(String::as_str),
            Some(case.external_sender_id),
            "fixture {} must preserve sender handle payload binding",
            case.case_id
        );
        assert_eq!(
            first.body.get("external_channel").map(String::as_str),
            Some(case.external_channel_id),
            "fixture {} must preserve source channel payload binding",
            case.case_id
        );
        assert_eq!(
            first.body.get("platform").map(String::as_str),
            Some(case.platform.label()),
            "fixture {} must preserve platform payload binding",
            case.case_id
        );
    }
}

#[test]
fn integration_normalizes_telegram_and_discord_ingress_fixtures() {
    for case in VALID_INGRESS_FIXTURE_MATRIX {
        let expected_bridge_message_id =
            format!("{}:{}", case.platform.label(), case.external_message_id);
        match case.platform {
            IngressPlatformCase::Telegram => {
                let engine =
                    TelegramBridgeEngine::new(telegram_config()).expect("engine should build");
                let normalized = engine
                    .process_inbound(&TelegramInboundRequest {
                        listener_did: TELEGRAM_LISTENER_DID.to_owned(),
                        webhook_token: TELEGRAM_WEBHOOK_TOKEN.to_owned(),
                        checkpoint: case.checkpoint,
                        observed_at_unix: case.observed_at_unix,
                        inbound: inbound_envelope(&case),
                    })
                    .expect("telegram inbound fixture should normalize");
                assert_eq!(normalized.bridge_message_id, expected_bridge_message_id);
                assert_eq!(normalized.platform, case.platform.bridge_platform());
            }
            IngressPlatformCase::Discord => {
                let engine =
                    DiscordBridgeEngine::new(discord_config()).expect("engine should build");
                let normalized = engine
                    .process_inbound(&DiscordInboundRequest {
                        listener_did: DISCORD_LISTENER_DID.to_owned(),
                        observed_at_unix: case.observed_at_unix,
                        inbound: inbound_envelope(&case),
                    })
                    .expect("discord inbound fixture should normalize");
                assert_eq!(normalized.bridge_message_id, expected_bridge_message_id);
                assert_eq!(normalized.platform, case.platform.bridge_platform());
            }
        }
    }
}

#[test]
fn regression_rejects_malformed_and_replayed_ingress_payloads() {
    // Regression: #850
    let telegram_case = VALID_INGRESS_FIXTURE_MATRIX
        .iter()
        .find(|case| case.platform == IngressPlatformCase::Telegram)
        .expect("telegram fixture case must exist");
    let telegram_engine =
        TelegramBridgeEngine::new(telegram_config()).expect("engine should build");
    let request = TelegramInboundRequest {
        listener_did: TELEGRAM_LISTENER_DID.to_owned(),
        webhook_token: TELEGRAM_WEBHOOK_TOKEN.to_owned(),
        checkpoint: telegram_case.checkpoint,
        observed_at_unix: telegram_case.observed_at_unix,
        inbound: inbound_envelope(telegram_case),
    };

    telegram_engine
        .process_inbound_to_envelope(
            &request,
            vec![telegram_case.recipient_key.to_owned()],
            telegram_case.expires,
            telegram_case.nonce,
        )
        .expect("first projection should succeed");
    assert_eq!(
        telegram_engine.process_inbound_to_envelope(
            &TelegramInboundRequest {
                checkpoint: request.checkpoint + 1,
                ..request.clone()
            },
            vec![telegram_case.recipient_key.to_owned()],
            telegram_case.expires,
            telegram_case.nonce + 1,
        ),
        Err(TelegramBridgeError::Bridge(format!(
            "duplicate inbound message id: telegram:{}",
            telegram_case.external_message_id
        )))
    );

    let forged_token_engine =
        TelegramBridgeEngine::new(telegram_config()).expect("engine should build");
    assert_eq!(
        forged_token_engine.process_inbound(&TelegramInboundRequest {
            webhook_token: "forged-token".to_owned(),
            ..request.clone()
        }),
        Err(TelegramBridgeError::InvalidWebhookToken)
    );

    let empty_body_engine =
        TelegramBridgeEngine::new(telegram_config()).expect("engine should build");
    assert_eq!(
        empty_body_engine.process_inbound(&TelegramInboundRequest {
            checkpoint: request.checkpoint + 3,
            inbound: BridgeInboundEnvelope {
                external_message_id: "tg-ingress-malformed-empty-body".to_owned(),
                body: String::new(),
                ..request.inbound.clone()
            },
            ..request.clone()
        }),
        Err(TelegramBridgeError::Bridge(
            "field must not be empty: bridge_inbound_envelope.body".to_owned()
        ))
    );

    let discord_engine = DiscordBridgeEngine::new(discord_config()).expect("engine should build");
    let discord_case = VALID_INGRESS_FIXTURE_MATRIX
        .iter()
        .find(|case| case.platform == IngressPlatformCase::Discord)
        .expect("discord fixture case must exist");
    let discord_request = DiscordInboundRequest {
        listener_did: DISCORD_LISTENER_DID.to_owned(),
        observed_at_unix: discord_case.observed_at_unix,
        inbound: inbound_envelope(discord_case),
    };
    assert_eq!(
        discord_engine.process_inbound(&DiscordInboundRequest {
            listener_did: "kamn:did:agent:listener-discord-x".to_owned(),
            ..discord_request.clone()
        }),
        Err(DiscordBridgeError::UnauthorizedListener(
            "kamn:did:agent:listener-discord-x".to_owned()
        ))
    );
    assert_eq!(
        discord_engine.process_inbound(&DiscordInboundRequest {
            inbound: BridgeInboundEnvelope {
                target_agent_did: "kamn:did:agent:listener-target-discord-x".to_owned(),
                ..discord_request.inbound.clone()
            },
            ..discord_request.clone()
        }),
        Err(DiscordBridgeError::RouteTargetMismatch {
            external_channel_id: DISCORD_CHANNEL_ID.to_owned(),
            expected_target_did: DISCORD_TARGET_DID.to_owned(),
            provided_target_did: "kamn:did:agent:listener-target-discord-x".to_owned(),
        })
    );
}
