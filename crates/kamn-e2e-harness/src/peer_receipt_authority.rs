/// Canonical request evidence supplied by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerRequestAuthority {
    /// Exact canonical request bytes represented as UTF-8.
    pub canonical_body: String,
    /// Peer-claimed request digest.
    pub request_digest: String,
}

/// Payment challenge evidence supplied by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerChallengeAuthority {
    /// Digest of the challenged request.
    pub request_digest: String,
    /// Stable challenge identifier.
    pub challenge_id: String,
    /// Unique challenge nonce.
    pub nonce: String,
    /// Absolute Unix expiry.
    pub expires_at_unix: u64,
    /// Payer identity.
    pub payer: String,
    /// Payee identity.
    pub payee: String,
    /// Asset identifier.
    pub asset: String,
    /// Network identifier.
    pub network: String,
    /// Amount in the asset's minor unit.
    pub amount_minor: u64,
    /// Peer-claimed challenge digest.
    pub challenge_digest: String,
}

/// Approval evidence supplied by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerApprovalAuthority {
    /// Digest of the approved request.
    pub request_digest: String,
    /// Digest of the approved challenge.
    pub challenge_digest: String,
    /// Stable challenge identifier.
    pub challenge_id: String,
    /// Approved challenge nonce.
    pub nonce: String,
    /// Approval Unix timestamp.
    pub approved_at_unix: u64,
    /// Payer identity.
    pub payer: String,
    /// Payee identity.
    pub payee: String,
    /// Asset identifier.
    pub asset: String,
    /// Network identifier.
    pub network: String,
    /// Amount in the asset's minor unit.
    pub amount_minor: u64,
    /// Peer-claimed approval digest.
    pub approval_digest: String,
}

/// Authoritative settlement evidence supplied by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerSettlementAuthority {
    /// Digest of the settled request.
    pub request_digest: String,
    /// Digest of the settled challenge.
    pub challenge_digest: String,
    /// Digest of the settled approval.
    pub approval_digest: String,
    /// Authoritative settlement receipt identifier.
    pub receipt_id: String,
    /// Authoritative network transaction identifier.
    pub transaction_id: String,
    /// Settlement finalization Unix timestamp.
    pub finalized_at_unix: u64,
    /// Payer identity.
    pub payer: String,
    /// Payee identity.
    pub payee: String,
    /// Asset identifier.
    pub asset: String,
    /// Network identifier.
    pub network: String,
    /// Amount in the asset's minor unit.
    pub amount_minor: u64,
    /// Peer-claimed settlement digest.
    pub settlement_digest: String,
}

/// Service-result evidence supplied by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerResultAuthority {
    /// Digest of the request producing the result.
    pub request_digest: String,
    /// Digest of the settlement authorizing the result.
    pub settlement_digest: String,
    /// Result production Unix timestamp.
    pub produced_at_unix: u64,
    /// Exact canonical result bytes represented as UTF-8.
    pub canonical_result: String,
    /// Peer-claimed result digest.
    pub result_digest: String,
}

/// Settlement visibility of one peer observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerSettlementVisibility {
    /// Settlement evidence was observed.
    Observed,
    /// Settlement was intentionally not attempted or observed.
    Blocked,
}

/// Possibly incomplete peer receipt-authority evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerReceiptAuthorityAttempt {
    /// Request evidence.
    pub request: Option<PeerRequestAuthority>,
    /// Challenge evidence.
    pub challenge: Option<PeerChallengeAuthority>,
    /// Approval evidence.
    pub approval: Option<PeerApprovalAuthority>,
    /// Settlement evidence.
    pub settlement: Option<PeerSettlementAuthority>,
    /// Service-result evidence.
    pub service_result: Option<PeerResultAuthority>,
    /// Whether settlement evidence was observable.
    pub settlement_visibility: PeerSettlementVisibility,
}

/// Structured peer receipt-authority validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerReceiptAuthorityError {
    /// Stable machine-readable error code.
    pub code: &'static str,
    /// Human-readable failure description.
    pub message: String,
    /// Failing receipt stage.
    pub stage: &'static str,
    /// Failing field.
    pub field: &'static str,
    /// Non-secret debugging context.
    pub context: String,
    /// Preserved underlying parsing cause when applicable.
    pub cause: Option<String>,
}

/// Verdict for one peer receipt-authority attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerReceiptAuthorityVerdict {
    /// Every stage and binding validated.
    Pass,
    /// Evidence was observed but failed validation.
    Fail(PeerReceiptAuthorityError),
    /// Evidence failed validation and settlement visibility was blocked.
    Blocked(PeerReceiptAuthorityError),
}
