/// Canonical algorithm identifier for supported baseline signatures.
pub const BASELINE_SIGNATURE_ALGORITHM: &str = "deterministic-v1";
/// Canonical profile identifier for supported baseline signatures.
pub const BASELINE_SIGNATURE_PROFILE_ID: &str = "baseline-v1";
/// Legacy unversioned profile identifier retained for compatibility fixtures.
pub const LEGACY_SIGNATURE_PROFILE_ID: &str = "legacy-unversioned";
/// Canonical unsupported algorithm identifier used in negative fixtures.
pub const UNKNOWN_SIGNATURE_ALGORITHM_ID: &str = "unknown-algorithm";
/// Canonical algorithm identifier for service-auth cryptographic signatures.
pub const SERVICE_AUTH_SIGNATURE_ALGORITHM: &str = "secp256k1";
/// Canonical profile identifier for service-auth cryptographic signatures.
pub const SERVICE_AUTH_SIGNATURE_PROFILE_ID: &str = "baseline-v2";
/// Environment variable that carries service-auth private key material (hex).
pub const SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
/// Environment variable that carries service-auth public key material (hex).
pub const SERVICE_AUTH_SIGNATURE_PUBLIC_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX";

/// Error taxonomy for service-auth cryptographic signing and verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceAuthSignatureError {
    /// Empty field variant for this public contract enum.
    EmptyField(&'static str),
    /// Invalid nonce variant for this public contract enum.
    InvalidNonce,
    /// Invalid signature format variant for this public contract enum.
    InvalidSignatureFormat,
    /// Unsupported algorithm variant for this public contract enum.
    UnsupportedAlgorithm(String),
    /// Unsupported profile variant for this public contract enum.
    UnsupportedProfile(String),
    /// Invalid recovery id variant for this public contract enum.
    InvalidRecoveryId,
    /// Invalid signature hex variant for this public contract enum.
    InvalidSignatureHex,
    /// Invalid private key hex variant for this public contract enum.
    InvalidPrivateKeyHex,
    /// Invalid public key hex variant for this public contract enum.
    InvalidPublicKeyHex,
    /// Signing failure variant for this public contract enum.
    SigningFailure,
    /// Verification failure variant for this public contract enum.
    VerificationFailure,
}

impl std::fmt::Display for ServiceAuthSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidNonce => write!(f, "nonce must be positive"),
            Self::InvalidSignatureFormat => write!(f, "signature format is invalid"),
            Self::UnsupportedAlgorithm(value) => {
                write!(f, "unsupported signature algorithm: {value}")
            }
            Self::UnsupportedProfile(value) => write!(f, "unsupported signature profile: {value}"),
            Self::InvalidRecoveryId => write!(f, "signature recovery id is invalid"),
            Self::InvalidSignatureHex => write!(f, "signature hex payload is invalid"),
            Self::InvalidPrivateKeyHex => write!(f, "private key hex payload is invalid"),
            Self::InvalidPublicKeyHex => write!(f, "public key hex payload is invalid"),
            Self::SigningFailure => write!(f, "failed to sign canonical payload"),
            Self::VerificationFailure => write!(f, "failed to verify canonical payload"),
        }
    }
}

impl std::error::Error for ServiceAuthSignatureError {}

/// Parsed metadata extracted from a `sig:<algorithm>:<profile_id>:...` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureProfileMetadata {
    /// Algorithm carried by this public contract model.
    pub algorithm: String,
    /// Profile id carried by this public contract model.
    pub profile_id: String,
}

/// Runs the baseline signature algorithm contract helper.
pub fn baseline_signature_algorithm() -> &'static str {
    BASELINE_SIGNATURE_ALGORITHM
}

/// Runs the baseline signature profile id contract helper.
pub fn baseline_signature_profile_id() -> &'static str {
    BASELINE_SIGNATURE_PROFILE_ID
}
