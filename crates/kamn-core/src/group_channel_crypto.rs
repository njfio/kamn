//! Group channel sender-key lifecycle and message protection contracts.

use crate::AgentDid;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256, Sha512};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use x25519_dalek::{PublicKey, StaticSecret};

/// Key-derivation algorithm identifier stamped on group ciphertext envelopes.
pub const GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM: &str = "X25519";
/// Cipher profile identifier stamped on group ciphertext envelopes.
pub const GROUP_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE: &str =
    "group_channel_crypto_invalid_sender_did";
const GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE: &str =
    "group_channel_crypto_invalid_recipient_did";

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

/// In-memory engine for sender-key distribution, rotation, and message sealing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupChannelCryptoEngine {
    channel_id: String,
    sender_key_history: BTreeMap<String, BTreeMap<u64, SenderKeyDistributionRecord>>,
    active_generation_by_sender: BTreeMap<String, u64>,
    used_nonces: BTreeSet<(String, u64, u64)>,
}

impl GroupChannelCryptoEngine {
    /// Constructs an engine for a specific channel identifier.
    pub fn new(channel_id: &str) -> Result<Self, GroupChannelCryptoError> {
        if channel_id.trim().is_empty() {
            return Err(GroupChannelCryptoError::EmptyChannelId);
        }

        Ok(Self {
            channel_id: channel_id.to_owned(),
            sender_key_history: BTreeMap::new(),
            active_generation_by_sender: BTreeMap::new(),
            used_nonces: BTreeSet::new(),
        })
    }

    /// Distributes a new sender-key generation and marks the previous one inactive.
    pub fn distribute_sender_key(
        &mut self,
        sender_did: &str,
        sender_key_ref: &str,
        recipients: Vec<String>,
    ) -> Result<SenderKeyDistributionRecord, GroupChannelCryptoError> {
        validate_did(
            sender_did,
            "sender_did",
            GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE,
        )?;
        validate_sender_key_ref(sender_key_ref)?;

        let recipient_allowlist = validate_recipients(recipients)?;
        let next_generation = self
            .active_generation_by_sender
            .get(sender_did)
            .copied()
            .unwrap_or(0)
            + 1;

        if let Some(history) = self.sender_key_history.get_mut(sender_did) {
            if let Some(active_generation) = self.active_generation_by_sender.get(sender_did) {
                if let Some(active_record) = history.get_mut(active_generation) {
                    active_record.active = false;
                }
            }
        }

        let record = SenderKeyDistributionRecord {
            channel_id: self.channel_id.clone(),
            sender_did: sender_did.to_owned(),
            sender_key_ref: sender_key_ref.to_owned(),
            key_generation: next_generation,
            recipient_allowlist,
            active: true,
        };

        self.sender_key_history
            .entry(sender_did.to_owned())
            .or_default()
            .insert(next_generation, record.clone());
        self.active_generation_by_sender
            .insert(sender_did.to_owned(), next_generation);

        Ok(record)
    }

    /// Rotates sender-key material by issuing a new distribution generation.
    pub fn rotate_sender_key(
        &mut self,
        sender_did: &str,
        sender_key_ref: &str,
        recipients: Vec<String>,
    ) -> Result<SenderKeyDistributionRecord, GroupChannelCryptoError> {
        self.distribute_sender_key(sender_did, sender_key_ref, recipients)
    }

    /// Returns the active sender-key generation for a sender DID.
    pub fn active_sender_key_generation(
        &self,
        sender_did: &str,
    ) -> Result<u64, GroupChannelCryptoError> {
        self.active_generation_by_sender
            .get(sender_did)
            .copied()
            .ok_or_else(|| GroupChannelCryptoError::SenderKeyNotFound(sender_did.to_owned()))
    }

    /// Returns a sender-key distribution record for a specific generation.
    pub fn sender_key_record(
        &self,
        sender_did: &str,
        key_generation: u64,
    ) -> Result<&SenderKeyDistributionRecord, GroupChannelCryptoError> {
        self.sender_key_history
            .get(sender_did)
            .and_then(|history| history.get(&key_generation))
            .ok_or_else(|| GroupChannelCryptoError::UnknownSenderKeyGeneration {
                sender_did: sender_did.to_owned(),
                key_generation,
            })
    }

    /// Encrypts plaintext for a sender using the active sender-key generation.
    pub fn encrypt(
        &mut self,
        sender_did: &str,
        plaintext: &str,
        nonce: u64,
    ) -> Result<GroupMessageCiphertext, GroupChannelCryptoError> {
        validate_did(
            sender_did,
            "sender_did",
            GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE,
        )?;
        if plaintext.is_empty() {
            return Err(GroupChannelCryptoError::EmptyPayload);
        }
        if nonce == 0 {
            return Err(GroupChannelCryptoError::InvalidNonce(nonce));
        }

        let active_generation = self.active_sender_key_generation(sender_did)?;
        let record = self
            .sender_key_record(sender_did, active_generation)?
            .clone();

        let nonce_key = (sender_did.to_owned(), record.key_generation, nonce);
        if !self.used_nonces.insert(nonce_key) {
            return Err(GroupChannelCryptoError::NonceReuse(nonce));
        }

        let master_seed = load_key_agreement_master_seed()?;
        let shared_secret = derive_group_shared_secret(
            self.channel_id.as_str(),
            record.sender_key_ref.as_str(),
            record.key_generation,
            &master_seed,
        );
        let aead_key = derive_group_aead_key(
            &shared_secret,
            self.channel_id.as_str(),
            record.key_generation,
        );

        let aad: [u8; 0] = [];
        let nonce_bytes = group_nonce_bytes(sender_did, record.key_generation, nonce);
        let xnonce = XNonce::from(nonce_bytes);
        let cipher = XChaCha20Poly1305::new((&aead_key).into());

        let mut sealed = cipher
            .encrypt(
                &xnonce,
                Payload {
                    msg: plaintext.as_bytes(),
                    aad: &aad,
                },
            )
            .map_err(|_| GroupChannelCryptoError::EncryptionFailed)?;

        let auth_tag = sealed.split_off(sealed.len() - 16);
        let ciphertext_hex = hex_encode(&sealed);
        let auth_tag_hex = hex_encode(&auth_tag);

        let signature = compute_signature(
            &shared_secret,
            &self.channel_id,
            sender_did,
            record.key_generation,
            nonce,
            ciphertext_hex.as_str(),
            auth_tag_hex.as_str(),
        );

        Ok(GroupMessageCiphertext {
            key_derivation_algorithm: GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM.to_owned(),
            cipher_algorithm: GROUP_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            channel_id: self.channel_id.clone(),
            sender_did: sender_did.to_owned(),
            key_generation: record.key_generation,
            nonce,
            ciphertext: ciphertext_hex,
            auth_tag: auth_tag_hex,
            signature,
        })
    }

    /// Decrypts a sealed group message for an authorized recipient DID.
    pub fn decrypt(
        &self,
        recipient_did: &str,
        sealed: &GroupMessageCiphertext,
    ) -> Result<String, GroupChannelCryptoError> {
        validate_did(
            recipient_did,
            "recipient_did",
            GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE,
        )?;
        if sealed.key_derivation_algorithm != GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM
            || sealed.cipher_algorithm != GROUP_MESSAGE_CIPHER_ALGORITHM
        {
            return Err(GroupChannelCryptoError::AlgorithmMismatch);
        }
        if sealed.channel_id != self.channel_id {
            return Err(GroupChannelCryptoError::ChannelMismatch {
                expected: self.channel_id.clone(),
                actual: sealed.channel_id.clone(),
            });
        }

        let record = self.sender_key_record(&sealed.sender_did, sealed.key_generation)?;
        if !record.recipient_allowlist.contains(recipient_did) {
            return Err(GroupChannelCryptoError::RecipientNotAuthorized {
                recipient_did: recipient_did.to_owned(),
                sender_did: sealed.sender_did.clone(),
                key_generation: sealed.key_generation,
            });
        }

        let master_seed = load_key_agreement_master_seed()?;
        let shared_secret = derive_group_shared_secret(
            self.channel_id.as_str(),
            record.sender_key_ref.as_str(),
            sealed.key_generation,
            &master_seed,
        );

        let expected_signature = compute_signature(
            &shared_secret,
            &sealed.channel_id,
            &sealed.sender_did,
            sealed.key_generation,
            sealed.nonce,
            &sealed.ciphertext,
            &sealed.auth_tag,
        );
        if expected_signature != sealed.signature {
            return Err(GroupChannelCryptoError::SignatureMismatch);
        }

        let ciphertext = hex_decode(&sealed.ciphertext)?;
        let auth_tag = hex_decode(&sealed.auth_tag)
            .map_err(|_| GroupChannelCryptoError::IntegrityCheckFailed)?;

        let mut combined = ciphertext;
        combined.extend_from_slice(&auth_tag);

        let aad: [u8; 0] = [];
        let nonce_bytes = group_nonce_bytes(
            sealed.sender_did.as_str(),
            sealed.key_generation,
            sealed.nonce,
        );
        let xnonce = XNonce::from(nonce_bytes);

        let aead_key = derive_group_aead_key(
            &shared_secret,
            self.channel_id.as_str(),
            sealed.key_generation,
        );
        let cipher = XChaCha20Poly1305::new((&aead_key).into());

        let plaintext = cipher
            .decrypt(
                &xnonce,
                Payload {
                    msg: &combined,
                    aad: &aad,
                },
            )
            .map_err(|_| GroupChannelCryptoError::IntegrityCheckFailed)?;

        String::from_utf8(plaintext).map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)
    }
}

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
    /// Integrity tag verification failed.
    IntegrityCheckFailed,
    /// Ciphertext encoding could not be decoded as valid hex.
    InvalidCiphertextEncoding,
}

impl fmt::Display for GroupChannelCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsecureCryptoDisabled => {
                write!(f, "legacy deterministic group-message crypto has been removed")
            }
            Self::MissingKeyAgreementMasterSeed => write!(
                f,
                "missing required key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX"
            ),
            Self::InvalidKeyAgreementMasterSeed => write!(
                f,
                "invalid key-agreement seed KAMN_KEY_AGREEMENT_MASTER_SEED_HEX; expected 32-byte hex"
            ),
            Self::EmptyChannelId => write!(f, "channel_id must not be empty"),
            Self::EmptyRecipients => write!(f, "recipient allowlist must not be empty"),
            Self::EmptyPayload => write!(f, "plaintext payload must not be empty"),
            Self::InvalidNonce(value) => write!(f, "nonce must be positive: {value}"),
            Self::NonceReuse(value) => write!(f, "nonce reuse detected: {value}"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidSenderKeyRef => {
                write!(f, "sender key reference must include #sender-key-")
            }
            Self::SenderKeyNotFound(value) => write!(f, "sender key not found: {value}"),
            Self::UnknownSenderKeyGeneration {
                sender_did,
                key_generation,
            } => write!(
                f,
                "unknown sender key generation {key_generation} for {sender_did}"
            ),
            Self::RecipientNotAuthorized {
                recipient_did,
                sender_did,
                key_generation,
            } => write!(
                f,
                "recipient {recipient_did} is not authorized for {sender_did} generation {key_generation}"
            ),
            Self::AlgorithmMismatch => write!(f, "group message algorithm mismatch"),
            Self::ChannelMismatch { expected, actual } => {
                write!(f, "group message channel mismatch, expected {expected}, got {actual}")
            }
            Self::SignatureMismatch => write!(f, "group message signature verification failed"),
            Self::EncryptionFailed => write!(f, "group message encryption failed"),
            Self::IntegrityCheckFailed => write!(f, "group message integrity check failed"),
            Self::InvalidCiphertextEncoding => write!(f, "invalid ciphertext encoding"),
        }
    }
}

impl std::error::Error for GroupChannelCryptoError {}

fn validate_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, GroupChannelCryptoError> {
    AgentDid::parse(value).map_err(|error| GroupChannelCryptoError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

fn validate_sender_key_ref(value: &str) -> Result<(), GroupChannelCryptoError> {
    if !value.contains("#sender-key-") {
        return Err(GroupChannelCryptoError::InvalidSenderKeyRef);
    }
    Ok(())
}

fn load_key_agreement_master_seed() -> Result<[u8; 32], GroupChannelCryptoError> {
    let seed_hex = env::var(KEY_AGREEMENT_MASTER_SEED_ENV)
        .map_err(|_| GroupChannelCryptoError::MissingKeyAgreementMasterSeed)?;
    parse_fixed_hex_32(seed_hex.trim())
}

fn parse_fixed_hex_32(value: &str) -> Result<[u8; 32], GroupChannelCryptoError> {
    if value.len() != 64 {
        return Err(GroupChannelCryptoError::InvalidKeyAgreementMasterSeed);
    }
    let mut out = [0u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let encoded = std::str::from_utf8(chunk)
            .map_err(|_| GroupChannelCryptoError::InvalidKeyAgreementMasterSeed)?;
        let byte = u8::from_str_radix(encoded, 16)
            .map_err(|_| GroupChannelCryptoError::InvalidKeyAgreementMasterSeed)?;
        out[index] = byte;
    }
    Ok(out)
}

fn validate_recipients(
    recipients: Vec<String>,
) -> Result<BTreeSet<String>, GroupChannelCryptoError> {
    if recipients.is_empty() {
        return Err(GroupChannelCryptoError::EmptyRecipients);
    }

    let mut allowlist = BTreeSet::new();
    for recipient in recipients {
        validate_did(
            &recipient,
            "recipients[]",
            GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE,
        )?;
        allowlist.insert(recipient);
    }
    Ok(allowlist)
}

fn derive_group_shared_secret(
    channel_id: &str,
    sender_key_ref: &str,
    generation: u64,
    master_seed: &[u8; 32],
) -> [u8; 32] {
    let sender_private = derive_x25519_private_key(master_seed, sender_key_ref);
    let channel_material_ref = format!("{channel_id}#group-key-material-{generation}");
    let channel_public = derive_x25519_public_key(master_seed, channel_material_ref.as_str());
    sender_private.diffie_hellman(&channel_public).to_bytes()
}

fn derive_group_aead_key(shared_secret: &[u8; 32], channel_id: &str, generation: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"kamn:group-message:aead-key:v1:");
    hasher.update(shared_secret);
    hasher.update(channel_id.as_bytes());
    hasher.update(generation.to_le_bytes());
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest[..32]);
    key
}

fn derive_x25519_private_key(master_seed: &[u8; 32], key_ref: &str) -> StaticSecret {
    let mut hasher = Sha512::new();
    hasher.update(b"kamn:x25519:key-ref:v1:");
    hasher.update(master_seed);
    hasher.update(key_ref.as_bytes());
    let digest = hasher.finalize();
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&digest[..32]);
    StaticSecret::from(key_bytes)
}

fn derive_x25519_public_key(master_seed: &[u8; 32], key_ref: &str) -> PublicKey {
    let private_key = derive_x25519_private_key(master_seed, key_ref);
    PublicKey::from(&private_key)
}

fn group_nonce_bytes(sender_did: &str, generation: u64, nonce: u64) -> [u8; 24] {
    let mut out = [0u8; 24];
    out[..8].copy_from_slice(&nonce.to_le_bytes());

    let mut hasher = Sha256::new();
    hasher.update(b"kamn:group-message:nonce:v1:");
    hasher.update(sender_did.as_bytes());
    hasher.update(generation.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    out[8..].copy_from_slice(&digest[..16]);
    out
}

fn compute_signature(
    shared_secret: &[u8; 32],
    channel_id: &str,
    sender_did: &str,
    generation: u64,
    nonce: u64,
    ciphertext: &str,
    auth_tag: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"kamn:group-message:signature:v2:");
    hasher.update(shared_secret);
    hasher.update(channel_id.as_bytes());
    hasher.update(sender_did.as_bytes());
    hasher.update(generation.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    hasher.update(ciphertext.as_bytes());
    hasher.update(auth_tag.as_bytes());
    let digest = hasher.finalize();
    format!("sig:sha256:{}", hex_encode(&digest))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

fn hex_decode(value: &str) -> Result<Vec<u8>, GroupChannelCryptoError> {
    if !value.len().is_multiple_of(2) {
        return Err(GroupChannelCryptoError::InvalidCiphertextEncoding);
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for chunk in value.as_bytes().chunks_exact(2) {
        let encoded = std::str::from_utf8(chunk)
            .map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)?;
        let byte = u8::from_str_radix(encoded, 16)
            .map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)?;
        bytes.push(byte);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{GroupChannelCryptoEngine, GroupChannelCryptoError};
    use std::sync::{Mutex, OnceLock};

    const TEST_KEY_SEED_HEX: &str =
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn with_key_agreement_seed<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock should not be poisoned");

        let previous = std::env::var(super::KEY_AGREEMENT_MASTER_SEED_ENV).ok();
        match value {
            Some(seed) => std::env::set_var(super::KEY_AGREEMENT_MASTER_SEED_ENV, seed),
            None => std::env::remove_var(super::KEY_AGREEMENT_MASTER_SEED_ENV),
        }
        let output = run();
        match previous {
            Some(seed) => std::env::set_var(super::KEY_AGREEMENT_MASTER_SEED_ENV, seed),
            None => std::env::remove_var(super::KEY_AGREEMENT_MASTER_SEED_ENV),
        }
        output
    }

    #[test]
    fn constructor_rejects_empty_channel_id() {
        assert_eq!(
            GroupChannelCryptoEngine::new(""),
            Err(GroupChannelCryptoError::EmptyChannelId)
        );
    }

    #[test]
    fn distribution_rejects_empty_recipients() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            assert_eq!(
                engine.distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    Vec::new(),
                ),
                Err(GroupChannelCryptoError::EmptyRecipients)
            );
        });
    }

    #[test]
    fn rotate_marks_previous_generation_inactive() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            let first = engine
                .distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("first distribution should succeed");

            let second = engine
                .rotate_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-2",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("rotation should succeed");

            let first_record = engine
                .sender_key_record("kamn:did:agent:alice", first.key_generation)
                .expect("first generation should exist");
            let second_record = engine
                .sender_key_record("kamn:did:agent:alice", second.key_generation)
                .expect("second generation should exist");

            assert!(!first_record.active);
            assert!(second_record.active);
        });
    }

    #[test]
    fn regression_constructor_accepts_without_insecure_fixture_opt_in() {
        // Regression: #5921
        let result = GroupChannelCryptoEngine::new("channel:group:locked");
        assert!(result.is_ok());
    }

    #[test]
    fn encrypt_requires_key_agreement_seed() {
        with_key_agreement_seed(None, || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            engine
                .distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("distribution should succeed");

            let result = engine.encrypt("kamn:did:agent:alice", "payload", 1);

            assert_eq!(
                result,
                Err(GroupChannelCryptoError::MissingKeyAgreementMasterSeed)
            );
        });
    }

    #[test]
    fn display_messages_remain_stable_for_reason_taxonomy() {
        assert_eq!(
            GroupChannelCryptoError::EmptyChannelId.to_string(),
            "channel_id must not be empty"
        );
    }
}
