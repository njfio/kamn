use kamn_core::{
    normalize_cross_chain_receipt, CrossChainReceiptFinality, CrossChainReceiptNetwork,
    CrossChainReceiptNormalizationError, CrossChainReceiptProof, CrossChainReceiptStatus,
};

fn ethereum_proof(
    finality_label: &str,
    confirmations: u64,
    status: CrossChainReceiptStatus,
) -> CrossChainReceiptProof {
    CrossChainReceiptProof {
        network: CrossChainReceiptNetwork::Ethereum,
        receipt_id: "0xabc123:7".to_owned(),
        block_reference: "0xblock-88".to_owned(),
        finality_label: finality_label.to_owned(),
        confirmation_count: confirmations,
        status,
    }
}

fn near_proof(finality_label: &str, status: CrossChainReceiptStatus) -> CrossChainReceiptProof {
    CrossChainReceiptProof {
        network: CrossChainReceiptNetwork::Near,
        receipt_id: "near:receipt:991".to_owned(),
        block_reference: "near:block:2201".to_owned(),
        finality_label: finality_label.to_owned(),
        confirmation_count: 0,
        status,
    }
}

#[test]
fn ethereum_finalized_receipt_normalizes_to_final() {
    let normalized = normalize_cross_chain_receipt(&ethereum_proof(
        "finalized",
        18,
        CrossChainReceiptStatus::Success,
    ))
    .expect("ethereum finalized receipt should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Final);
}

#[test]
fn ethereum_latest_receipt_with_low_confirmations_normalizes_to_pending() {
    let normalized = normalize_cross_chain_receipt(&ethereum_proof(
        "latest",
        2,
        CrossChainReceiptStatus::Success,
    ))
    .expect("ethereum latest receipt should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Pending);
}

#[test]
fn near_final_receipt_normalizes_to_final() {
    let normalized =
        normalize_cross_chain_receipt(&near_proof("final", CrossChainReceiptStatus::Success))
            .expect("near final receipt should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Final);
}

#[test]
fn near_optimistic_receipt_normalizes_to_pending() {
    let normalized =
        normalize_cross_chain_receipt(&near_proof("optimistic", CrossChainReceiptStatus::Success))
            .expect("near optimistic receipt should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Pending);
}

#[test]
fn near_failed_receipt_normalizes_to_failed() {
    let normalized =
        normalize_cross_chain_receipt(&near_proof("final", CrossChainReceiptStatus::Failed))
            .expect("failed receipt should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Failed);
}

#[test]
fn regression_rejects_unknown_near_finality_label() {
    // Regression: #740
    assert_eq!(
        normalize_cross_chain_receipt(&near_proof("unsafe", CrossChainReceiptStatus::Success)),
        Err(
            CrossChainReceiptNormalizationError::UnsupportedFinalityLabel {
                network: CrossChainReceiptNetwork::Near,
                label: "unsafe".to_owned(),
            }
        )
    );
}
