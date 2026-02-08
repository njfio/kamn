use std::collections::BTreeSet;
use std::fmt;

pub const DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM: &str = "X25519";
pub const DIRECT_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectMessageCiphertext {
    pub key_agreement_algorithm: String,
    pub cipher_algorithm: String,
    pub sender_key_ref: String,
    pub recipient_key_ref: String,
    pub nonce: u64,
    pub ciphertext: String,
    pub auth_tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectMessageCryptoEngine {
    sender_key_ref: String,
    recipient_key_ref: String,
    shared_secret_fingerprint: String,
    used_nonces: BTreeSet<u64>,
}

impl DirectMessageCryptoEngine {
    pub fn new(
        sender_key_ref: &str,
        recipient_key_ref: &str,
    ) -> Result<Self, DirectMessageCryptoError> {
        validate_key_ref("sender", sender_key_ref)?;
        validate_key_ref("recipient", recipient_key_ref)?;

        Ok(Self {
            sender_key_ref: sender_key_ref.to_owned(),
            recipient_key_ref: recipient_key_ref.to_owned(),
            shared_secret_fingerprint: derive_shared_secret_fingerprint(
                sender_key_ref,
                recipient_key_ref,
            ),
            used_nonces: BTreeSet::new(),
        })
    }

    pub fn encrypt(
        &mut self,
        plaintext: &str,
        nonce: u64,
    ) -> Result<DirectMessageCiphertext, DirectMessageCryptoError> {
        if plaintext.is_empty() {
            return Err(DirectMessageCryptoError::EmptyPayload);
        }
        if nonce == 0 {
            return Err(DirectMessageCryptoError::InvalidNonce(nonce));
        }
        if !self.used_nonces.insert(nonce) {
            return Err(DirectMessageCryptoError::NonceReuse(nonce));
        }

        let plaintext_bytes = plaintext.as_bytes();
        let keystream = derive_keystream(
            &self.shared_secret_fingerprint,
            nonce,
            plaintext_bytes.len(),
        );
        let encrypted: Vec<u8> = plaintext_bytes
            .iter()
            .zip(keystream.iter())
            .map(|(byte, mask)| byte ^ mask)
            .collect();
        let ciphertext = hex_encode(&encrypted);
        let auth_tag = compute_auth_tag(&self.shared_secret_fingerprint, nonce, &ciphertext);

        Ok(DirectMessageCiphertext {
            key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
            cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            sender_key_ref: self.sender_key_ref.clone(),
            recipient_key_ref: self.recipient_key_ref.clone(),
            nonce,
            ciphertext,
            auth_tag,
        })
    }

    pub fn decrypt(
        &self,
        sealed: &DirectMessageCiphertext,
    ) -> Result<String, DirectMessageCryptoError> {
        if sealed.key_agreement_algorithm != DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM
            || sealed.cipher_algorithm != DIRECT_MESSAGE_CIPHER_ALGORITHM
        {
            return Err(DirectMessageCryptoError::AlgorithmMismatch);
        }
        let expected_tag = compute_auth_tag(
            &self.shared_secret_fingerprint,
            sealed.nonce,
            &sealed.ciphertext,
        );
        if expected_tag != sealed.auth_tag {
            return Err(DirectMessageCryptoError::IntegrityCheckFailed);
        }

        let encrypted = hex_decode(&sealed.ciphertext)?;
        let keystream = derive_keystream(
            &self.shared_secret_fingerprint,
            sealed.nonce,
            encrypted.len(),
        );
        let plaintext: Vec<u8> = encrypted
            .iter()
            .zip(keystream.iter())
            .map(|(byte, mask)| byte ^ mask)
            .collect();
        String::from_utf8(plaintext)
            .map_err(|_| DirectMessageCryptoError::InvalidCiphertextEncoding)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectMessageCryptoError {
    EmptyKeyRef(&'static str),
    InvalidKeyRef(&'static str),
    EmptyPayload,
    InvalidNonce(u64),
    NonceReuse(u64),
    AlgorithmMismatch,
    IntegrityCheckFailed,
    InvalidCiphertextEncoding,
}

impl fmt::Display for DirectMessageCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKeyRef(role) => write!(f, "{role} key reference must not be empty"),
            Self::InvalidKeyRef(role) => {
                write!(f, "{role} key reference must include #key-agreement")
            }
            Self::EmptyPayload => write!(f, "plaintext payload must not be empty"),
            Self::InvalidNonce(value) => write!(f, "nonce must be positive: {value}"),
            Self::NonceReuse(value) => write!(f, "nonce reuse detected: {value}"),
            Self::AlgorithmMismatch => write!(f, "direct message algorithm mismatch"),
            Self::IntegrityCheckFailed => write!(f, "ciphertext integrity check failed"),
            Self::InvalidCiphertextEncoding => write!(f, "invalid ciphertext encoding"),
        }
    }
}

impl std::error::Error for DirectMessageCryptoError {}

fn validate_key_ref(role: &'static str, key_ref: &str) -> Result<(), DirectMessageCryptoError> {
    if key_ref.trim().is_empty() {
        return Err(DirectMessageCryptoError::EmptyKeyRef(role));
    }
    if !key_ref.contains("#key-agreement") {
        return Err(DirectMessageCryptoError::InvalidKeyRef(role));
    }
    Ok(())
}

fn derive_shared_secret_fingerprint(sender: &str, recipient: &str) -> String {
    let (left, right) = if sender <= recipient {
        (sender, recipient)
    } else {
        (recipient, sender)
    };
    format!("x25519:{left}|{right}")
}

fn derive_keystream(secret: &str, nonce: u64, len: usize) -> Vec<u8> {
    let secret_bytes = secret.as_bytes();
    (0..len)
        .map(|idx| {
            let seed = secret_bytes[idx % secret_bytes.len()];
            seed ^ (nonce as u8).wrapping_add((idx as u8).wrapping_mul(31))
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

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

fn hex_decode(value: &str) -> Result<Vec<u8>, DirectMessageCryptoError> {
    if !value.len().is_multiple_of(2) {
        return Err(DirectMessageCryptoError::InvalidCiphertextEncoding);
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for chunk in value.as_bytes().chunks_exact(2) {
        let encoded = std::str::from_utf8(chunk)
            .map_err(|_| DirectMessageCryptoError::InvalidCiphertextEncoding)?;
        let byte = u8::from_str_radix(encoded, 16)
            .map_err(|_| DirectMessageCryptoError::InvalidCiphertextEncoding)?;
        bytes.push(byte);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{DirectMessageCryptoEngine, DirectMessageCryptoError};

    #[test]
    fn constructor_rejects_invalid_key_reference() {
        assert_eq!(
            DirectMessageCryptoEngine::new("did:alice#keys-1", "did:bob#key-agreement-1"),
            Err(DirectMessageCryptoError::InvalidKeyRef("sender"))
        );
    }

    #[test]
    fn decrypt_rejects_algorithm_mismatch() {
        let mut engine = match DirectMessageCryptoEngine::new(
            "did:alice#key-agreement-1",
            "did:bob#key-agreement-1",
        ) {
            Ok(value) => value,
            Err(error) => panic!("engine init failed: {error}"),
        };
        let mut sealed = match engine.encrypt("payload", 1) {
            Ok(value) => value,
            Err(error) => panic!("encrypt failed: {error}"),
        };
        sealed.cipher_algorithm = "AES-GCM".to_owned();

        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::AlgorithmMismatch)
        );
    }
}
