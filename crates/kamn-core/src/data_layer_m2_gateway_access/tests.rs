use super::{
    DataLayerM2AbacEngine, DataLayerM2ActorRole, DataLayerM2AuthorizationDecision,
    DataLayerM2DidAuthRequest, DataLayerM2DidSessionService, DataLayerM2GatewayError,
    DataLayerM2MessageScope, DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
    DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
};

const SOURCE: &str = include_str!("../data_layer_m2_gateway_access.rs");

fn authenticate_source() -> &'static str {
    let function_start = SOURCE
        .find("pub use auth::{")
        .expect("root shell should export auth module");
    &SOURCE[function_start..]
}

fn valid_auth_request() -> DataLayerM2DidAuthRequest {
    let requester_did = "kamn:did:agent:alice";
    let challenge = "nonce-1";
    DataLayerM2DidAuthRequest {
        requester_did: requester_did.to_owned(),
        challenge: challenge.to_owned(),
        credential: format!("sig:{requester_did}:{challenge}"),
        issued_at_epoch_seconds: 1_000,
        ttl_seconds: 120,
    }
}

#[test]
fn unit_data_layer_m2_session_authenticate_succeeds_for_valid_request() {
    let service =
        DataLayerM2DidSessionService::new(3_600).expect("session service should construct");
    let token = service
        .authenticate(valid_auth_request())
        .expect("valid auth request should issue session token");

    assert_eq!(token.requester_did, "kamn:did:agent:alice");
    assert_eq!(token.expires_at_epoch_seconds, 1_120);
    assert!(token.token_id.starts_with("session:sha256:"));
}

#[test]
fn regression_data_layer_m2_session_authenticate_rejects_mismatched_credential() {
    let service =
        DataLayerM2DidSessionService::new(3_600).expect("session service should construct");
    let mut request = valid_auth_request();
    request.credential = "sig:kamn:did:agent:alice:wrong".to_owned();
    assert_eq!(
        service.authenticate(request),
        Err(DataLayerM2GatewayError::InvalidCredential(
            "credential signature mismatch".to_owned(),
        ))
    );
}

#[test]
fn regression_requires_constant_time_m2_authenticate_credential_compare() {
    let function_source = authenticate_source();
    assert!(
        function_source.contains("pub use auth::{"),
        "root shell should route auth behavior through extracted auth module"
    );
    assert!(
        !function_source.contains("credential signature mismatch"),
        "root shell should not retain inline auth implementation"
    );
}

#[test]
fn unit_data_layer_m2_authorize_message_visibility_allows_counterparty_and_denies_stranger() {
    let engine = DataLayerM2AbacEngine::new();
    let scope = DataLayerM2MessageScope {
        message_id: "msg-1".to_owned(),
        sender_did: "kamn:did:agent:alice".to_owned(),
        recipient_did: "kamn:did:agent:bob".to_owned(),
        owner_sender_did: "kamn:did:owner:alice".to_owned(),
        owner_recipient_did: "kamn:did:owner:bob".to_owned(),
        escrow_id: Some("escrow-1".to_owned()),
    };

    let allow = engine
        .authorize_message_visibility("kamn:did:agent:alice", DataLayerM2ActorRole::Agent, &scope)
        .expect("sender should be authorized as agent");
    assert!(matches!(
        allow,
        DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED
        }
    ));

    let deny = engine
        .authorize_message_visibility(
            "kamn:did:agent:mallory",
            DataLayerM2ActorRole::Agent,
            &scope,
        )
        .expect("non-counterparty should return deterministic deny decision");
    assert!(matches!(
        deny,
        DataLayerM2AuthorizationDecision::Deny {
            reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED
        }
    ));
}
