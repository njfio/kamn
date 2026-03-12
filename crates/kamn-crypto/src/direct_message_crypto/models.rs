/// Encrypted direct-message payload and metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectMessageCiphertext {
    /// Key agreement algorithm used to derive the shared secret.
    pub key_agreement_algorithm: String,
    /// Cipher algorithm used to encrypt the payload.
    pub cipher_algorithm: String,
    /// Sender key reference used in key agreement.
    pub sender_key_ref: String,
    /// Recipient key reference used in key agreement.
    pub recipient_key_ref: String,
    /// Nonce used for encryption.
    pub nonce: u64,
    /// Hex-encoded ciphertext bytes.
    pub ciphertext: String,
    /// Hex-encoded 16-byte Poly1305 authentication tag.
    pub auth_tag: String,
}
