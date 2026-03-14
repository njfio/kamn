use super::{
    normalize_cross_chain_receipt, CrossChainReceiptFinality, CrossChainReceiptNetwork,
    CrossChainReceiptNormalizationError, CrossChainReceiptProof, CrossChainReceiptStatus,
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
fn ethereum_safe_with_threshold_confirmations_is_final() {
    let normalized = normalize_cross_chain_receipt(&proof(
        CrossChainReceiptNetwork::Ethereum,
        "safe",
        12,
        CrossChainReceiptStatus::Success,
    ))
    .expect("ethereum proof should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Final);
}

#[test]
fn ethereum_safe_below_threshold_is_pending() {
    let normalized = normalize_cross_chain_receipt(&proof(
        CrossChainReceiptNetwork::Ethereum,
        "safe",
        11,
        CrossChainReceiptStatus::Success,
    ))
    .expect("ethereum proof should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Pending);
}

#[test]
fn near_final_is_final() {
    let normalized = normalize_cross_chain_receipt(&proof(
        CrossChainReceiptNetwork::Near,
        "final",
        0,
        CrossChainReceiptStatus::Success,
    ))
    .expect("near proof should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Final);
}

#[test]
fn failed_status_is_failed_regardless_of_chain_label() {
    let normalized = normalize_cross_chain_receipt(&proof(
        CrossChainReceiptNetwork::Near,
        "unsupported-label",
        0,
        CrossChainReceiptStatus::Failed,
    ))
    .expect("failed status should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Failed);
}

#[test]
fn rejects_unknown_near_finality_label_when_successful() {
    assert_eq!(
        normalize_cross_chain_receipt(&proof(
            CrossChainReceiptNetwork::Near,
            "unsafe",
            0,
            CrossChainReceiptStatus::Success,
        )),
        Err(
            CrossChainReceiptNormalizationError::UnsupportedFinalityLabel {
                network: CrossChainReceiptNetwork::Near,
                label: "unsafe".to_owned(),
            }
        )
    );
}

#[test]
fn rejects_empty_receipt_id() {
    assert_eq!(
        normalize_cross_chain_receipt(&CrossChainReceiptProof {
            receipt_id: "   ".to_owned(),
            ..proof(
                CrossChainReceiptNetwork::Near,
                "final",
                0,
                CrossChainReceiptStatus::Success,
            )
        }),
        Err(CrossChainReceiptNormalizationError::EmptyField(
            "receipt_id"
        ))
    );
}

#[test]
fn rejects_empty_block_reference() {
    assert_eq!(
        normalize_cross_chain_receipt(&CrossChainReceiptProof {
            block_reference: "".to_owned(),
            ..proof(
                CrossChainReceiptNetwork::Near,
                "final",
                0,
                CrossChainReceiptStatus::Success,
            )
        }),
        Err(CrossChainReceiptNormalizationError::EmptyField(
            "block_reference"
        ))
    );
}

#[test]
fn rejects_empty_finality_label() {
    assert_eq!(
        normalize_cross_chain_receipt(&CrossChainReceiptProof {
            finality_label: " ".to_owned(),
            ..proof(
                CrossChainReceiptNetwork::Near,
                "final",
                0,
                CrossChainReceiptStatus::Success,
            )
        }),
        Err(CrossChainReceiptNormalizationError::EmptyField(
            "finality_label"
        ))
    );
}

#[test]
fn ethereum_finality_label_is_normalized_for_case_and_whitespace() {
    let normalized = normalize_cross_chain_receipt(&proof(
        CrossChainReceiptNetwork::Ethereum,
        "  SaFe ",
        12,
        CrossChainReceiptStatus::Success,
    ))
    .expect("ethereum normalized safe label should parse");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Final);
}

#[test]
fn pending_status_remains_pending_even_when_label_is_final() {
    let normalized = normalize_cross_chain_receipt(&proof(
        CrossChainReceiptNetwork::Near,
        "final",
        0,
        CrossChainReceiptStatus::Pending,
    ))
    .expect("pending status should normalize");
    assert_eq!(normalized.finality, CrossChainReceiptFinality::Pending);
}
