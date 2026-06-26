use super::super::super::models::*;
use super::{apply_dispute_transition, apply_simple_transition};

pub(super) fn fund_transition(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    apply_simple_transition(
        escrow,
        from,
        action,
        require_fund_timestamp(action),
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
    apply_simple_transition(
        escrow,
        from,
        action,
        require_activate_timestamp(action),
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
    apply_dispute_transition(escrow, from, action, require_dispute_timestamp(action))
}

pub(super) fn expire_transition(
    escrow: &DataLayerM4EscrowRecord,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
) -> Result<DataLayerM4EscrowState, DataLayerM4SettlementEvidenceRegistryError> {
    apply_simple_transition(
        escrow,
        from,
        action,
        require_expire_timestamp(action),
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
    let (settled_at_epoch_seconds, settlement_receipt_hash) = require_release_transition(action);
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
    let (settled_at_epoch_seconds, settlement_receipt_hash) = require_refund_transition(action);
    super::apply_settlement_transition(
        escrow,
        from,
        action,
        settled_at_epoch_seconds,
        settlement_receipt_hash,
        DataLayerM4EscrowState::Refunded,
    )
}

fn require_fund_timestamp(action: &DataLayerM4EscrowTransitionAction) -> u64 {
    match action {
        DataLayerM4EscrowTransitionAction::Fund {
            funded_at_epoch_seconds,
        } => *funded_at_epoch_seconds,
        _ => unreachable!("fund transition requires fund action"),
    }
}

fn require_activate_timestamp(action: &DataLayerM4EscrowTransitionAction) -> u64 {
    match action {
        DataLayerM4EscrowTransitionAction::Activate {
            activated_at_epoch_seconds,
        } => *activated_at_epoch_seconds,
        _ => unreachable!("activate transition requires activate action"),
    }
}

fn require_dispute_timestamp(action: &DataLayerM4EscrowTransitionAction) -> u64 {
    match action {
        DataLayerM4EscrowTransitionAction::OpenDispute {
            dispute_opened_at_epoch_seconds,
        } => *dispute_opened_at_epoch_seconds,
        _ => unreachable!("dispute transition requires dispute action"),
    }
}

fn require_expire_timestamp(action: &DataLayerM4EscrowTransitionAction) -> u64 {
    match action {
        DataLayerM4EscrowTransitionAction::Expire {
            expired_at_epoch_seconds,
        } => *expired_at_epoch_seconds,
        _ => unreachable!("expire transition requires expire action"),
    }
}

fn require_release_transition(action: &DataLayerM4EscrowTransitionAction) -> (u64, &str) {
    match action {
        DataLayerM4EscrowTransitionAction::ResolveRelease {
            settled_at_epoch_seconds,
            settlement_receipt_hash,
        } => (*settled_at_epoch_seconds, settlement_receipt_hash.as_str()),
        _ => unreachable!("release transition requires release action"),
    }
}

fn require_refund_transition(action: &DataLayerM4EscrowTransitionAction) -> (u64, &str) {
    match action {
        DataLayerM4EscrowTransitionAction::ResolveRefund {
            settled_at_epoch_seconds,
            settlement_receipt_hash,
        } => (*settled_at_epoch_seconds, settlement_receipt_hash.as_str()),
        _ => unreachable!("refund transition requires refund action"),
    }
}
