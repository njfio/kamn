use std::collections::BTreeSet;

/// Persisted sender-key distribution event for one sender generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderKeyDistributionRecord {
    /// Channel identifier this sender-key distribution belongs to.
    pub channel_id: String,
    /// Sender DID that owns the sender-key generation.
    pub sender_did: String,
    /// Sender key reference used to derive message secrets.
    pub sender_key_ref: String,
    /// Monotonic generation counter for sender-key rotations.
    pub key_generation: u64,
    /// Recipients authorized to decrypt ciphertext from this generation.
    pub recipient_allowlist: BTreeSet<String>,
    /// Whether this generation is currently active for encryption.
    pub active: bool,
}

/// Encrypted group message envelope carrying sender-key metadata and proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMessageCiphertext {
    /// Declared key-derivation algorithm for decryptor compatibility checks.
    pub key_derivation_algorithm: String,
    /// Declared cipher algorithm for decryptor compatibility checks.
    pub cipher_algorithm: String,
    /// Channel identifier this ciphertext targets.
    pub channel_id: String,
    /// Sender DID that produced the ciphertext.
    pub sender_did: String,
    /// Sender-key generation used to derive this ciphertext.
    pub key_generation: u64,
    /// Nonce value used for this encryption operation.
    pub nonce: u64,
    /// Hex-encoded encrypted payload bytes.
    pub ciphertext: String,
    /// Hex-encoded 16-byte Poly1305 authentication tag.
    pub auth_tag: String,
    /// Deterministic provenance signature token.
    pub signature: String,
}
