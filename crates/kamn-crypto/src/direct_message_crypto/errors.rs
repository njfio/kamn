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
        if let Some(message) = self.static_message() {
            return write!(f, "{message}");
        }

        match self {
            Self::EmptyKeyRef(role) => write!(f, "{role} key reference must not be empty"),
            Self::InvalidKeyRef(role) => {
                write!(f, "{role} key reference must include #key-agreement")
            }
            Self::KeyRefMismatch(role) => write!(f, "{role} key reference mismatch"),
            Self::InvalidNonce(value) => write!(f, "nonce must be positive: {value}"),
            Self::NonceReuse(value) => write!(f, "nonce reuse detected: {value}"),
            _ => unreachable!("static message variants are handled above"),
        }
    }
}

impl std::error::Error for DirectMessageCryptoError {}

impl DirectMessageCryptoError {
    fn static_message(&self) -> Option<&'static str> {
        match self {
            Self::InsecureCryptoDisabled => {
                Some("legacy deterministic direct-message crypto has been removed")
            }
            Self::MissingKeyAgreementMasterSeed => {
                Some("missing required key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX")
            }
            Self::InvalidKeyAgreementMasterSeed => Some(
                "invalid key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX; expected 32-byte hex",
            ),
            Self::EmptyPayload => Some("plaintext payload must not be empty"),
            Self::AlgorithmMismatch => Some("direct message algorithm mismatch"),
            Self::EncryptionFailed => Some("direct message encryption failed"),
            Self::KeyDerivationFailed => Some("direct message key derivation failed"),
            Self::IntegrityCheckFailed => Some("ciphertext integrity check failed"),
            Self::InvalidCiphertextEncoding => Some("invalid ciphertext encoding"),
            _ => None,
        }
    }
}
