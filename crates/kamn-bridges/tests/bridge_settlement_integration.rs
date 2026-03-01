use kamn_bridges::cross_chain_receipt::{
    CrossChainReceiptNetwork, CrossChainReceiptNormalizationError, CrossChainReceiptProof,
    CrossChainReceiptStatus,
};
use kamn_bridges::cross_chain_settlement::{
    evaluate_cross_chain_settlement_decision, CrossChainSettlementDecision,
    CrossChainSettlementRejectionReason,
};

fn proof(
    network: CrossChainReceiptNetwork,
    finality_label: &str,
    confirmation_count: u64,
    status: CrossChainReceiptStatus,
) -> CrossChainReceiptProof {
    CrossChainReceiptProof {
        network,
        receipt_id: "receipt-1".to_owned(),
        block_reference: "block-1".to_owned(),
        finality_label: finality_label.to_owned(),
        confirmation_count,
        status,
    }
}

#[test]
fn integration_settlement_decision_settles_final_receipt() {
    let decision = evaluate_cross_chain_settlement_decision(&proof(
        CrossChainReceiptNetwork::Ethereum,
        "safe",
        12,
        CrossChainReceiptStatus::Success,
    ));
    assert_eq!(decision, CrossChainSettlementDecision::Settle);
}

#[test]
fn integration_settlement_decision_defers_pending_receipt() {
    let decision = evaluate_cross_chain_settlement_decision(&proof(
        CrossChainReceiptNetwork::Ethereum,
        "safe",
        3,
        CrossChainReceiptStatus::Success,
    ));
    assert_eq!(decision, CrossChainSettlementDecision::DeferPendingFinality);
}

#[test]
fn integration_settlement_decision_rejects_failed_receipt() {
    let decision = evaluate_cross_chain_settlement_decision(&proof(
        CrossChainReceiptNetwork::Near,
        "final",
        0,
        CrossChainReceiptStatus::Failed,
    ));
    assert_eq!(
        decision,
        CrossChainSettlementDecision::Reject(CrossChainSettlementRejectionReason::FailedReceipt)
    );
}

#[test]
fn integration_settlement_decision_rejects_invalid_proof() {
    let decision = evaluate_cross_chain_settlement_decision(&CrossChainReceiptProof {
        receipt_id: " ".to_owned(),
        ..proof(
            CrossChainReceiptNetwork::Near,
            "final",
            0,
            CrossChainReceiptStatus::Success,
        )
    });
    assert_eq!(
        decision,
        CrossChainSettlementDecision::Reject(CrossChainSettlementRejectionReason::InvalidProof(
            CrossChainReceiptNormalizationError::EmptyField("receipt_id")
        ))
    );
}
