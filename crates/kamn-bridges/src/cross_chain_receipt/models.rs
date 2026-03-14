/// Ethereum confirmations required before a successful receipt is final.
pub const ETHEREUM_FINAL_CONFIRMATION_THRESHOLD: u64 = 12;

/// Supported receipt networks for normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrossChainReceiptNetwork {
    /// Ethereum receipts.
    Ethereum,
    /// Solana receipts.
    Solana,
    /// Near receipts.
    Near,
}

impl CrossChainReceiptNetwork {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Solana => "solana",
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
