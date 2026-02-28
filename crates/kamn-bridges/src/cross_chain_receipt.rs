//! Cross-chain receipt normalization and finality policy contracts.
//!
//! This module maps network-specific receipt semantics into a normalized finality
//! model used by bridge settlement and reconciliation workflows.

use std::fmt;

/// Ethereum confirmations required before a successful receipt is final.
pub const ETHEREUM_FINAL_CONFIRMATION_THRESHOLD: u64 = 12;

/// Supported receipt networks for normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrossChainReceiptNetwork {
    /// Ethereum receipts.
    Ethereum,
    /// Near receipts.
    Near,
}

impl CrossChainReceiptNetwork {
    fn label(self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Near => "near",
        }
    }
}

/// Raw receipt execution status from external network sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossChainReceiptStatus {
    /// Receipt execution completed successfully.
    Success,
    /// Receipt execution is still pending.
    Pending,
    /// Receipt execution failed.
    Failed,
}

/// Raw receipt proof payload before normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossChainReceiptProof {
    /// Network where the receipt originated.
    pub network: CrossChainReceiptNetwork,
    /// Receipt identifier.
    pub receipt_id: String,
    /// Block reference containing the receipt.
    pub block_reference: String,
    /// Network-specific finality label.
    pub finality_label: String,
    /// Confirmation count observed for the receipt.
    pub confirmation_count: u64,
    /// Raw execution status.
    pub status: CrossChainReceiptStatus,
}

/// Normalized settlement finality classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossChainReceiptFinality {
    /// Receipt is final and safe for settlement.
    Final,
    /// Receipt is not yet final.
    Pending,
    /// Receipt failed and should not settle.
    Failed,
}

/// Network-agnostic receipt view used by settlement workflows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedCrossChainReceipt {
    /// Source network.
    pub network: CrossChainReceiptNetwork,
    /// Receipt identifier.
    pub receipt_id: String,
    /// Block reference.
    pub block_reference: String,
    /// Normalized finality classification.
    pub finality: CrossChainReceiptFinality,
}

/// Error surface for receipt normalization inputs and labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossChainReceiptNormalizationError {
    /// Required string field was empty.
    EmptyField(&'static str),
    /// Finality label is unsupported for the specified network.
    UnsupportedFinalityLabel {
        /// Source network.
        network: CrossChainReceiptNetwork,
        /// Unsupported finality label value.
        label: String,
    },
}

impl fmt::Display for CrossChainReceiptNormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::UnsupportedFinalityLabel { network, label } => {
                write!(f, "unsupported {} finality label: {label}", network.label())
            }
        }
    }
}

impl std::error::Error for CrossChainReceiptNormalizationError {}

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
        CrossChainReceiptNetwork::Ethereum => match label.as_str() {
            "finalized" | "safe" if confirmation_count >= ETHEREUM_FINAL_CONFIRMATION_THRESHOLD => {
                Ok(CrossChainReceiptFinality::Final)
            }
            "finalized" | "safe" | "latest" => Ok(CrossChainReceiptFinality::Pending),
            _ => Err(
                CrossChainReceiptNormalizationError::UnsupportedFinalityLabel { network, label },
            ),
        },
        CrossChainReceiptNetwork::Near => match label.as_str() {
            "final" => Ok(CrossChainReceiptFinality::Final),
            "optimistic" | "none" => Ok(CrossChainReceiptFinality::Pending),
            _ => Err(
                CrossChainReceiptNormalizationError::UnsupportedFinalityLabel { network, label },
            ),
        },
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

#[cfg(test)]
mod tests {
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
}
