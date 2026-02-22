use crate::errors::AgentLibError;
use kamn_kolme::{parse_receipt_finality, ReceiptFinality};

/// Proof receipt fields required for verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeProofReceipt {
    /// Transaction hash.
    pub tx_hash: String,
    /// Anchored block height.
    pub block_height: u64,
    /// Finality marker.
    pub finality: String,
}

/// Verification result projected from one proof receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeProofVerification {
    /// Message identifier associated with the proof.
    pub message_id: String,
    /// Anchored block height.
    pub block_height: u64,
    /// Canonical finality marker (`PENDING`, `FINAL`, `FAILED`).
    pub finality: String,
    /// Whether proof verification is final and successful.
    pub verified: bool,
}

/// Minimal Kolme verification client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeClient {
    endpoint: String,
}

impl KolmeClient {
    /// Creates a client bound to one Kolme endpoint.
    pub fn new(endpoint: &str) -> Result<Self, AgentLibError> {
        let trimmed = endpoint.trim();
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err(AgentLibError::InvalidInput {
                field: "kolme_endpoint",
                reason: "must start with http:// or https://".to_owned(),
            });
        }
        Ok(Self {
            endpoint: trimmed.to_owned(),
        })
    }

    /// Returns the configured Kolme endpoint.
    pub fn endpoint(&self) -> &str {
        self.endpoint.as_str()
    }

    /// Verifies one receipt against deterministic finality contracts.
    pub fn verify_proof(
        &self,
        message_id: &str,
        receipt: &KolmeProofReceipt,
    ) -> Result<KolmeProofVerification, AgentLibError> {
        if message_id.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "message_id",
                reason: "must not be empty".to_owned(),
            });
        }
        if receipt.tx_hash.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "receipt.tx_hash",
                reason: "must not be empty".to_owned(),
            });
        }
        if receipt.block_height == 0 {
            return Err(AgentLibError::InvalidInput {
                field: "receipt.block_height",
                reason: "must be greater than zero".to_owned(),
            });
        }

        let finality_input = receipt.finality.trim().to_ascii_lowercase();
        let parsed_finality = parse_receipt_finality(finality_input.as_str()).map_err(|_| {
            AgentLibError::InvalidInput {
                field: "receipt.finality",
                reason:
                    "must be pending|accepted|mempool|final|confirmed|finalized|failed|rejected"
                        .to_owned(),
            }
        })?;

        let (finality, verified) = match parsed_finality {
            ReceiptFinality::Pending => ("PENDING".to_owned(), false),
            ReceiptFinality::Final => ("FINAL".to_owned(), true),
            ReceiptFinality::Failed => ("FAILED".to_owned(), false),
        };

        Ok(KolmeProofVerification {
            message_id: message_id.to_owned(),
            block_height: receipt.block_height,
            finality,
            verified,
        })
    }
}
