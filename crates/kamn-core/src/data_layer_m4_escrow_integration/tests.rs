use super::{
    DataLayerM4EscrowDraftInput, DataLayerM4EscrowState, DataLayerM4EscrowTransitionAction,
    DataLayerM4EscrowTransitionEngine, DataLayerM4EscrowVisibilityDecision,
    DataLayerM4EscrowVisibilityRequest, DataLayerM4SettlementEvidenceRegistryError,
    DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE,
};

fn fixture_draft() -> DataLayerM4EscrowDraftInput {
    DataLayerM4EscrowDraftInput {
        escrow_id: "escrow-1".to_owned(),
        initiator_did: "kamn:did:owner:alice".to_owned(),
        counterparty_did: "kamn:did:owner:bob".to_owned(),
        auditor_did: Some("kamn:did:auditor:carol".to_owned()),
        auditor_threshold: Some(2),
        auditor_share_holders: vec![
            "kamn:did:holder:one".to_owned(),
            "kamn:did:holder:two".to_owned(),
        ],
        expires_at_epoch_seconds: Some(2_000),
    }
}

#[test]
fn unit_data_layer_m4_escrow_transition_flow_reaches_disputed() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    create_and_progress_escrow(&mut engine).expect("escrow dispute transition flow should succeed");

    let escrow = engine
        .escrow("escrow-1")
        .expect("escrow should remain stored after transitions");
    assert_eq!(escrow.state, DataLayerM4EscrowState::Disputed);

    let visibility = engine
        .authorize_message_visibility(DataLayerM4EscrowVisibilityRequest {
            escrow_id: "escrow-1".to_owned(),
            requester_did: "kamn:did:auditor:carol".to_owned(),
            reconstructed_auditor_shares: Some(2),
        })
        .expect("auditor visibility decision should be computed");
    assert!(matches!(
        visibility,
        DataLayerM4EscrowVisibilityDecision::Allow {
            reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE
        }
    ));
}

fn create_and_progress_escrow(
    engine: &mut DataLayerM4EscrowTransitionEngine,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    engine.create_escrow(fixture_draft())?;
    apply_transition(engine, DataLayerM4EscrowTransitionAction::Fund {
        funded_at_epoch_seconds: 1_001,
    })?;
    apply_transition(engine, DataLayerM4EscrowTransitionAction::Activate {
        activated_at_epoch_seconds: 1_002,
    })?;
    apply_transition(engine, DataLayerM4EscrowTransitionAction::OpenDispute {
        dispute_opened_at_epoch_seconds: 1_003,
    })
}

fn apply_transition(
    engine: &mut DataLayerM4EscrowTransitionEngine,
    action: DataLayerM4EscrowTransitionAction,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    engine.apply_transition("escrow-1", action).map(|_| ())
}

#[test]
fn unit_data_layer_m4_rejects_invalid_state_transition() {
    let mut engine = DataLayerM4EscrowTransitionEngine::new();
    engine
        .create_escrow(fixture_draft())
        .expect("escrow draft should create");
    let error = engine
        .apply_transition(
            "escrow-1",
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds: 1_001,
            },
        )
        .expect_err("activate from created state must fail");
    assert!(matches!(
        error,
        DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowTransition { .. }
    ));
}
