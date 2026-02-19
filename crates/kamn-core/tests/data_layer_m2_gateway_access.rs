use kamn_core::{
    data_layer_m2_default_rls_policies, DataLayerM2AbacEngine, DataLayerM2AccessAuditInput,
    DataLayerM2AccessAuditLedger, DataLayerM2ActorRole, DataLayerM2AuthorizationDecision,
    DataLayerM2DidAuthRequest, DataLayerM2DidSessionService, DataLayerM2GatewayError,
    DataLayerM2MessageScope, DataLayerM2NegativeAuthorizationCase,
    DataLayerM2NegativeAuthorizationMatrixDecision,
    DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
    DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
    DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED, DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
    DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED, DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
    DATA_LAYER_M2_REQUESTER_DID_SETTING,
};

fn message_scope(escrow_id: Option<&str>) -> DataLayerM2MessageScope {
    DataLayerM2MessageScope {
        message_id: "msg-m2-1".to_owned(),
        sender_did: "kamn:did:agent:sender-1".to_owned(),
        recipient_did: "kamn:did:agent:recipient-1".to_owned(),
        owner_sender_did: "kamn:did:owner:sender".to_owned(),
        owner_recipient_did: "kamn:did:owner:recipient".to_owned(),
        escrow_id: escrow_id.map(str::to_owned),
    }
}

fn seeded_abac() -> DataLayerM2AbacEngine {
    let mut abac = DataLayerM2AbacEngine::new();
    abac.register_escrow_auditor("escrow-1", "kamn:did:auditor:1")
        .expect("auditor registration should pass");
    abac.set_escrow_dispute_active("escrow-1", true);
    abac
}

#[test]
fn spec_c01_did_authentication_issues_deterministic_bounded_session() {
    let service = DataLayerM2DidSessionService::new(900).expect("service should initialize");
    let token = service
        .authenticate(DataLayerM2DidAuthRequest {
            requester_did: "kamn:did:agent:sender-1".to_owned(),
            challenge: "nonce-123".to_owned(),
            credential: "sig:kamn:did:agent:sender-1:nonce-123".to_owned(),
            issued_at_epoch_seconds: 1_708_160_000,
            ttl_seconds: 300,
        })
        .expect("auth should issue session token");

    assert_eq!(token.requester_did, "kamn:did:agent:sender-1");
    assert_eq!(token.issued_at_epoch_seconds, 1_708_160_000);
    assert_eq!(token.expires_at_epoch_seconds, 1_708_160_300);
    assert!(token.token_id.starts_with("session:sha256:"));
}

#[test]
fn spec_c02_did_authentication_rejects_invalid_identity_or_credential_inputs() {
    let service = DataLayerM2DidSessionService::new(900).expect("service should initialize");

    let invalid_did = service.authenticate(DataLayerM2DidAuthRequest {
        requester_did: "did:example:not-kamn".to_owned(),
        challenge: "nonce-123".to_owned(),
        credential: "sig:did:example:not-kamn:nonce-123".to_owned(),
        issued_at_epoch_seconds: 1_708_160_000,
        ttl_seconds: 300,
    });
    assert!(matches!(
        invalid_did,
        Err(DataLayerM2GatewayError::InvalidDid(_))
    ));

    let invalid_credential = service.authenticate(DataLayerM2DidAuthRequest {
        requester_did: "kamn:did:agent:sender-1".to_owned(),
        challenge: "nonce-123".to_owned(),
        credential: "sig:tampered".to_owned(),
        issued_at_epoch_seconds: 1_708_160_000,
        ttl_seconds: 300,
    });
    assert_eq!(
        invalid_credential,
        Err(DataLayerM2GatewayError::InvalidCredential(
            "credential signature mismatch".to_owned()
        ))
    );
}

#[test]
fn spec_c02b_did_authentication_rejects_non_canonical_agent_did_shapes() {
    let service = DataLayerM2DidSessionService::new(900).expect("service should initialize");

    let uppercase_agent_segment = service.authenticate(DataLayerM2DidAuthRequest {
        requester_did: "kamn:did:agent:Sender-1".to_owned(),
        challenge: "nonce-123".to_owned(),
        credential: "sig:kamn:did:agent:Sender-1:nonce-123".to_owned(),
        issued_at_epoch_seconds: 1_708_160_000,
        ttl_seconds: 300,
    });
    assert!(matches!(
        uppercase_agent_segment,
        Err(DataLayerM2GatewayError::InvalidDid(_))
    ));
}

#[test]
fn spec_c03_abac_message_visibility_matrix_is_fail_closed_for_unrelated_requesters() {
    let abac = seeded_abac();

    let participant = abac
        .authorize_message_visibility(
            "kamn:did:agent:sender-1",
            DataLayerM2ActorRole::Agent,
            &message_scope(None),
        )
        .expect("participant authorization should evaluate");
    assert_eq!(
        participant,
        DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
        }
    );

    let owner = abac
        .authorize_message_visibility(
            "kamn:did:owner:recipient",
            DataLayerM2ActorRole::Owner,
            &message_scope(None),
        )
        .expect("owner authorization should evaluate");
    assert_eq!(
        owner,
        DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
        }
    );

    let auditor = abac
        .authorize_message_visibility(
            "kamn:did:auditor:1",
            DataLayerM2ActorRole::EscrowAuditor,
            &message_scope(Some("escrow-1")),
        )
        .expect("auditor authorization should evaluate");
    assert_eq!(
        auditor,
        DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED,
        }
    );

    let unrelated = abac
        .authorize_message_visibility(
            "kamn:did:agent:intruder-1",
            DataLayerM2ActorRole::Agent,
            &message_scope(Some("escrow-1")),
        )
        .expect("unrelated authorization should evaluate");
    assert_eq!(
        unrelated,
        DataLayerM2AuthorizationDecision::Deny {
            reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
        }
    );
}

#[test]
fn spec_c03b_abac_rejects_non_canonical_agent_did_fields() {
    let abac = seeded_abac();
    let mut invalid_scope = message_scope(None);
    invalid_scope.sender_did = "kamn:did:agent:Sender-1".to_owned();

    let invalid_sender = abac.authorize_message_visibility(
        "kamn:did:agent:recipient-1",
        DataLayerM2ActorRole::Agent,
        &invalid_scope,
    );
    assert!(matches!(
        invalid_sender,
        Err(DataLayerM2GatewayError::InvalidDid(_))
    ));

    let invalid_agent_requester = abac.authorize_message_visibility(
        "kamn:did:owner:sender",
        DataLayerM2ActorRole::Agent,
        &message_scope(None),
    );
    assert!(matches!(
        invalid_agent_requester,
        Err(DataLayerM2GatewayError::InvalidDid(_))
    ));
}

#[test]
fn spec_c04_negative_matrix_all_denied_emits_deterministic_fixtures() {
    let abac = seeded_abac();
    let cases = vec![
        DataLayerM2NegativeAuthorizationCase {
            case_id: "intruder".to_owned(),
            requester_did: "kamn:did:agent:intruder-1".to_owned(),
            requester_role: DataLayerM2ActorRole::Agent,
            scope: message_scope(Some("escrow-1")),
            expected_denied: true,
            event_epoch_seconds: 1_708_160_010,
        },
        DataLayerM2NegativeAuthorizationCase {
            case_id: "operator".to_owned(),
            requester_did: "kamn:did:platform:ops-1".to_owned(),
            requester_role: DataLayerM2ActorRole::PlatformOperator,
            scope: message_scope(None),
            expected_denied: true,
            event_epoch_seconds: 1_708_160_020,
        },
    ];

    let report = abac
        .evaluate_negative_authorization_matrix(&cases)
        .expect("negative matrix should evaluate");
    assert_eq!(
        report.decision,
        DataLayerM2NegativeAuthorizationMatrixDecision::AllDenied {
            reason_code: DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
        }
    );
    assert_eq!(report.fixtures.len(), 2);
    assert!(report.fixtures.iter().all(|fixture| fixture.denied));
    assert!(report.fixtures.iter().all(|fixture| !fixture.mismatch));
    assert!(report
        .fixtures
        .iter()
        .all(|fixture| fixture.decision_reason_code == DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED));
}

#[test]
fn spec_c05_negative_matrix_detects_unexpected_allow_drift() {
    let abac = seeded_abac();
    let cases = vec![
        DataLayerM2NegativeAuthorizationCase {
            case_id: "participant".to_owned(),
            requester_did: "kamn:did:agent:sender-1".to_owned(),
            requester_role: DataLayerM2ActorRole::Agent,
            scope: message_scope(None),
            expected_denied: true,
            event_epoch_seconds: 1_708_160_030,
        },
        DataLayerM2NegativeAuthorizationCase {
            case_id: "intruder".to_owned(),
            requester_did: "kamn:did:agent:intruder-1".to_owned(),
            requester_role: DataLayerM2ActorRole::Agent,
            scope: message_scope(None),
            expected_denied: true,
            event_epoch_seconds: 1_708_160_040,
        },
    ];

    let report = abac
        .evaluate_negative_authorization_matrix(&cases)
        .expect("negative matrix should evaluate");
    assert_eq!(
        report.decision,
        DataLayerM2NegativeAuthorizationMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
        }
    );
    assert_eq!(report.fixtures.len(), 2);
    assert!(report.fixtures[0].mismatch);
    assert!(!report.fixtures[1].mismatch);
}

#[test]
fn spec_c06_negative_matrix_fails_closed_for_invalid_inputs() {
    let abac = seeded_abac();

    let empty_cases = abac.evaluate_negative_authorization_matrix(&[]);
    assert_eq!(
        empty_cases,
        Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
            "cases",
        ))
    );

    let zero_timestamp =
        abac.evaluate_negative_authorization_matrix(&[DataLayerM2NegativeAuthorizationCase {
            case_id: "invalid-ts".to_owned(),
            requester_did: "kamn:did:agent:intruder-1".to_owned(),
            requester_role: DataLayerM2ActorRole::Agent,
            scope: message_scope(None),
            expected_denied: true,
            event_epoch_seconds: 0,
        }]);
    assert_eq!(
        zero_timestamp,
        Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
            "event_epoch_seconds",
        ))
    );
}

#[test]
fn spec_c04_rls_policies_include_requester_guard_and_fail_closed_predicates() {
    let policies = data_layer_m2_default_rls_policies();
    assert!(policies.iter().any(|policy| policy.table_name == "messages"
        && policy
            .using_clause
            .contains(DATA_LAYER_M2_REQUESTER_DID_SETTING)));
    assert!(policies.iter().all(|policy| policy
        .using_clause
        .contains("current_setting('kamn.requester_did', true) <> ''")));
}

#[test]
fn spec_c05_access_audit_log_hash_chain_detects_tamper() {
    let mut ledger = DataLayerM2AccessAuditLedger::new();
    ledger
        .append(DataLayerM2AccessAuditInput {
            requester_did: "kamn:did:agent:sender-1".to_owned(),
            action: "read_message".to_owned(),
            resource_id: "msg-m2-1".to_owned(),
            reason_code: DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED.to_owned(),
            event_epoch_seconds: 1_708_160_000,
        })
        .expect("first audit append should succeed");
    ledger
        .append(DataLayerM2AccessAuditInput {
            requester_did: "kamn:did:owner:sender".to_owned(),
            action: "read_message".to_owned(),
            resource_id: "msg-m2-1".to_owned(),
            reason_code: DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED.to_owned(),
            event_epoch_seconds: 1_708_160_100,
        })
        .expect("second audit append should succeed");

    ledger
        .verify_hash_chain()
        .expect("untampered audit chain should verify");
    ledger
        .replace_record_hash_unchecked(1, "sha256:tampered")
        .expect("test tamper helper should succeed");

    let verify = ledger.verify_hash_chain();
    assert!(matches!(
        verify,
        Err(DataLayerM2GatewayError::InvalidAuditHashChain { .. })
    ));
}
