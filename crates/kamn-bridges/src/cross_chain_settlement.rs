//! Settlement decision policy over normalized cross-chain receipts.

use crate::cross_chain_receipt::{
    normalize_cross_chain_receipt, CrossChainReceiptFinality, CrossChainReceiptNormalizationError,
    CrossChainReceiptProof,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical settlement decision for one cross-chain receipt proof.
pub enum CrossChainSettlementDecision {
    /// Receipt is final and eligible for settlement execution.
    Settle,
    /// Receipt is valid but not final yet.
    DeferPendingFinality,
    /// Receipt must not settle.
    Reject(CrossChainSettlementRejectionReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Typed rejection reasons for bridge settlement policy.
pub enum CrossChainSettlementRejectionReason {
    /// Receipt execution failed on source network.
    FailedReceipt,
    /// Receipt proof was invalid for normalization.
    InvalidProof(CrossChainReceiptNormalizationError),
}

/// Evaluates deterministic bridge settlement decision for one receipt proof.
pub fn evaluate_cross_chain_settlement_decision(
    proof: &CrossChainReceiptProof,
) -> CrossChainSettlementDecision {
    match normalize_cross_chain_receipt(proof) {
        Ok(normalized) => map_finality_to_settlement_decision(normalized.finality),
        Err(error) => CrossChainSettlementDecision::Reject(
            CrossChainSettlementRejectionReason::InvalidProof(error),
        ),
    }
}

fn map_finality_to_settlement_decision(
    finality: CrossChainReceiptFinality,
) -> CrossChainSettlementDecision {
    match finality {
        CrossChainReceiptFinality::Final => CrossChainSettlementDecision::Settle,
        CrossChainReceiptFinality::Pending => CrossChainSettlementDecision::DeferPendingFinality,
        CrossChainReceiptFinality::Failed => {
            CrossChainSettlementDecision::Reject(CrossChainSettlementRejectionReason::FailedReceipt)
        }
    }
}
