use std::fmt;

/// Errors emitted by direct-message crypto construction and processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectMessageCryptoError {
    /// Compatibility marker retained for existing callers; no longer emitted.
    InsecureCryptoDisabled,
    /// Required key-agreement seed is missing from the environment.
    MissingKeyAgreementMasterSeed,
    /// Required key-agreement seed format is invalid.
    InvalidKeyAgreementMasterSeed,
    /// Key reference for role was empty.
    EmptyKeyRef(&'static str),
    /// Key reference for role did not match expected shape.
    InvalidKeyRef(&'static str),
    /// Ciphertext key reference does not match decrypt engine context.
    KeyRefMismatch(&'static str),
    /// Plaintext payload was empty.
    EmptyPayload,
    /// Nonce value was invalid.
    InvalidNonce(u64),
    /// Nonce was reused.
    NonceReuse(u64),
    /// Ciphertext algorithm metadata did not match expected algorithms.
    AlgorithmMismatch,
    /// Encryption failed.
    EncryptionFailed,
    /// HKDF key derivation failed.
    KeyDerivationFailed,
    /// Ciphertext integrity verification failed.
    IntegrityCheckFailed,
    /// Ciphertext bytes were not valid hex or UTF-8 output.
    InvalidCiphertextEncoding,
}

impl fmt::Display for DirectMessageCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsecureCryptoDisabled => write!(
                f,
                "legacy deterministic direct-message crypto has been removed"
            ),
            Self::MissingKeyAgreementMasterSeed => write!(
                f,
                "missing required key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX"
            ),
            Self::InvalidKeyAgreementMasterSeed => write!(
                f,
                "invalid key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX; expected 32-byte hex"
            ),
            Self::EmptyKeyRef(role) => write!(f, "{role} key reference must not be empty"),
            Self::InvalidKeyRef(role) => {
                write!(f, "{role} key reference must include #key-agreement")
            }
            Self::KeyRefMismatch(role) => write!(f, "{role} key reference mismatch"),
            Self::EmptyPayload => write!(f, "plaintext payload must not be empty"),
            Self::InvalidNonce(value) => write!(f, "nonce must be positive: {value}"),
            Self::NonceReuse(value) => write!(f, "nonce reuse detected: {value}"),
            Self::AlgorithmMismatch => write!(f, "direct message algorithm mismatch"),
            Self::EncryptionFailed => write!(f, "direct message encryption failed"),
            Self::KeyDerivationFailed => write!(f, "direct message key derivation failed"),
            Self::IntegrityCheckFailed => write!(f, "ciphertext integrity check failed"),
            Self::InvalidCiphertextEncoding => write!(f, "invalid ciphertext encoding"),
        }
    }
}

impl std::error::Error for DirectMessageCryptoError {}
