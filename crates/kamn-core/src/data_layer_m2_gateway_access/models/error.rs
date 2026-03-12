use std::fmt;

/// Error taxonomy for M2 gateway authn/authz/audit contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2GatewayError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID value failed validation.
    InvalidDid {
        /// Input field carrying DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Credential payload failed deterministic validation.
    InvalidCredential(String),
    /// Session TTL is invalid.
    InvalidSessionTtl {
        /// Requested TTL.
        ttl_seconds: u64,
        /// Maximum allowed TTL.
        max_ttl_seconds: u64,
    },
    /// Expiry computation overflowed u64 bounds.
    SessionExpiryOverflow,
    /// Access-audit hash chain failed integrity checks.
    InvalidAuditHashChain {
        /// Zero-based record position.
        position: usize,
        /// Deterministic mismatch reason marker.
        reason: &'static str,
    },
    /// Access-audit sequence number not found.
    AuditSequenceNotFound(u64),
    /// Negative authorization matrix input failed fail-closed validation.
    InvalidNegativeAuthorizationMatrix(&'static str),
}

impl fmt::Display for DataLayerM2GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidCredential(reason) => write!(f, "invalid credential: {reason}"),
            Self::InvalidSessionTtl {
                ttl_seconds,
                max_ttl_seconds,
            } => write!(
                f,
                "invalid session ttl: requested {ttl_seconds}, max {max_ttl_seconds}"
            ),
            Self::SessionExpiryOverflow => write!(f, "session expiry overflow"),
            Self::InvalidAuditHashChain { position, reason } => {
                write!(
                    f,
                    "invalid audit hash chain at position {position}: {reason}"
                )
            }
            Self::AuditSequenceNotFound(sequence) => {
                write!(f, "audit sequence not found: {sequence}")
            }
            Self::InvalidNegativeAuthorizationMatrix(field) => {
                write!(f, "invalid negative authorization matrix input: {field}")
            }
        }
    }
}

impl std::error::Error for DataLayerM2GatewayError {}
