//! Receipt finality parsing contracts shared by Kolme runtime components.

use std::error::Error;
use std::fmt;

/// Runtime receipt finality classes returned by provider responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptFinality {
    /// Receipt is pending confirmation.
    Pending,
    /// Receipt is finalized.
    Final,
    /// Receipt is terminally failed.
    Failed,
}

/// Error raised while parsing a provider finality token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptFinalityError {
    /// Provider returned an unsupported finality token.
    InvalidFinalityValue(String),
}

impl fmt::Display for ReceiptFinalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFinalityValue(value) => write!(f, "invalid finality value: {value}"),
        }
    }
}

impl Error for ReceiptFinalityError {}

/// Parses canonical provider finality aliases into deterministic classes.
pub fn parse_receipt_finality(value: &str) -> Result<ReceiptFinality, ReceiptFinalityError> {
    match value {
        "pending" | "accepted" | "mempool" => Ok(ReceiptFinality::Pending),
        "final" | "confirmed" | "finalized" => Ok(ReceiptFinality::Final),
        "failed" | "rejected" => Ok(ReceiptFinality::Failed),
        _ => Err(ReceiptFinalityError::InvalidFinalityValue(value.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_receipt_finality, ReceiptFinality, ReceiptFinalityError};

    #[test]
    fn unit_parse_receipt_finality_maps_aliases() {
        assert_eq!(
            parse_receipt_finality("accepted").expect("accepted alias should parse"),
            ReceiptFinality::Pending
        );
        assert_eq!(
            parse_receipt_finality("finalized").expect("finalized alias should parse"),
            ReceiptFinality::Final
        );
        assert_eq!(
            parse_receipt_finality("failed").expect("failed alias should parse"),
            ReceiptFinality::Failed
        );
    }

    #[test]
    fn unit_parse_receipt_finality_rejects_unknown_value() {
        assert_eq!(
            parse_receipt_finality("settled"),
            Err(ReceiptFinalityError::InvalidFinalityValue(
                "settled".to_owned()
            ))
        );
    }
}
