/// Visibility request for one escrow-scoped message lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowVisibilityRequest {
    /// Escrow identifier being accessed.
    pub escrow_id: String,
    /// Requester DID.
    pub requester_did: String,
    /// Optional number of reconstructed auditor shares presented by requester.
    pub reconstructed_auditor_shares: Option<u8>,
}

/// Visibility decision for escrow-scoped message lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4EscrowVisibilityDecision {
    /// Access allowed with reason code.
    /// Allow variant for this public contract enum.
    Allow {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
    /// Access denied with reason code.
    /// Deny variant for this public contract enum.
    Deny {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
}
