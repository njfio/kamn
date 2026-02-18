use kamn_core::{
    DataLayerM4EscrowDraftInput, DataLayerM4EscrowState, DataLayerM4EscrowTransitionAction,
    DataLayerM4EscrowTransitionEngine, DataLayerM4EscrowVisibilityDecision,
    DataLayerM4EscrowVisibilityRequest, DataLayerM4SettlementEvidenceInput,
    DataLayerM4SettlementEvidenceRegistry, DataLayerM4SettlementEvidenceRegistryError,
};

fn draft_input(escrow_id: &str) -> DataLayerM4EscrowDraftInput {
    DataLayerM4EscrowDraftInput {
        escrow_id: escrow_id.to_owned(),
        initiator_did: "kamn:did:agent:init-1".to_owned(),
        counterparty_did: "kamn:did:agent:cp-1".to_owned(),
        auditor_did: Some("kamn:did:auditor:escrow-1".to_owned()),
        auditor_threshold: Some(2),
        auditor_share_holders: vec![
            "kamn:did:holder:h1".to_owned(),
            "kamn:did:holder:h2".to_owned(),
            "kamn:did:holder:h3".to_owned(),
        ],
        expires_at_epoch_seconds: Some(1_708_200_000),
    }
}

#[test]
fn spec_c01_escrow_state_machine_accepts_valid_transition_sequence() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    engine
        .create_escrow(draft_input("escrow-m4-1"))
        .expect("escrow draft should initialize");

    let funded = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: 1_708_160_000,
            },
        )
        .expect("fund transition should succeed");
    assert_eq!(funded.reason_code, "m4_escrow_funded");

    let activated = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds: 1_708_160_005,
            },
        )
        .expect("activate transition should succeed");
    assert_eq!(activated.reason_code, "m4_escrow_active");

    let disputed = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::OpenDispute {
                dispute_opened_at_epoch_seconds: 1_708_160_010,
            },
        )
        .expect("dispute transition should succeed");
    assert_eq!(disputed.reason_code, "m4_escrow_disputed");

    let released = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::ResolveRelease {
                settled_at_epoch_seconds: 1_708_160_020,
                settlement_receipt_hash: "sha256:receipt-r1".to_owned(),
            },
        )
        .expect("release resolution should succeed");
    assert_eq!(released.reason_code, "m4_escrow_released");

    let escrow = engine
        .escrow("escrow-m4-1")
        .expect("escrow should still exist");
    assert_eq!(escrow.state, DataLayerM4EscrowState::Released);
}

#[test]
fn spec_c02_invalid_transition_paths_fail_closed() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    engine
        .create_escrow(draft_input("escrow-m4-2"))
        .expect("escrow draft should initialize");

    let invalid = engine.apply_transition(
        "escrow-m4-2",
        DataLayerM4EscrowTransitionAction::Activate {
            activated_at_epoch_seconds: 1_708_160_005,
        },
    );
    assert!(matches!(
        invalid,
        Err(DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowTransition { .. })
    ));
}

#[test]
fn spec_c03_scoped_message_visibility_enforces_participant_and_threshold_rules() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    engine
        .create_escrow(draft_input("escrow-m4-3"))
        .expect("escrow draft should initialize");
    engine
        .apply_transition(
            "escrow-m4-3",
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: 1_708_160_000,
            },
        )
        .expect("fund transition should succeed");
    engine
        .apply_transition(
            "escrow-m4-3",
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds: 1_708_160_010,
            },
        )
        .expect("activate transition should succeed");
    engine
        .apply_transition(
            "escrow-m4-3",
            DataLayerM4EscrowTransitionAction::OpenDispute {
                dispute_opened_at_epoch_seconds: 1_708_160_020,
            },
        )
        .expect("dispute transition should succeed");

    let initiator = engine
        .authorize_message_visibility(DataLayerM4EscrowVisibilityRequest {
            escrow_id: "escrow-m4-3".to_owned(),
            requester_did: "kamn:did:agent:init-1".to_owned(),
            reconstructed_auditor_shares: None,
        })
        .expect("initiator visibility should evaluate");
    assert!(matches!(
        initiator,
        DataLayerM4EscrowVisibilityDecision::Allow { .. }
    ));

    let auditor_denied = engine
        .authorize_message_visibility(DataLayerM4EscrowVisibilityRequest {
            escrow_id: "escrow-m4-3".to_owned(),
            requester_did: "kamn:did:auditor:escrow-1".to_owned(),
            reconstructed_auditor_shares: Some(1),
        })
        .expect("auditor visibility should evaluate");
    assert_eq!(
        auditor_denied,
        DataLayerM4EscrowVisibilityDecision::Deny {
            reason_code: "m4_escrow_auditor_threshold_not_met"
        }
    );

    let auditor_allowed = engine
        .authorize_message_visibility(DataLayerM4EscrowVisibilityRequest {
            escrow_id: "escrow-m4-3".to_owned(),
            requester_did: "kamn:did:auditor:escrow-1".to_owned(),
            reconstructed_auditor_shares: Some(2),
        })
        .expect("auditor visibility should evaluate");
    assert_eq!(
        auditor_allowed,
        DataLayerM4EscrowVisibilityDecision::Allow {
            reason_code: "m4_escrow_auditor_scope_allowed"
        }
    );

    let intruder = engine
        .authorize_message_visibility(DataLayerM4EscrowVisibilityRequest {
            escrow_id: "escrow-m4-3".to_owned(),
            requester_did: "kamn:did:agent:intruder-1".to_owned(),
            reconstructed_auditor_shares: None,
        })
        .expect("intruder visibility should evaluate");
    assert_eq!(
        intruder,
        DataLayerM4EscrowVisibilityDecision::Deny {
            reason_code: "m4_escrow_scope_denied"
        }
    );
}

#[test]
fn spec_c04_settlement_evidence_append_is_deterministic_for_final_states() {
    let mut registry_a = DataLayerM4SettlementEvidenceRegistry::new();
    let mut registry_b = DataLayerM4SettlementEvidenceRegistry::new();
    let input = DataLayerM4SettlementEvidenceInput {
        escrow_id: "escrow-m4-4".to_owned(),
        escrow_state: DataLayerM4EscrowState::Released,
        settlement_receipt_hash: "sha256:receipt-a".to_owned(),
        settlement_payload_hash: "sha256:payload-a".to_owned(),
        recorded_at_epoch_seconds: 1_708_160_100,
    };

    let record_a = registry_a
        .append(input.clone())
        .expect("settlement evidence append should succeed");
    let record_b = registry_b
        .append(input)
        .expect("settlement evidence append should succeed");

    assert_eq!(record_a.record_hash, record_b.record_hash);
    assert!(record_a.record_hash.starts_with("sha256:"));
}

#[test]
fn spec_c05_settlement_evidence_hash_chain_detects_tamper() {
    let mut registry = DataLayerM4SettlementEvidenceRegistry::new();
    registry
        .append(DataLayerM4SettlementEvidenceInput {
            escrow_id: "escrow-m4-5".to_owned(),
            escrow_state: DataLayerM4EscrowState::Released,
            settlement_receipt_hash: "sha256:receipt-a".to_owned(),
            settlement_payload_hash: "sha256:payload-a".to_owned(),
            recorded_at_epoch_seconds: 1_708_160_100,
        })
        .expect("first append should succeed");
    registry
        .append(DataLayerM4SettlementEvidenceInput {
            escrow_id: "escrow-m4-5".to_owned(),
            escrow_state: DataLayerM4EscrowState::Refunded,
            settlement_receipt_hash: "sha256:receipt-b".to_owned(),
            settlement_payload_hash: "sha256:payload-b".to_owned(),
            recorded_at_epoch_seconds: 1_708_160_200,
        })
        .expect("second append should succeed");

    registry
        .verify_escrow_integrity("escrow-m4-5")
        .expect("untampered evidence chain should pass");
    registry
        .replace_record_hash_unchecked("escrow-m4-5", 1, "sha256:tampered")
        .expect("tamper helper should succeed");

    let verify = registry.verify_escrow_integrity("escrow-m4-5");
    assert!(matches!(
        verify,
        Err(DataLayerM4SettlementEvidenceRegistryError::InvalidEvidenceHashChain { .. })
    ));
}
