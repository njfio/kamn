//! M10 Phase-6 compliance seam contracts.
//!
//! This module defines core-agnostic interfaces used by M10 Phase-6 retention,
//! shredding, and projection orchestration paths.

use std::fmt;

use super::DataLayerM10ComplianceProjectionMessageState;

/// Due-candidate record returned by Phase-6 compliance lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6RetentionDueCandidate {
    /// Stable message identifier.
    pub message_id: String,
}

/// Crypto-shred input contract for one message in Phase-6 execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6CryptoShredInput {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Message identifier.
    pub message_id: String,
    /// Shredded timestamp in epoch seconds.
    pub shredded_at_epoch_seconds: u64,
}

/// Typed fail-closed errors from the M10 Phase-6 compliance seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10Phase6CompliancePortError {
    /// Owner-scope authorization denied.
    OwnerScopeViolation,
    /// Data lookup failed.
    LookupFailed(String),
    /// Input validation failed.
    InvalidInput(String),
}

impl fmt::Display for DataLayerM10Phase6CompliancePortError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerScopeViolation => write!(formatter, "owner scope violation"),
            Self::LookupFailed(detail) => write!(formatter, "lookup failed: {detail}"),
            Self::InvalidInput(detail) => write!(formatter, "invalid input: {detail}"),
        }
    }
}

impl std::error::Error for DataLayerM10Phase6CompliancePortError {}

/// Core-agnostic compliance seam for M10 Phase-6 orchestration.
pub trait DataLayerM10Phase6CompliancePort {
    /// Authorizes owner scope and returns a normalized owner DID.
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10Phase6CompliancePortError>;

    /// Returns retention-due candidates for the owner scope.
    fn retention_due_for_owner(
        &self,
        owner_did: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM10Phase6RetentionDueCandidate>, DataLayerM10Phase6CompliancePortError>;

    /// Applies crypto-shred for one message.
    fn crypto_shred(
        &mut self,
        input: DataLayerM10Phase6CryptoShredInput,
    ) -> Result<(), DataLayerM10Phase6CompliancePortError>;

    /// Returns compliance message state for one owner/message tuple.
    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10Phase6CompliancePortError>;
}
