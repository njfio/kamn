use super::super::super::models::*;
use super::{apply_dispute_transition, apply_simple_transition};

pub(super) fn fund_transition(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let funded_at_epoch_seconds = require_fund_timestamp(escrow, from, action)?;
    apply_simple_transition(
        escrow,
        from,
        action,
        funded_at_epoch_seconds,
        "funded_at_epoch_seconds",
        &[DataLayerM4EscrowState::Created],
        DataLayerM4EscrowState::Funded,
    )
}

pub(super) fn activate_transition(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let activated_at_epoch_seconds = require_activate_timestamp(escrow, from, action)?;
    apply_simple_transition(
        escrow,
        from,
        action,
        activated_at_epoch_seconds,
        "activated_at_epoch_seconds",
        &[DataLayerM4EscrowState::Funded],
        DataLayerM4EscrowState::Active,
    )
}

pub(super) fn dispute_transition(
    escrow: &mut DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let dispute_opened_at_epoch_seconds = require_dispute_timestamp(escrow, from, action)?;
    apply_dispute_transition(escrow, from, action, dispute_opened_at_epoch_seconds)
}

pub(super) fn expire_transition(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let expired_at_epoch_seconds = require_expire_timestamp(escrow, from, action)?;
    apply_simple_transition(
        escrow,
        from,
        action,
        expired_at_epoch_seconds,
        "expired_at_epoch_seconds",
        &[
            DataLayerM4EscrowState::Created,
            DataLayerM4EscrowState::Funded,
            DataLayerM4EscrowState::Active,
        ],
        DataLayerM4EscrowState::Expired,
    )
}

pub(super) fn release_transition(
    escrow: &mut DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let (settled_at_epoch_seconds, settlement_receipt_hash) =
        require_release_transition(escrow, from, action)?;
    super::apply_settlement_transition(
        escrow,
        from,
        action,
        settled_at_epoch_seconds,
        settlement_receipt_hash,
        DataLayerM4EscrowState::Released,
    )
}

pub(super) fn refund_transition(
    escrow: &mut DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    let (settled_at_epoch_seconds, settlement_receipt_hash) =
        require_refund_transition(escrow, from, action)?;
    super::apply_settlement_transition(
        escrow,
        from,
        action,
        settled_at_epoch_seconds,
        settlement_receipt_hash,
        DataLayerM4EscrowState::Refunded,
    )
}

fn require_fund_timestamp(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<u64, DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::Fund {
            funded_at_epoch_seconds,
        } => Ok(*funded_at_epoch_seconds),
        _ => invalid_transition_action(escrow, from, action),
    }
}

fn require_activate_timestamp(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<u64, DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::Activate {
            activated_at_epoch_seconds,
        } => Ok(*activated_at_epoch_seconds),
        _ => invalid_transition_action(escrow, from, action),
    }
}

fn require_dispute_timestamp(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<u64, DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::OpenDispute {
            dispute_opened_at_epoch_seconds,
        } => Ok(*dispute_opened_at_epoch_seconds),
        _ => invalid_transition_action(escrow, from, action),
    }
}

fn require_expire_timestamp(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<u64, DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::Expire {
            expired_at_epoch_seconds,
        } => Ok(*expired_at_epoch_seconds),
        _ => invalid_transition_action(escrow, from, action),
    }
}

fn require_release_transition<'a>(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &'a DataLayerM4EscrowTransitionAction,
) -> Result<(u64, &'a str), DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::ResolveRelease {
            settled_at_epoch_seconds,
            settlement_receipt_hash,
        } => Ok((*settled_at_epoch_seconds, settlement_receipt_hash.as_str())),
        _ => invalid_transition_action(escrow, from, action),
    }
}

fn require_refund_transition<'a>(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &'a DataLayerM4EscrowTransitionAction,
) -> Result<(u64, &'a str), DataLayerM4SettlementEvidenceRegistryError> {
    match action {
        DataLayerM4EscrowTransitionAction::ResolveRefund {
            settled_at_epoch_seconds,
            settlement_receipt_hash,
        } => Ok((*settled_at_epoch_seconds, settlement_receipt_hash.as_str())),
        _ => invalid_transition_action(escrow, from, action),
    }
}

fn invalid_transition_action<T>(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<T, DataLayerM4SettlementEvidenceRegistryError> {
    Err(
        DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowTransition {
            escrow_id: escrow.escrow_id.clone(),
            from,
            action: action.marker(),
        },
    )
}
