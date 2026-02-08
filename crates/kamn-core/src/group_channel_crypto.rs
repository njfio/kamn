use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM: &str = "SenderKey-v1";
pub const GROUP_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderKeyDistributionRecord {
    pub channel_id: String,
    pub sender_did: String,
    pub sender_key_ref: String,
    pub key_generation: u64,
    pub recipient_allowlist: BTreeSet<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMessageCiphertext {
    pub key_derivation_algorithm: String,
    pub cipher_algorithm: String,
    pub channel_id: String,
    pub sender_did: String,
    pub key_generation: u64,
    pub nonce: u64,
    pub ciphertext: String,
    pub auth_tag: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupChannelCryptoEngine {
    channel_id: String,
    sender_key_history: BTreeMap<String, BTreeMap<u64, SenderKeyDistributionRecord>>,
    active_generation_by_sender: BTreeMap<String, u64>,
    used_nonces: BTreeSet<(String, u64, u64)>,
}

impl GroupChannelCryptoEngine {
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

    pub fn distribute_sender_key(
        &mut self,
        sender_did: &str,
        sender_key_ref: &str,
        recipients: Vec<String>,
    ) -> Result<SenderKeyDistributionRecord, GroupChannelCryptoError> {
        validate_did(sender_did)?;
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

    pub fn rotate_sender_key(
        &mut self,
        sender_did: &str,
        sender_key_ref: &str,
        recipients: Vec<String>,
    ) -> Result<SenderKeyDistributionRecord, GroupChannelCryptoError> {
        self.distribute_sender_key(sender_did, sender_key_ref, recipients)
    }

    pub fn active_sender_key_generation(
        &self,
        sender_did: &str,
    ) -> Result<u64, GroupChannelCryptoError> {
        self.active_generation_by_sender
            .get(sender_did)
            .copied()
            .ok_or_else(|| GroupChannelCryptoError::SenderKeyNotFound(sender_did.to_owned()))
    }

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

    pub fn encrypt(
        &mut self,
        sender_did: &str,
        plaintext: &str,
        nonce: u64,
    ) -> Result<GroupMessageCiphertext, GroupChannelCryptoError> {
        validate_did(sender_did)?;
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

        let shared_secret = derive_sender_secret(
            &self.channel_id,
            &record.sender_key_ref,
            record.key_generation,
        );
        let plaintext_bytes = plaintext.as_bytes();
        let keystream = derive_keystream(&shared_secret, nonce, plaintext_bytes.len());
        let encrypted: Vec<u8> = plaintext_bytes
            .iter()
            .zip(keystream.iter())
            .map(|(byte, mask)| byte ^ mask)
            .collect();
        let ciphertext = hex_encode(&encrypted);

        let auth_tag = compute_auth_tag(&shared_secret, nonce, &ciphertext);
        let signature = compute_signature(
            &record.sender_key_ref,
            &self.channel_id,
            record.key_generation,
            nonce,
            &ciphertext,
        );

        Ok(GroupMessageCiphertext {
            key_derivation_algorithm: GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM.to_owned(),
            cipher_algorithm: GROUP_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            channel_id: self.channel_id.clone(),
            sender_did: sender_did.to_owned(),
            key_generation: record.key_generation,
            nonce,
            ciphertext,
            auth_tag,
            signature,
        })
    }

    pub fn decrypt(
        &self,
        recipient_did: &str,
        sealed: &GroupMessageCiphertext,
    ) -> Result<String, GroupChannelCryptoError> {
        validate_did(recipient_did)?;
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

        let expected_signature = compute_signature(
            &record.sender_key_ref,
            &sealed.channel_id,
            sealed.key_generation,
            sealed.nonce,
            &sealed.ciphertext,
        );
        if expected_signature != sealed.signature {
            return Err(GroupChannelCryptoError::SignatureMismatch);
        }

        let shared_secret = derive_sender_secret(
            &self.channel_id,
            &record.sender_key_ref,
            sealed.key_generation,
        );
        let expected_tag = compute_auth_tag(&shared_secret, sealed.nonce, &sealed.ciphertext);
        if expected_tag != sealed.auth_tag {
            return Err(GroupChannelCryptoError::IntegrityCheckFailed);
        }

        let encrypted = hex_decode(&sealed.ciphertext)?;
        let keystream = derive_keystream(&shared_secret, sealed.nonce, encrypted.len());
        let plaintext: Vec<u8> = encrypted
            .iter()
            .zip(keystream.iter())
            .map(|(byte, mask)| byte ^ mask)
            .collect();

        String::from_utf8(plaintext).map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupChannelCryptoError {
    EmptyChannelId,
    EmptyRecipients,
    EmptyPayload,
    InvalidNonce(u64),
    NonceReuse(u64),
    InvalidDid(String),
    InvalidSenderKeyRef,
    SenderKeyNotFound(String),
    UnknownSenderKeyGeneration {
        sender_did: String,
        key_generation: u64,
    },
    RecipientNotAuthorized {
        recipient_did: String,
        sender_did: String,
        key_generation: u64,
    },
    AlgorithmMismatch,
    ChannelMismatch {
        expected: String,
        actual: String,
    },
    SignatureMismatch,
    IntegrityCheckFailed,
    InvalidCiphertextEncoding,
}

impl fmt::Display for GroupChannelCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChannelId => write!(f, "channel_id must not be empty"),
            Self::EmptyRecipients => write!(f, "recipient allowlist must not be empty"),
            Self::EmptyPayload => write!(f, "plaintext payload must not be empty"),
            Self::InvalidNonce(value) => write!(f, "nonce must be positive: {value}"),
            Self::NonceReuse(value) => write!(f, "nonce reuse detected: {value}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
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
            Self::IntegrityCheckFailed => write!(f, "group message integrity check failed"),
            Self::InvalidCiphertextEncoding => write!(f, "invalid ciphertext encoding"),
        }
    }
}

impl std::error::Error for GroupChannelCryptoError {}

fn validate_did(value: &str) -> Result<(), GroupChannelCryptoError> {
    AgentDid::parse(value)
        .map_err(|error| GroupChannelCryptoError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn validate_sender_key_ref(value: &str) -> Result<(), GroupChannelCryptoError> {
    if !value.contains("#sender-key-") {
        return Err(GroupChannelCryptoError::InvalidSenderKeyRef);
    }
    Ok(())
}

fn validate_recipients(
    recipients: Vec<String>,
) -> Result<BTreeSet<String>, GroupChannelCryptoError> {
    if recipients.is_empty() {
        return Err(GroupChannelCryptoError::EmptyRecipients);
    }

    let mut allowlist = BTreeSet::new();
    for recipient in recipients {
        validate_did(&recipient)?;
        allowlist.insert(recipient);
    }
    Ok(allowlist)
}

fn derive_sender_secret(channel_id: &str, sender_key_ref: &str, generation: u64) -> String {
    format!("senderkey:{channel_id}|{sender_key_ref}|{generation}")
}

fn derive_keystream(secret: &str, nonce: u64, len: usize) -> Vec<u8> {
    let secret_bytes = secret.as_bytes();
    (0..len)
        .map(|idx| {
            let seed = secret_bytes[idx % secret_bytes.len()];
            seed ^ (nonce as u8).wrapping_add((idx as u8).wrapping_mul(17))
        })
        .collect()
}

fn compute_auth_tag(secret: &str, nonce: u64, ciphertext: &str) -> String {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in secret.bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    for byte in nonce.to_le_bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    for byte in ciphertext.bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    format!("{acc:016x}")
}

fn compute_signature(
    sender_key_ref: &str,
    channel_id: &str,
    generation: u64,
    nonce: u64,
    ciphertext: &str,
) -> String {
    let mut acc: u64 = 0x84222325cbf29ce4;
    for byte in sender_key_ref.bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    for byte in channel_id.bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    for byte in generation.to_le_bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    for byte in nonce.to_le_bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    for byte in ciphertext.bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    format!("sig:{acc:016x}")
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

    #[test]
    fn constructor_rejects_empty_channel_id() {
        assert_eq!(
            GroupChannelCryptoEngine::new(""),
            Err(GroupChannelCryptoError::EmptyChannelId)
        );
    }

    #[test]
    fn distribution_rejects_empty_recipients() {
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
    }

    #[test]
    fn rotate_marks_previous_generation_inactive() {
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
    }
}
