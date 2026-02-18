use kamn_core::{
    DataLayerM4EscrowDraftInput, DataLayerM4EscrowInteropError, DataLayerM4EscrowState,
    DataLayerM4EscrowTransitionAction, DataLayerM4EscrowTransitionEngine,
    DataLayerM4EscrowVisibilityDecision, DataLayerM4EscrowVisibilityRequest,
    DataLayerM4SettlementEvidenceInput, DataLayerM4SettlementEvidenceReconciliationDecision,
    DataLayerM4SettlementEvidenceRegistry, DataLayerM4SettlementEvidenceRegistryError,
    EscrowStatus, DATA_LAYER_M4_ESCROW_ACTIVE_REASON_CODE,
    DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_MET_REASON_CODE,
    DATA_LAYER_M4_ESCROW_DISPUTED_REASON_CODE, DATA_LAYER_M4_ESCROW_FUNDED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_RELEASED_REASON_CODE, DATA_LAYER_M4_ESCROW_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MISMATCH_REASON_CODE,
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
    assert_eq!(funded.reason_code, DATA_LAYER_M4_ESCROW_FUNDED_REASON_CODE);

    let activated = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds: 1_708_160_005,
            },
        )
        .expect("activate transition should succeed");
    assert_eq!(
        activated.reason_code,
        DATA_LAYER_M4_ESCROW_ACTIVE_REASON_CODE
    );

    let disputed = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::OpenDispute {
                dispute_opened_at_epoch_seconds: 1_708_160_010,
            },
        )
        .expect("dispute transition should succeed");
    assert_eq!(
        disputed.reason_code,
        DATA_LAYER_M4_ESCROW_DISPUTED_REASON_CODE
    );

    let released = engine
        .apply_transition(
            "escrow-m4-1",
            DataLayerM4EscrowTransitionAction::ResolveRelease {
                settled_at_epoch_seconds: 1_708_160_020,
                settlement_receipt_hash: "sha256:receipt-r1".to_owned(),
            },
        )
        .expect("release resolution should succeed");
    assert_eq!(
        released.reason_code,
        DATA_LAYER_M4_ESCROW_RELEASED_REASON_CODE
    );

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
            reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_MET_REASON_CODE
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
            reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE
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
            reason_code: DATA_LAYER_M4_ESCROW_SCOPE_DENIED_REASON_CODE
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

#[test]
fn spec_c06_settlement_evidence_reconciliation_matches_terminal_escrow_projection() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    let mut registry = DataLayerM4SettlementEvidenceRegistry::new();
    engine
        .create_escrow(draft_input("escrow-m4-6"))
        .expect("escrow draft should initialize");
    engine
        .apply_transition(
            "escrow-m4-6",
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: 1_708_160_000,
            },
        )
        .expect("fund transition should succeed");
    engine
        .apply_transition(
            "escrow-m4-6",
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds: 1_708_160_010,
            },
        )
        .expect("activate transition should succeed");
    engine
        .apply_transition(
            "escrow-m4-6",
            DataLayerM4EscrowTransitionAction::ResolveRelease {
                settled_at_epoch_seconds: 1_708_160_020,
                settlement_receipt_hash: "sha256:receipt-6".to_owned(),
            },
        )
        .expect("release transition should succeed");

    registry
        .append(DataLayerM4SettlementEvidenceInput {
            escrow_id: "escrow-m4-6".to_owned(),
            escrow_state: DataLayerM4EscrowState::Released,
            settlement_receipt_hash: "sha256:receipt-6".to_owned(),
            settlement_payload_hash: "sha256:payload-6".to_owned(),
            recorded_at_epoch_seconds: 1_708_160_030,
        })
        .expect("evidence append should succeed");
    let escrow = engine
        .escrow("escrow-m4-6")
        .expect("escrow should exist")
        .clone();

    let report = registry
        .reconcile_against_escrow(&escrow)
        .expect("reconciliation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM4SettlementEvidenceReconciliationDecision::Match
    );
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MATCH_REASON_CODE
    );
}

#[test]
fn spec_c07_settlement_evidence_reconciliation_reports_mismatch_when_receipt_differs() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    let mut registry = DataLayerM4SettlementEvidenceRegistry::new();
    engine
        .create_escrow(draft_input("escrow-m4-7"))
        .expect("escrow draft should initialize");
    engine
        .apply_transition(
            "escrow-m4-7",
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: 1_708_160_000,
            },
        )
        .expect("fund transition should succeed");
    engine
        .apply_transition(
            "escrow-m4-7",
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds: 1_708_160_010,
            },
        )
        .expect("activate transition should succeed");
    engine
        .apply_transition(
            "escrow-m4-7",
            DataLayerM4EscrowTransitionAction::ResolveRelease {
                settled_at_epoch_seconds: 1_708_160_020,
                settlement_receipt_hash: "sha256:receipt-7-expected".to_owned(),
            },
        )
        .expect("release transition should succeed");

    registry
        .append(DataLayerM4SettlementEvidenceInput {
            escrow_id: "escrow-m4-7".to_owned(),
            escrow_state: DataLayerM4EscrowState::Released,
            settlement_receipt_hash: "sha256:receipt-7-actual".to_owned(),
            settlement_payload_hash: "sha256:payload-7".to_owned(),
            recorded_at_epoch_seconds: 1_708_160_030,
        })
        .expect("evidence append should succeed");
    let escrow = engine
        .escrow("escrow-m4-7")
        .expect("escrow should exist")
        .clone();

    let report = registry
        .reconcile_against_escrow(&escrow)
        .expect("reconciliation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM4SettlementEvidenceReconciliationDecision::Mismatch
    );
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MISMATCH_REASON_CODE
    );
}

#[test]
fn spec_c08_settlement_evidence_reconciliation_rejects_non_terminal_escrow_state() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    let registry = DataLayerM4SettlementEvidenceRegistry::new();
    engine
        .create_escrow(draft_input("escrow-m4-8"))
        .expect("escrow draft should initialize");
    engine
        .apply_transition(
            "escrow-m4-8",
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: 1_708_160_000,
            },
        )
        .expect("fund transition should succeed");
    let escrow = engine
        .escrow("escrow-m4-8")
        .expect("escrow should exist")
        .clone();

    let invalid = registry.reconcile_against_escrow(&escrow);
    assert!(matches!(
        invalid,
        Err(
            DataLayerM4SettlementEvidenceRegistryError::UnsupportedSettlementState(
                DataLayerM4EscrowState::Funded
            )
        )
    ));
}

#[test]
fn spec_c09_m4_bridge_maps_representable_legacy_escrow_states() {
    assert_eq!(
        DataLayerM4EscrowState::try_from(EscrowStatus::Funded).expect("funded should map"),
        DataLayerM4EscrowState::Funded
    );
    assert_eq!(
        DataLayerM4EscrowState::try_from(EscrowStatus::PartiallyReleased {
            released: 3,
            remaining: 7,
        })
        .expect("partially-released should map"),
        DataLayerM4EscrowState::Active
    );
    assert_eq!(
        DataLayerM4EscrowState::try_from(EscrowStatus::Disputed).expect("disputed should map"),
        DataLayerM4EscrowState::Disputed
    );
    assert_eq!(
        DataLayerM4EscrowState::try_from(EscrowStatus::Released).expect("released should map"),
        DataLayerM4EscrowState::Released
    );
    assert_eq!(
        DataLayerM4EscrowState::try_from(EscrowStatus::Refunded).expect("refunded should map"),
        DataLayerM4EscrowState::Refunded
    );
}

#[test]
fn spec_c10_m4_bridge_rejects_ambiguous_legacy_resolved_split() {
    let ambiguous = DataLayerM4EscrowState::try_from(EscrowStatus::Resolved {
        released_total: 5,
        refunded_total: 5,
    });
    assert!(matches!(
        ambiguous,
        Err(DataLayerM4EscrowInteropError::UnsupportedLegacyStatus(_))
    ));
}
