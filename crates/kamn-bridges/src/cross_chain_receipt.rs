//! Cross-chain receipt normalization and finality policy contracts.
//!
//! This module maps network-specific receipt semantics into a normalized finality
//! model used by bridge settlement and reconciliation workflows.

mod error;
mod models;
mod normalize;
#[cfg(test)]
mod tests;

pub use error::CrossChainReceiptNormalizationError;
pub use models::{
    CrossChainReceiptFinality, CrossChainReceiptNetwork, CrossChainReceiptProof,
    CrossChainReceiptStatus, NormalizedCrossChainReceipt, ETHEREUM_FINAL_CONFIRMATION_THRESHOLD,
};
pub use normalize::normalize_cross_chain_receipt;
