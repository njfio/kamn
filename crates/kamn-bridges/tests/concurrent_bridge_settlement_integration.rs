use kamn_bridges::cross_chain_receipt::{
    CrossChainReceiptNetwork, CrossChainReceiptNormalizationError, CrossChainReceiptProof,
    CrossChainReceiptStatus,
};
use kamn_bridges::cross_chain_settlement::{
    evaluate_cross_chain_settlement_decision, CrossChainSettlementDecision,
    CrossChainSettlementRejectionReason,
};
use std::thread;

fn proof(
    network: CrossChainReceiptNetwork,
    receipt_id: &str,
    finality_label: &str,
    confirmation_count: u64,
    status: CrossChainReceiptStatus,
) -> CrossChainReceiptProof {
    CrossChainReceiptProof {
        network,
        receipt_id: receipt_id.to_owned(),
        block_reference: "block-1".to_owned(),
        finality_label: finality_label.to_owned(),
        confirmation_count,
        status,
    }
}

fn evaluate_all_concurrently(
    proofs: Vec<CrossChainReceiptProof>,
) -> Vec<CrossChainSettlementDecision> {
    let mut handles = proofs
        .into_iter()
        .enumerate()
        .map(|(index, proof)| {
            thread::spawn(move || (index, evaluate_cross_chain_settlement_decision(&proof)))
        })
        .collect::<Vec<_>>();

    let mut decisions = vec![None; handles.len()];
    for handle in handles.drain(..) {
        let (index, decision) = handle.join().expect("thread should complete");
        decisions[index] = Some(decision);
    }

    decisions
        .into_iter()
        .map(|decision| decision.expect("index should be filled"))
        .collect()
}

#[test]
fn integration_concurrent_mixed_proofs_yield_expected_decisions() {
    let decisions = evaluate_all_concurrently(vec![
        proof(
            CrossChainReceiptNetwork::Ethereum,
            "receipt-final",
            "safe",
            12,
            CrossChainReceiptStatus::Success,
        ),
        proof(
            CrossChainReceiptNetwork::Ethereum,
            "receipt-pending",
            "safe",
            3,
            CrossChainReceiptStatus::Success,
        ),
        proof(
            CrossChainReceiptNetwork::Near,
            "receipt-failed",
            "final",
            0,
            CrossChainReceiptStatus::Failed,
        ),
        proof(
            CrossChainReceiptNetwork::Near,
            " ",
            "final",
            0,
            CrossChainReceiptStatus::Success,
        ),
    ]);

    assert_eq!(
        decisions,
        vec![
            CrossChainSettlementDecision::Settle,
            CrossChainSettlementDecision::DeferPendingFinality,
            CrossChainSettlementDecision::Reject(
                CrossChainSettlementRejectionReason::FailedReceipt
            ),
            CrossChainSettlementDecision::Reject(
                CrossChainSettlementRejectionReason::InvalidProof(
                    CrossChainReceiptNormalizationError::EmptyField("receipt_id")
                )
            ),
        ]
    );
}

#[test]
fn integration_concurrent_identical_final_proof_always_settles() {
    let decisions = evaluate_all_concurrently(
        std::iter::repeat_with(|| {
            proof(
                CrossChainReceiptNetwork::Ethereum,
                "receipt-final",
                "safe",
                12,
                CrossChainReceiptStatus::Success,
            )
        })
        .take(16)
        .collect(),
    );

    assert!(decisions
        .into_iter()
        .all(|decision| decision == CrossChainSettlementDecision::Settle));
}

#[test]
fn integration_concurrent_pending_and_invalid_proofs_preserve_decision_boundaries() {
    let decisions = evaluate_all_concurrently(vec![
        proof(
            CrossChainReceiptNetwork::Near,
            "receipt-pending",
            "optimistic",
            0,
            CrossChainReceiptStatus::Success,
        ),
        proof(
            CrossChainReceiptNetwork::Near,
            "receipt-invalid-label",
            "unsafe",
            0,
            CrossChainReceiptStatus::Success,
        ),
        proof(
            CrossChainReceiptNetwork::Ethereum,
            "receipt-pending-eth",
            "latest",
            2,
            CrossChainReceiptStatus::Success,
        ),
        proof(
            CrossChainReceiptNetwork::Ethereum,
            "receipt-invalid-field",
            " ",
            0,
            CrossChainReceiptStatus::Success,
        ),
    ]);

    assert_eq!(
        decisions[0],
        CrossChainSettlementDecision::DeferPendingFinality
    );
    assert_eq!(
        decisions[1],
        CrossChainSettlementDecision::Reject(CrossChainSettlementRejectionReason::InvalidProof(
            CrossChainReceiptNormalizationError::UnsupportedFinalityLabel {
                network: CrossChainReceiptNetwork::Near,
                label: "unsafe".to_owned(),
            }
        ))
    );
    assert_eq!(
        decisions[2],
        CrossChainSettlementDecision::DeferPendingFinality
    );
    assert_eq!(
        decisions[3],
        CrossChainSettlementDecision::Reject(CrossChainSettlementRejectionReason::InvalidProof(
            CrossChainReceiptNormalizationError::EmptyField("finality_label")
        ))
    );
}
