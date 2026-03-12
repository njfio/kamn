mod display;

/// Error surface for group channel sender-key and ciphertext validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupChannelCryptoError {
    /// Compatibility marker retained for existing callers; no longer emitted.
    InsecureCryptoDisabled,
    /// Required key-agreement seed is missing from the environment.
    MissingKeyAgreementMasterSeed,
    /// Required key-agreement seed format is invalid.
    InvalidKeyAgreementMasterSeed,
    /// Channel identifier was empty.
    EmptyChannelId,
    /// Recipient allowlist was empty.
    EmptyRecipients,
    /// Plaintext payload was empty.
    EmptyPayload,
    /// Nonce value was invalid.
    InvalidNonce(u64),
    /// Nonce was already used for the same sender/generation.
    NonceReuse(u64),
    /// DID failed parser validation.
    InvalidDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Sender key reference format was invalid.
    InvalidSenderKeyRef,
    /// Sender DID has no registered sender-key generation.
    SenderKeyNotFound(String),
    /// Sender DID does not have the requested key generation.
    UnknownSenderKeyGeneration {
        /// Sender DID requested.
        sender_did: String,
        /// Sender-key generation requested.
        key_generation: u64,
    },
    /// Recipient DID is not in the distribution allowlist.
    RecipientNotAuthorized {
        /// Recipient DID that attempted decryption.
        recipient_did: String,
        /// Sender DID that produced the ciphertext.
        sender_did: String,
        /// Sender-key generation used by the ciphertext.
        key_generation: u64,
    },
    /// Ciphertext declared unsupported algorithm identifiers.
    AlgorithmMismatch,
    /// Ciphertext channel identifier does not match engine channel.
    ChannelMismatch {
        /// Channel identifier expected by this engine.
        expected: String,
        /// Channel identifier supplied by ciphertext.
        actual: String,
    },
    /// Signature check failed for ciphertext provenance.
    SignatureMismatch,
    /// Encryption failed.
    EncryptionFailed,
    /// HKDF key derivation failed.
    KeyDerivationFailed,
    /// Integrity tag verification failed.
    IntegrityCheckFailed,
    /// Ciphertext encoding could not be decoded as valid hex.
    InvalidCiphertextEncoding,
}

impl std::error::Error for GroupChannelCryptoError {}
