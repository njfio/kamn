//! Group channel sender-key lifecycle and message protection contracts.

use crate::AgentDid;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256, Sha512};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

/// Key-derivation algorithm identifier stamped on group ciphertext envelopes.
pub const GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM: &str = "X25519";
/// Cipher profile identifier stamped on group ciphertext envelopes.
pub const GROUP_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const GROUP_MESSAGE_AEAD_KDF_SALT_V2: &[u8] = b"kamn:group-message:aead-key:hkdf-salt:v2";
const GROUP_MESSAGE_AEAD_KDF_INFO_PREFIX_V2: &[u8] = b"kamn:group-message:aead-key:hkdf-info:v2:";
const GROUP_MESSAGE_NONCE_INFO_V2: &[u8] = b"kamn:group-message:nonce:v2:";
const GROUP_MESSAGE_NONCE_INFO_V1: &[u8] = b"kamn:group-message:nonce:v1:";
/// Marker asserting HKDF derivation is backed by RustCrypto hkdf crate.
pub const GROUP_MESSAGE_HKDF_BACKEND_MARKER: &str =
    kamn_crypto::hkdf_sha256::HKDF_SHA256_BACKEND_MARKER;
/// Marker asserting HMAC backend semantics are provided by RustCrypto primitives.
pub const GROUP_MESSAGE_HMAC_BACKEND_MARKER: &str =
    kamn_crypto::hkdf_sha256::HMAC_SHA256_BACKEND_MARKER;
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
#[derive(PartialEq, Eq)]
pub struct GroupChannelCryptoEngine {
    channel_id: String,
    sender_key_history: BTreeMap<String, BTreeMap<u64, SenderKeyDistributionRecord>>,
    active_generation_by_sender: BTreeMap<String, u64>,
    used_nonces: BTreeSet<(String, u64, u64)>,
    cached_master_seed: RefCell<Option<[u8; 32]>>,
}

impl fmt::Debug for GroupChannelCryptoEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupChannelCryptoEngine")
            .field("channel_id", &self.channel_id)
            .field("sender_count", &self.sender_key_history.len())
            .field(
                "active_sender_count",
                &self.active_generation_by_sender.len(),
            )
            .field("used_nonce_count", &self.used_nonces.len())
            .finish()
    }
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
            cached_master_seed: RefCell::new(None),
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

        let master_seed = self.cached_master_seed()?;
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
        )?;

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
        if sealed.nonce == 0 {
            return Err(GroupChannelCryptoError::InvalidNonce(sealed.nonce));
        }

        let record = self.sender_key_record(&sealed.sender_did, sealed.key_generation)?;
        if !record.recipient_allowlist.contains(recipient_did) {
            return Err(GroupChannelCryptoError::RecipientNotAuthorized {
                recipient_did: recipient_did.to_owned(),
                sender_did: sealed.sender_did.clone(),
                key_generation: sealed.key_generation,
            });
        }

        let master_seed = self.cached_master_seed()?;
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
        if !crate::constant_time_eq::constant_time_eq_str(
            expected_signature.as_str(),
            sealed.signature.as_str(),
        ) {
            return Err(GroupChannelCryptoError::SignatureMismatch);
        }

        let ciphertext = hex_decode(&sealed.ciphertext)?;
        let auth_tag = hex_decode(&sealed.auth_tag)
            .map_err(|_| GroupChannelCryptoError::IntegrityCheckFailed)?;

        let mut combined = ciphertext;
        combined.extend_from_slice(&auth_tag);

        let aad: [u8; 0] = [];
        let aead_key_v2 = derive_group_aead_key(
            &shared_secret,
            self.channel_id.as_str(),
            sealed.key_generation,
        )?;
        let aead_key_v1 = derive_group_aead_key_legacy(
            &shared_secret,
            self.channel_id.as_str(),
            sealed.key_generation,
        );

        let decrypt_with = |key: &[u8; 32], nonce_bytes: [u8; 24]| {
            let xnonce = XNonce::from(nonce_bytes);
            XChaCha20Poly1305::new(key.into())
                .decrypt(
                    &xnonce,
                    Payload {
                        msg: &combined,
                        aad: &aad,
                    },
                )
                .map_err(|_| GroupChannelCryptoError::IntegrityCheckFailed)
        };

        // Compatibility policy: encrypt with the fully derived v2 nonce layout and HKDF-v2 key,
        // but continue accepting legacy raw-prefix nonce layout and legacy SHA-256-v1 keys.
        let nonce_candidates = [
            group_nonce_bytes(
                sealed.sender_did.as_str(),
                sealed.key_generation,
                sealed.nonce,
            ),
            legacy_raw_prefix_group_nonce_bytes(
                sealed.sender_did.as_str(),
                sealed.key_generation,
                sealed.nonce,
            ),
        ];
        let key_candidates = [&aead_key_v2, &aead_key_v1];

        let mut plaintext = None;
        for key in key_candidates {
            for nonce_bytes in nonce_candidates {
                if let Ok(value) = decrypt_with(key, nonce_bytes) {
                    plaintext = Some(value);
                    break;
                }
            }
            if plaintext.is_some() {
                break;
            }
        }
        let plaintext = plaintext.ok_or(GroupChannelCryptoError::IntegrityCheckFailed)?;

        String::from_utf8(plaintext).map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)
    }

    fn cached_master_seed(&self) -> Result<[u8; 32], GroupChannelCryptoError> {
        if let Some(seed) = self.cached_master_seed.borrow().as_ref().copied() {
            return Ok(seed);
        }

        let seed = load_key_agreement_master_seed()?;
        self.cached_master_seed.borrow_mut().replace(seed);
        Ok(seed)
    }
}

impl Drop for GroupChannelCryptoEngine {
    fn drop(&mut self) {
        self.channel_id.zeroize();
        zeroize_sender_key_history(&mut self.sender_key_history);
        zeroize_u64_keyed_sender_history(&mut self.active_generation_by_sender);
        self.used_nonces.clear();
        if let Some(seed) = self.cached_master_seed.get_mut().as_mut() {
            seed.zeroize();
        }
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
    /// HKDF key derivation failed.
    KeyDerivationFailed,
    /// Integrity tag verification failed.
    IntegrityCheckFailed,
    /// Ciphertext encoding could not be decoded as valid hex.
    InvalidCiphertextEncoding,
}

impl fmt::Display for GroupChannelCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsecureCryptoDisabled => {
                write!(
                    f,
                    "legacy deterministic group-message crypto has been removed"
                )
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
                write!(
                    f,
                    "group message channel mismatch, expected {expected}, got {actual}"
                )
            }
            Self::SignatureMismatch => write!(f, "group message signature verification failed"),
            Self::EncryptionFailed => write!(f, "group message encryption failed"),
            Self::KeyDerivationFailed => write!(f, "group message key derivation failed"),
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
    let mut seed_hex = env::var(KEY_AGREEMENT_MASTER_SEED_ENV)
        .map_err(|_| GroupChannelCryptoError::MissingKeyAgreementMasterSeed)?;
    let seed = parse_fixed_hex_32(seed_hex.trim());
    seed_hex.zeroize();
    seed
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

fn derive_group_aead_key(
    shared_secret: &[u8; 32],
    channel_id: &str,
    generation: u64,
) -> Result<[u8; 32], GroupChannelCryptoError> {
    let mut info = Vec::with_capacity(
        GROUP_MESSAGE_AEAD_KDF_INFO_PREFIX_V2.len() + channel_id.len() + std::mem::size_of::<u64>(),
    );
    info.extend_from_slice(GROUP_MESSAGE_AEAD_KDF_INFO_PREFIX_V2);
    info.extend_from_slice(channel_id.as_bytes());
    info.extend_from_slice(&generation.to_le_bytes());
    match kamn_crypto::hkdf_sha256::derive_key_32(
        GROUP_MESSAGE_AEAD_KDF_SALT_V2,
        shared_secret,
        &info,
    ) {
        Ok(key) => Ok(key),
        Err(_) => Err(GroupChannelCryptoError::KeyDerivationFailed),
    }
}

fn derive_group_aead_key_legacy(
    shared_secret: &[u8; 32],
    channel_id: &str,
    generation: u64,
) -> [u8; 32] {
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
    let mut hasher = Sha256::new();
    hasher.update(GROUP_MESSAGE_NONCE_INFO_V2);
    hasher.update(sender_did.as_bytes());
    hasher.update(generation.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 24];
    out.copy_from_slice(&digest[..24]);
    out
}

fn legacy_raw_prefix_group_nonce_bytes(sender_did: &str, generation: u64, nonce: u64) -> [u8; 24] {
    let mut out = [0u8; 24];
    out[..8].copy_from_slice(&nonce.to_le_bytes());

    let mut hasher = Sha256::new();
    hasher.update(GROUP_MESSAGE_NONCE_INFO_V1);
    hasher.update(sender_did.as_bytes());
    hasher.update(generation.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    out[8..].copy_from_slice(&digest[..16]);
    out
}

fn zeroize_sender_key_history(
    sender_key_history: &mut BTreeMap<String, BTreeMap<u64, SenderKeyDistributionRecord>>,
) {
    for (mut sender_did, generations) in std::mem::take(sender_key_history) {
        sender_did.zeroize();
        for (_, mut record) in generations {
            zeroize_sender_key_distribution_record(&mut record);
        }
    }
}

fn zeroize_sender_key_distribution_record(record: &mut SenderKeyDistributionRecord) {
    record.channel_id.zeroize();
    record.sender_did.zeroize();
    record.sender_key_ref.zeroize();
    let allowlist = std::mem::take(&mut record.recipient_allowlist);
    for mut recipient in allowlist {
        recipient.zeroize();
    }
}

fn zeroize_u64_keyed_sender_history(sender_history: &mut BTreeMap<String, u64>) {
    for (mut sender_did, _) in std::mem::take(sender_history) {
        sender_did.zeroize();
    }
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
    use super::{
        GROUP_MESSAGE_CIPHER_ALGORITHM, GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM,
        GroupChannelCryptoEngine, GroupChannelCryptoError, GroupMessageCiphertext,
    };
    use chacha20poly1305::aead::{Aead, KeyInit, Payload};
    use chacha20poly1305::{XChaCha20Poly1305, XNonce};

    const SOURCE: &str = include_str!("group_channel_crypto.rs");
    const TEST_KEY_SEED_HEX: &str =
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn with_key_agreement_seed<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
        let _guard = crate::crypto_test_env_lock::key_agreement_seed_env_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());

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

    fn legacy_v1_ciphertext(
        channel_id: &str,
        sender_did: &str,
        sender_key_ref: &str,
        generation: u64,
        nonce: u64,
        plaintext: &str,
    ) -> GroupMessageCiphertext {
        let master_seed =
            super::load_key_agreement_master_seed().expect("master seed should be available");
        let shared_secret =
            super::derive_group_shared_secret(channel_id, sender_key_ref, generation, &master_seed);
        let legacy_key =
            super::derive_group_aead_key_legacy(&shared_secret, channel_id, generation);

        let cipher = XChaCha20Poly1305::new((&legacy_key).into());
        let nonce_bytes = super::group_nonce_bytes(sender_did, generation, nonce);
        let xnonce = XNonce::from(nonce_bytes);
        let aad: [u8; 0] = [];
        let mut sealed = cipher
            .encrypt(
                &xnonce,
                Payload {
                    msg: plaintext.as_bytes(),
                    aad: &aad,
                },
            )
            .expect("legacy encryption should succeed");
        let auth_tag = sealed.split_off(sealed.len() - 16);
        let ciphertext_hex = super::hex_encode(&sealed);
        let auth_tag_hex = super::hex_encode(&auth_tag);

        GroupMessageCiphertext {
            key_derivation_algorithm: GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM.to_owned(),
            cipher_algorithm: GROUP_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            channel_id: channel_id.to_owned(),
            sender_did: sender_did.to_owned(),
            key_generation: generation,
            nonce,
            ciphertext: ciphertext_hex.clone(),
            auth_tag: auth_tag_hex.clone(),
            signature: super::compute_signature(
                &shared_secret,
                channel_id,
                sender_did,
                generation,
                nonce,
                ciphertext_hex.as_str(),
                auth_tag_hex.as_str(),
            ),
        }
    }

    fn legacy_raw_prefix_group_nonce_bytes(
        sender_did: &str,
        generation: u64,
        nonce: u64,
    ) -> [u8; 24] {
        super::legacy_raw_prefix_group_nonce_bytes(sender_did, generation, nonce)
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
    fn encrypt_decrypt_roundtrip_requires_authorized_recipient() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            let distribution = engine
                .distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("distribution should succeed");
            engine
                .rotate_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-2",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("rotation should succeed");

            assert_eq!(
                engine
                    .active_sender_key_generation("kamn:did:agent:alice")
                    .expect("active generation should exist"),
                2
            );
            assert_eq!(
                engine
                    .sender_key_record("kamn:did:agent:alice", distribution.key_generation)
                    .expect("first generation record should exist")
                    .sender_key_ref,
                "kamn:did:agent:alice#sender-key-1"
            );

            let sealed = engine
                .encrypt("kamn:did:agent:alice", "group payload", 33)
                .expect("encrypt should succeed");

            let plaintext = engine
                .decrypt("kamn:did:agent:bob", &sealed)
                .expect("authorized recipient should decrypt");
            assert_eq!(plaintext, "group payload");
            assert_eq!(sealed.key_generation, 2);

            let debug_output = format!("{engine:?}");
            assert!(
                debug_output.contains("used_nonce_count: 1"),
                "debug output should expose only redacted summary counts: {debug_output}"
            );
            assert!(
                !debug_output.contains("sender-key-2"),
                "debug output must not expose sender key refs: {debug_output}"
            );

            let legacy = legacy_v1_ciphertext(
                "channel:group:1",
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-2",
                2,
                34,
                "legacy payload",
            );
            assert_eq!(
                engine
                    .decrypt("kamn:did:agent:bob", &legacy)
                    .expect("legacy ciphertext should remain decryptable"),
                "legacy payload"
            );

            drop(engine);
        });
    }

    #[test]
    fn decrypt_rejects_unauthorized_recipient() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            let distribution = engine
                .distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("distribution should succeed");
            let sealed = engine
                .encrypt("kamn:did:agent:alice", "group payload", 35)
                .expect("encrypt should succeed");

            assert_eq!(
                engine.decrypt("kamn:did:agent:charlie", &sealed),
                Err(GroupChannelCryptoError::RecipientNotAuthorized {
                    recipient_did: "kamn:did:agent:charlie".to_owned(),
                    sender_did: "kamn:did:agent:alice".to_owned(),
                    key_generation: distribution.key_generation,
                })
            );
        });
    }

    #[test]
    fn decrypt_rejects_tampered_signature() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            engine
                .distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("distribution should succeed");
            let mut sealed = engine
                .encrypt("kamn:did:agent:alice", "group payload", 37)
                .expect("encrypt should succeed");
            assert!(
                !sealed.signature.is_empty(),
                "signature fixture must be non-empty"
            );
            let replacement = if sealed.signature.starts_with('0') {
                '1'
            } else {
                '0'
            };
            sealed
                .signature
                .replace_range(0..1, &replacement.to_string());

            let decrypted = engine.decrypt("kamn:did:agent:bob", &sealed);
            assert!(
                matches!(
                    decrypted,
                    Err(GroupChannelCryptoError::SignatureMismatch)
                        | Err(GroupChannelCryptoError::MissingKeyAgreementMasterSeed)
                ),
                "tampered signature must fail closed even when the key-agreement seed is missing; got {decrypted:?}"
            );
        });
    }

    #[test]
    fn regression_requires_constant_time_group_signature_compare() {
        assert!(
            SOURCE.contains("crate::constant_time_eq::constant_time_eq_str("),
            "group decrypt should use the scoped constant-time helper for signature comparison"
        );
        assert!(
            !SOURCE.contains(
                ["if expected_signature !=", " sealed.signature {"]
                    .concat()
                    .as_str()
            ),
            "group decrypt must not use direct signature inequality"
        );
    }

    #[test]
    fn decrypt_rejects_zero_nonce_fail_closed() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine =
                GroupChannelCryptoEngine::new("channel:group:1").expect("engine should initialize");
            engine
                .distribute_sender_key(
                    "kamn:did:agent:alice",
                    "kamn:did:agent:alice#sender-key-1",
                    vec!["kamn:did:agent:bob".to_owned()],
                )
                .expect("distribution should succeed");
            let mut sealed = engine
                .encrypt("kamn:did:agent:alice", "group payload", 39)
                .expect("encrypt should succeed");
            sealed.nonce = 0;

            assert_eq!(
                engine.decrypt("kamn:did:agent:bob", &sealed),
                Err(GroupChannelCryptoError::InvalidNonce(0))
            );
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
    fn group_message_hkdf_derivation_is_deterministic_and_distinct_from_legacy_v1() {
        let shared_secret = [0x3cu8; 32];
        let hkdf_key_a = super::derive_group_aead_key(&shared_secret, "channel:test", 9)
            .expect("hkdf key should derive");
        let hkdf_key_b = super::derive_group_aead_key(&shared_secret, "channel:test", 9)
            .expect("hkdf key should derive");
        let legacy_key = super::derive_group_aead_key_legacy(&shared_secret, "channel:test", 9);

        assert_eq!(hkdf_key_a, hkdf_key_b);
        assert_ne!(hkdf_key_a, legacy_key);
    }

    #[test]
    fn group_message_derivation_backend_markers_and_manual_helper_removal_contract() {
        assert_eq!(
            super::GROUP_MESSAGE_HKDF_BACKEND_MARKER,
            "rustcrypto.hkdf.sha256.v1"
        );
        assert_eq!(
            super::GROUP_MESSAGE_HMAC_BACKEND_MARKER,
            "rustcrypto.hmac.sha256.v1"
        );
        assert!(
            !SOURCE.contains("\nfn hkdf_sha256_derive_32("),
            "manual hkdf helper must be removed"
        );
        assert!(
            !SOURCE.contains("\nfn hmac_sha256("),
            "manual hmac helper must be removed"
        );
    }

    #[test]
    fn spec_c09_group_channel_engine_source_contract_enforces_non_clone_redacted_debug_and_drop_zeroize()
     {
        let production_source = SOURCE.split("\n#[cfg(test)]").next().unwrap_or(SOURCE);
        let derive_line = production_source
            .split("pub struct GroupChannelCryptoEngine")
            .next()
            .and_then(|prefix| prefix.lines().last())
            .unwrap_or_default();
        let derive_window = derive_line.replace(' ', "");
        assert!(
            !derive_window.contains("Clone"),
            "group-channel engine must not derive Clone"
        );
        assert!(
            production_source.contains("impl fmt::Debug for GroupChannelCryptoEngine"),
            "group-channel engine must define a redacted Debug impl"
        );
        assert!(
            production_source.contains("used_nonce_count"),
            "group-channel engine debug output must expose only a nonce-count summary"
        );
        assert!(
            production_source.contains("impl Drop for GroupChannelCryptoEngine"),
            "group-channel engine must define Drop"
        );
        assert!(
            production_source.contains("self.channel_id.zeroize();"),
            "group-channel engine Drop must zeroize channel_id"
        );
        assert!(
            production_source.contains("zeroize_sender_key_history(&mut self.sender_key_history);"),
            "group-channel engine Drop must clear sender-key history"
        );
        assert!(
            production_source.contains("seed_hex.zeroize();"),
            "group-channel master seed loader must zeroize env-loaded hex buffer"
        );
    }

    #[test]
    fn group_nonce_bytes_do_not_expose_raw_counter_prefix() {
        let nonce = 0x0102_0304_0506_0708_u64;
        let nonce_bytes = super::group_nonce_bytes("kamn:did:agent:alice", 7, nonce);

        assert_ne!(
            &nonce_bytes[..8],
            &nonce.to_le_bytes(),
            "derived group nonce bytes must not expose nonce.to_le_bytes() as the prefix"
        );
    }

    #[test]
    fn encrypt_output_does_not_authenticate_under_legacy_raw_prefix_nonce_layout() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let channel_id = "channel:group:nonce-layout";
            let sender_did = "kamn:did:agent:alice";
            let sender_key_ref = "kamn:did:agent:alice#sender-key-1";
            let recipient_did = "kamn:did:agent:bob";
            let nonce = 57;

            let mut engine =
                GroupChannelCryptoEngine::new(channel_id).expect("group engine should initialize");
            let distribution = engine
                .distribute_sender_key(sender_did, sender_key_ref, vec![recipient_did.to_owned()])
                .expect("distribution should succeed");
            let sealed = engine
                .encrypt(sender_did, "group-nonce-layout", nonce)
                .expect("encrypt should succeed");

            let master_seed =
                super::load_key_agreement_master_seed().expect("master seed should be available");
            let shared_secret = super::derive_group_shared_secret(
                channel_id,
                sender_key_ref,
                distribution.key_generation,
                &master_seed,
            );
            let aead_key = super::derive_group_aead_key(
                &shared_secret,
                channel_id,
                distribution.key_generation,
            )
            .expect("aead key should derive");
            let cipher = XChaCha20Poly1305::new((&aead_key).into());
            let legacy_nonce_bytes =
                legacy_raw_prefix_group_nonce_bytes(sender_did, distribution.key_generation, nonce);
            let xnonce = XNonce::from(legacy_nonce_bytes);
            let mut combined = super::hex_decode(&sealed.ciphertext).expect("ciphertext hex");
            combined.extend_from_slice(&super::hex_decode(&sealed.auth_tag).expect("auth tag hex"));

            let decrypted = cipher.decrypt(
                &xnonce,
                Payload {
                    msg: &combined,
                    aad: &[],
                },
            );

            assert!(
                decrypted.is_err(),
                "current group encryptions must not authenticate under the legacy raw-prefix nonce layout"
            );
        });
    }

    #[test]
    fn decrypt_accepts_legacy_v1_sha256_kdf_ciphertext_for_compatibility() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let channel_id = "channel:group:legacy";
            let sender_did = "kamn:did:agent:alice";
            let sender_key_ref = "kamn:did:agent:alice#sender-key-1";
            let recipient_did = "kamn:did:agent:bob";

            let mut engine =
                GroupChannelCryptoEngine::new(channel_id).expect("group engine should initialize");
            let distribution = engine
                .distribute_sender_key(sender_did, sender_key_ref, vec![recipient_did.to_owned()])
                .expect("distribution should succeed");
            let sealed = legacy_v1_ciphertext(
                channel_id,
                sender_did,
                sender_key_ref,
                distribution.key_generation,
                57,
                "legacy-group-v1",
            );

            let plaintext = engine
                .decrypt(recipient_did, &sealed)
                .expect("legacy-v1 group decrypt must succeed");
            assert_eq!(plaintext, "legacy-group-v1");
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
