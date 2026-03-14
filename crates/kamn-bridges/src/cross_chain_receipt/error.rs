use std::fmt;

use super::models::CrossChainReceiptNetwork;

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
