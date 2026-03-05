//! M10 compliance projection seam contracts.
//!
//! This module defines a core-agnostic port for owner-scope authorization and
//! message compliance lookups used by M10 shred-completeness projection.

use std::fmt;

/// Message compliance state required by M10 shred-completeness projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ComplianceProjectionMessageState {
    /// Stable message identifier.
    pub message_id: String,
    /// True when legal hold is active and archival must remain blocked.
    pub legal_hold_active: bool,
    /// Shredded timestamp when crypto-shredding completed.
    pub shredded_at_epoch_seconds: Option<u64>,
}

/// Typed fail-closed errors from M10 compliance projection ports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10ComplianceProjectionPortError {
    /// Owner-scope authorization denied.
    OwnerScopeViolation,
    /// Owner/message lookup failed.
    LookupFailed(String),
    /// Input validation failed.
    InvalidInput(String),
}

impl fmt::Display for DataLayerM10ComplianceProjectionPortError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerScopeViolation => write!(formatter, "owner scope violation"),
            Self::LookupFailed(detail) => write!(formatter, "lookup failed: {detail}"),
            Self::InvalidInput(detail) => write!(formatter, "invalid input: {detail}"),
        }
    }
}

impl std::error::Error for DataLayerM10ComplianceProjectionPortError {}

/// Core-agnostic projection port for M10 shred-completeness decisions.
pub trait DataLayerM10ComplianceProjectionPort {
    /// Authorizes owner scope and returns a normalized owner DID.
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10ComplianceProjectionPortError>;

    /// Returns the compliance message state for one owner/message tuple.
    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPortError>;
}
