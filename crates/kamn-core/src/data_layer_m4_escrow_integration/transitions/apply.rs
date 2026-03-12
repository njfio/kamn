mod support;

use super::super::models::*;
use super::super::validation::{
    ensure_transition_allowed, reason_code_for_transition, validate_hash_token,
    validate_non_empty, validate_non_zero_timestamp,
};
use support::{
    activate_transition, dispute_transition, expire_transition, fund_transition,
    refund_transition, release_transition,
};

impl DataLayerM4EscrowTransitionEngine {
    /// Applies one escrow transition action.
    pub fn apply_transition(
        &mut self,
        escrow_id: &str,
        action: DataLayerM4EscrowTransitionAction,
    ) -> Result<DataLayerM4EscrowTransitionEvidence, DataLayerM4SettlementEvidenceRegistryError>
    {
        validate_non_empty(escrow_id, "escrow_id")?;
        let escrow = self.escrows.get_mut(escrow_id).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: escrow_id.to_owned(),
            }
        })?;
        let from = escrow.state;
        let to = apply_transition_action(escrow, from, &action)?;
        escrow.state = to;
        Ok(DataLayerM4EscrowTransitionEvidence {
            escrow_id: escrow.escrow_id.clone(),
            from,
            action: action.clone(),
            to,
            reason_code: reason_code_for_transition(&action),
        })
    }
}

fn apply_transition_action(
    escrow: &mut DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::Fund { .. } => fund_transition(escrow, from, action),
        DataLayerM4EscrowTransitionAction::Activate { .. } => {
            activate_transition(escrow, from, action)
        }
        DataLayerM4EscrowTransitionAction::OpenDispute { .. } => {
            dispute_transition(escrow, from, action)
        }
        DataLayerM4EscrowTransitionAction::ResolveRelease { .. } => {
            release_transition(escrow, from, action)
        }
        DataLayerM4EscrowTransitionAction::ResolveRefund { .. } => {
            refund_transition(escrow, from, action)
        }
        DataLayerM4EscrowTransitionAction::Expire { .. } => expire_transition(escrow, from, action),
    }
}

pub(super) fn apply_simple_transition(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
    timestamp: u64,
    field: &'static str,
    allowed: &[DataLayerM4EscrowState],
    next_state: DataLayerM4EscrowState,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    validate_non_zero_timestamp(timestamp, field)?;
    ensure_transition_allowed(escrow.escrow_id.as_str(), from, action, allowed)?;
    Ok(next_state)
}

pub(super) fn apply_dispute_transition(
    escrow: &mut DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
    opened_at: u64,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let next_state = apply_simple_transition(
        escrow,
        from,
        action,
        opened_at,
        "dispute_opened_at_epoch_seconds",
        &[DataLayerM4EscrowState::Active],
        DataLayerM4EscrowState::Disputed,
    )?;
    escrow.dispute_opened_at_epoch_seconds = Some(opened_at);
    Ok(next_state)
}

fn apply_settlement_transition(
    escrow: &mut DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
    settled_at_epoch_seconds: u64,
    settlement_receipt_hash: &str,
    next_state: DataLayerM4EscrowState,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    validate_non_zero_timestamp(settled_at_epoch_seconds, "settled_at_epoch_seconds")?;
    validate_hash_token(settlement_receipt_hash, "settlement_receipt_hash")?;
    ensure_transition_allowed(
        escrow.escrow_id.as_str(),
        from,
        action,
        &[DataLayerM4EscrowState::Active, DataLayerM4EscrowState::Disputed],
    )?;
    escrow.settled_at_epoch_seconds = Some(settled_at_epoch_seconds);
    escrow.settlement_receipt_hash = Some(settlement_receipt_hash.to_owned());
    Ok(next_state)
}
