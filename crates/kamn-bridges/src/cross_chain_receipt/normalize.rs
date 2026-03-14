use super::{
    CrossChainReceiptFinality, CrossChainReceiptNetwork, CrossChainReceiptNormalizationError,
    CrossChainReceiptProof, CrossChainReceiptStatus, NormalizedCrossChainReceipt,
    ETHEREUM_FINAL_CONFIRMATION_THRESHOLD,
};

/// Converts a network-specific receipt proof into normalized finality form.
pub fn normalize_cross_chain_receipt(
    proof: &CrossChainReceiptProof,
) -> Result<NormalizedCrossChainReceipt, CrossChainReceiptNormalizationError> {
    validate_non_empty("receipt_id", &proof.receipt_id)?;
    validate_non_empty("block_reference", &proof.block_reference)?;
    validate_non_empty("finality_label", &proof.finality_label)?;

    let finality = match proof.status {
        CrossChainReceiptStatus::Failed => CrossChainReceiptFinality::Failed,
        CrossChainReceiptStatus::Pending => CrossChainReceiptFinality::Pending,
        CrossChainReceiptStatus::Success => normalize_success_finality(
            proof.network,
            &proof.finality_label,
            proof.confirmation_count,
        )?,
    };

    Ok(NormalizedCrossChainReceipt {
        network: proof.network,
        receipt_id: proof.receipt_id.clone(),
        block_reference: proof.block_reference.clone(),
        finality,
    })
}

fn normalize_success_finality(
    network: CrossChainReceiptNetwork,
    finality_label: &str,
    confirmation_count: u64,
) -> Result<CrossChainReceiptFinality, CrossChainReceiptNormalizationError> {
    let label = finality_label.trim().to_ascii_lowercase();

    match network {
        CrossChainReceiptNetwork::Ethereum => {
            normalize_ethereum(label.as_str(), confirmation_count)
        }
        CrossChainReceiptNetwork::Solana => normalize_solana(label.as_str(), network),
        CrossChainReceiptNetwork::Near => normalize_near(label.as_str(), network),
    }
}

fn normalize_ethereum(
    label: &str,
    confirmation_count: u64,
) -> Result<CrossChainReceiptFinality, CrossChainReceiptNormalizationError> {
    match label {
        "finalized" | "safe" if confirmation_count >= ETHEREUM_FINAL_CONFIRMATION_THRESHOLD => {
            Ok(CrossChainReceiptFinality::Final)
        }
        "finalized" | "safe" | "latest" => Ok(CrossChainReceiptFinality::Pending),
        _ => unsupported_label(CrossChainReceiptNetwork::Ethereum, label),
    }
}

fn normalize_solana(
    label: &str,
    network: CrossChainReceiptNetwork,
) -> Result<CrossChainReceiptFinality, CrossChainReceiptNormalizationError> {
    match label {
        "finalized" => Ok(CrossChainReceiptFinality::Final),
        "confirmed" | "processed" => Ok(CrossChainReceiptFinality::Pending),
        _ => unsupported_label(network, label),
    }
}

fn normalize_near(
    label: &str,
    network: CrossChainReceiptNetwork,
) -> Result<CrossChainReceiptFinality, CrossChainReceiptNormalizationError> {
    match label {
        "final" => Ok(CrossChainReceiptFinality::Final),
        "optimistic" | "none" => Ok(CrossChainReceiptFinality::Pending),
        _ => unsupported_label(network, label),
    }
}

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), CrossChainReceiptNormalizationError> {
    if value.trim().is_empty() {
        return Err(CrossChainReceiptNormalizationError::EmptyField(field));
    }
    Ok(())
}

fn unsupported_label(
    network: CrossChainReceiptNetwork,
    label: &str,
) -> Result<CrossChainReceiptFinality, CrossChainReceiptNormalizationError> {
    Err(
        CrossChainReceiptNormalizationError::UnsupportedFinalityLabel {
            network,
            label: label.to_owned(),
        },
    )
}
