use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256, Sha512};
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use x25519_dalek::{PublicKey, StaticSecret};

/// Canonical direct-message key-agreement algorithm identifier.
pub const DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM: &str = "X25519";
/// Canonical direct-message cipher algorithm identifier.
pub const DIRECT_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const DIRECT_MESSAGE_AEAD_KDF_SALT_V2: &[u8] = b"kamn:direct-message:aead-key:hkdf-salt:v2";
const DIRECT_MESSAGE_AEAD_KDF_INFO_V2: &[u8] = b"kamn:direct-message:aead-key:hkdf-info:v2";
/// Marker asserting HKDF derivation is backed by RustCrypto hkdf crate.
pub const DIRECT_MESSAGE_HKDF_BACKEND_MARKER: &str = crate::hkdf_sha256::HKDF_SHA256_BACKEND_MARKER;
/// Marker asserting HMAC backend semantics are provided by RustCrypto primitives.
pub const DIRECT_MESSAGE_HMAC_BACKEND_MARKER: &str = crate::hkdf_sha256::HMAC_SHA256_BACKEND_MARKER;

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

/// Direct-message crypto engine with nonce reuse protection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectMessageCryptoEngine {
    sender_key_ref: String,
    recipient_key_ref: String,
    aead_key: [u8; 32],
    legacy_aead_key: [u8; 32],
    used_nonces: BTreeSet<u64>,
}

impl DirectMessageCryptoEngine {
    /// Creates a new engine for sender/recipient key references.
    pub fn new(
        sender_key_ref: &str,
        recipient_key_ref: &str,
    ) -> Result<Self, DirectMessageCryptoError> {
        validate_key_ref("sender", sender_key_ref)?;
        validate_key_ref("recipient", recipient_key_ref)?;

        let master_seed = load_key_agreement_master_seed()?;
        let shared_secret =
            derive_x25519_shared_secret(sender_key_ref, recipient_key_ref, &master_seed);

        Ok(Self {
            sender_key_ref: sender_key_ref.to_owned(),
            recipient_key_ref: recipient_key_ref.to_owned(),
            aead_key: derive_direct_message_aead_key(&shared_secret)?,
            legacy_aead_key: derive_direct_message_aead_key_legacy(&shared_secret),
            used_nonces: BTreeSet::new(),
        })
    }

    /// Encrypts plaintext with the provided nonce and returns ciphertext metadata.
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

        let cipher = XChaCha20Poly1305::new((&self.aead_key).into());
        let nonce_bytes = direct_message_nonce_bytes(nonce);
        let xnonce = XNonce::from(nonce_bytes);
        let aad = canonical_direct_message_aad(
            self.sender_key_ref.as_str(),
            self.recipient_key_ref.as_str(),
            nonce,
        );
        let payload = Payload {
            msg: plaintext.as_bytes(),
            aad: aad.as_bytes(),
        };

        let mut sealed = cipher
            .encrypt(&xnonce, payload)
            .map_err(|_| DirectMessageCryptoError::EncryptionFailed)?;
        let auth_tag = sealed.split_off(sealed.len() - 16);

        Ok(DirectMessageCiphertext {
            key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
            cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            sender_key_ref: self.sender_key_ref.clone(),
            recipient_key_ref: self.recipient_key_ref.clone(),
            nonce,
            ciphertext: hex_encode(&sealed),
            auth_tag: hex_encode(&auth_tag),
        })
    }

    /// Decrypts ciphertext after algorithm and integrity validation.
    pub fn decrypt(
        &self,
        sealed: &DirectMessageCiphertext,
    ) -> Result<String, DirectMessageCryptoError> {
        if sealed.key_agreement_algorithm != DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM
            || sealed.cipher_algorithm != DIRECT_MESSAGE_CIPHER_ALGORITHM
        {
            return Err(DirectMessageCryptoError::AlgorithmMismatch);
        }
        validate_key_ref_match(
            "sender",
            self.sender_key_ref.as_str(),
            sealed.sender_key_ref.as_str(),
        )?;
        validate_key_ref_match(
            "recipient",
            self.recipient_key_ref.as_str(),
            sealed.recipient_key_ref.as_str(),
        )?;
        if sealed.nonce == 0 {
            return Err(DirectMessageCryptoError::InvalidNonce(sealed.nonce));
        }

        let ciphertext = hex_decode(&sealed.ciphertext)?;
        let auth_tag = hex_decode(&sealed.auth_tag)
            .map_err(|_| DirectMessageCryptoError::IntegrityCheckFailed)?;

        let mut combined = ciphertext;
        combined.extend_from_slice(&auth_tag);

        let nonce_bytes = direct_message_nonce_bytes(sealed.nonce);
        let xnonce = XNonce::from(nonce_bytes);
        let aad = canonical_direct_message_aad(
            sealed.sender_key_ref.as_str(),
            sealed.recipient_key_ref.as_str(),
            sealed.nonce,
        );

        let decrypt_with_key = |key: &[u8; 32]| {
            XChaCha20Poly1305::new(key.into())
                .decrypt(
                    &xnonce,
                    Payload {
                        msg: &combined,
                        aad: aad.as_bytes(),
                    },
                )
                .map_err(|_| DirectMessageCryptoError::IntegrityCheckFailed)
        };

        // Compatibility policy: encrypt with HKDF-v2 key, but continue accepting
        // legacy SHA-256-v1 derived ciphertext on decrypt.
        let plaintext = match decrypt_with_key(&self.aead_key) {
            Ok(value) => Ok(value),
            Err(DirectMessageCryptoError::IntegrityCheckFailed) => {
                decrypt_with_key(&self.legacy_aead_key)
            }
            Err(other) => Err(other),
        }?;
        String::from_utf8(plaintext)
            .map_err(|_| DirectMessageCryptoError::InvalidCiphertextEncoding)
    }
}

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

fn validate_key_ref(role: &'static str, key_ref: &str) -> Result<(), DirectMessageCryptoError> {
    if key_ref.trim().is_empty() {
        return Err(DirectMessageCryptoError::EmptyKeyRef(role));
    }
    if !key_ref.contains("#key-agreement") {
        return Err(DirectMessageCryptoError::InvalidKeyRef(role));
    }
    Ok(())
}

fn validate_key_ref_match(
    role: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), DirectMessageCryptoError> {
    if expected != actual {
        return Err(DirectMessageCryptoError::KeyRefMismatch(role));
    }
    Ok(())
}

fn load_key_agreement_master_seed() -> Result<[u8; 32], DirectMessageCryptoError> {
    let seed_hex = env::var(KEY_AGREEMENT_MASTER_SEED_ENV)
        .map_err(|_| DirectMessageCryptoError::MissingKeyAgreementMasterSeed)?;
    parse_fixed_hex_32(seed_hex.trim())
}

fn parse_fixed_hex_32(value: &str) -> Result<[u8; 32], DirectMessageCryptoError> {
    if value.len() != 64 {
        return Err(DirectMessageCryptoError::InvalidKeyAgreementMasterSeed);
    }
    let mut out = [0u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let encoded = std::str::from_utf8(chunk)
            .map_err(|_| DirectMessageCryptoError::InvalidKeyAgreementMasterSeed)?;
        let byte = u8::from_str_radix(encoded, 16)
            .map_err(|_| DirectMessageCryptoError::InvalidKeyAgreementMasterSeed)?;
        out[index] = byte;
    }
    Ok(out)
}

fn derive_x25519_shared_secret(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    master_seed: &[u8; 32],
) -> [u8; 32] {
    let sender_private = derive_x25519_private_key(master_seed, sender_key_ref);
    let recipient_public = derive_x25519_public_key(master_seed, recipient_key_ref);
    sender_private.diffie_hellman(&recipient_public).to_bytes()
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

fn derive_direct_message_aead_key(
    shared_secret: &[u8; 32],
) -> Result<[u8; 32], DirectMessageCryptoError> {
    crate::hkdf_sha256::derive_key_32(
        DIRECT_MESSAGE_AEAD_KDF_SALT_V2,
        shared_secret,
        DIRECT_MESSAGE_AEAD_KDF_INFO_V2,
    )
    .map_err(|_| DirectMessageCryptoError::KeyDerivationFailed)
}

fn derive_direct_message_aead_key_legacy(shared_secret: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"kamn:direct-message:aead-key:v1:");
    hasher.update(shared_secret);
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest[..32]);
    key
}

fn direct_message_nonce_bytes(nonce: u64) -> [u8; 24] {
    let mut out = [0u8; 24];
    out[..8].copy_from_slice(&nonce.to_le_bytes());

    let mut hasher = Sha256::new();
    hasher.update(b"kamn:direct-message:nonce:v1:");
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    out[8..].copy_from_slice(&digest[..16]);
    out
}

fn canonical_direct_message_aad(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    nonce: u64,
) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
        DIRECT_MESSAGE_CIPHER_ALGORITHM,
        sender_key_ref,
        recipient_key_ref,
        nonce
    )
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
    use super::{
        DirectMessageCiphertext, DirectMessageCryptoEngine, DirectMessageCryptoError,
        DIRECT_MESSAGE_CIPHER_ALGORITHM, DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
    };
    use chacha20poly1305::aead::{Aead, KeyInit, Payload};
    use chacha20poly1305::{XChaCha20Poly1305, XNonce};
    use std::sync::{Mutex, OnceLock};

    const TEST_KEY_SEED_HEX: &str =
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    const SOURCE: &str = include_str!("direct_message_crypto.rs");

    fn key_agreement_seed_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_key_agreement_seed<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
        let _guard = key_agreement_seed_env_lock()
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
        sender_key_ref: &str,
        recipient_key_ref: &str,
        plaintext: &str,
        nonce: u64,
    ) -> DirectMessageCiphertext {
        let master_seed =
            super::load_key_agreement_master_seed().expect("master seed should be available");
        let shared_secret =
            super::derive_x25519_shared_secret(sender_key_ref, recipient_key_ref, &master_seed);
        let legacy_key = super::derive_direct_message_aead_key_legacy(&shared_secret);

        let cipher = XChaCha20Poly1305::new((&legacy_key).into());
        let nonce_bytes = super::direct_message_nonce_bytes(nonce);
        let xnonce = XNonce::from(nonce_bytes);
        let aad = super::canonical_direct_message_aad(sender_key_ref, recipient_key_ref, nonce);
        let payload = Payload {
            msg: plaintext.as_bytes(),
            aad: aad.as_bytes(),
        };
        let mut sealed = cipher
            .encrypt(&xnonce, payload)
            .expect("legacy encryption should succeed");
        let auth_tag = sealed.split_off(sealed.len() - 16);

        DirectMessageCiphertext {
            key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
            cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            sender_key_ref: sender_key_ref.to_owned(),
            recipient_key_ref: recipient_key_ref.to_owned(),
            nonce,
            ciphertext: super::hex_encode(&sealed),
            auth_tag: super::hex_encode(&auth_tag),
        }
    }

    #[test]
    fn constructor_rejects_invalid_key_reference() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            assert_eq!(
                DirectMessageCryptoEngine::new("did:alice#keys-1", "did:bob#key-agreement-1"),
                Err(DirectMessageCryptoError::InvalidKeyRef("sender"))
            );
        });
    }

    #[test]
    fn decrypt_rejects_algorithm_mismatch() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
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
        });
    }

    #[test]
    fn encrypt_decrypt_roundtrip_succeeds_for_valid_payload() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine = match DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            ) {
                Ok(value) => value,
                Err(error) => panic!("engine init failed: {error}"),
            };
            let sealed = match engine.encrypt("hello-secure-world", 7) {
                Ok(value) => value,
                Err(error) => panic!("encrypt failed: {error}"),
            };

            let plaintext = match engine.decrypt(&sealed) {
                Ok(value) => value,
                Err(error) => panic!("decrypt failed: {error}"),
            };
            assert_eq!(plaintext, "hello-secure-world");
        });
    }

    #[test]
    fn encrypt_rejects_nonce_reuse_for_same_engine_instance() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine = match DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            ) {
                Ok(value) => value,
                Err(error) => panic!("engine init failed: {error}"),
            };
            if let Err(error) = engine.encrypt("payload", 11) {
                panic!("initial encrypt failed unexpectedly: {error}");
            }

            assert_eq!(
                engine.encrypt("payload-2", 11),
                Err(DirectMessageCryptoError::NonceReuse(11))
            );
        });
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext_with_integrity_error() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine = match DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            ) {
                Ok(value) => value,
                Err(error) => panic!("engine init failed: {error}"),
            };
            let mut sealed = match engine.encrypt("payload", 13) {
                Ok(value) => value,
                Err(error) => panic!("encrypt failed: {error}"),
            };
            sealed.ciphertext.replace_range(..1, "f");

            assert_eq!(
                engine.decrypt(&sealed),
                Err(DirectMessageCryptoError::IntegrityCheckFailed)
            );
        });
    }

    #[test]
    fn regression_constructor_accepts_without_insecure_fixture_opt_in() {
        // Regression: #5921
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let result = DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            );
            assert!(result.is_ok());
        });
    }

    #[test]
    fn constructor_requires_key_agreement_seed() {
        with_key_agreement_seed(None, || {
            let result = DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            );

            assert_eq!(
                result,
                Err(DirectMessageCryptoError::MissingKeyAgreementMasterSeed)
            );
        });
    }

    #[test]
    fn direct_message_hkdf_derivation_is_deterministic_and_distinct_from_legacy_v1() {
        let shared_secret = [0x5au8; 32];
        let hkdf_key_a =
            super::derive_direct_message_aead_key(&shared_secret).expect("hkdf key should derive");
        let hkdf_key_b =
            super::derive_direct_message_aead_key(&shared_secret).expect("hkdf key should derive");
        let legacy_key = super::derive_direct_message_aead_key_legacy(&shared_secret);

        assert_eq!(hkdf_key_a, hkdf_key_b);
        assert_ne!(hkdf_key_a, legacy_key);
    }

    #[test]
    fn direct_message_derivation_backend_markers_and_manual_helper_removal_contract() {
        assert_eq!(
            super::DIRECT_MESSAGE_HKDF_BACKEND_MARKER,
            "rustcrypto.hkdf.sha256.v1"
        );
        assert_eq!(
            super::DIRECT_MESSAGE_HMAC_BACKEND_MARKER,
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
    fn spec_c09_direct_message_engine_source_contract_enforces_non_clone_redacted_debug_and_drop_zeroize()
    {
        assert!(
            !SOURCE.contains("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct DirectMessageCryptoEngine"),
            "direct-message engine must not derive Clone"
        );
        assert!(
            SOURCE.contains("impl fmt::Debug for DirectMessageCryptoEngine"),
            "direct-message engine must define custom Debug"
        );
        assert!(
            SOURCE.contains("impl Drop for DirectMessageCryptoEngine"),
            "direct-message engine must define Drop"
        );
        assert!(
            SOURCE.contains("self.aead_key.zeroize();"),
            "direct-message engine Drop must zeroize aead_key"
        );
        assert!(
            SOURCE.contains("self.legacy_aead_key.zeroize();"),
            "direct-message engine Drop must zeroize legacy_aead_key"
        );
    }

    #[test]
    fn spec_c10_direct_message_engine_debug_output_redacts_sensitive_key_material() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let engine = DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            )
            .expect("engine init should succeed");
            let debug_output = format!("{engine:?}");
            assert!(
                debug_output.contains("sender_key_ref"),
                "debug output should preserve safe sender metadata"
            );
            assert!(
                debug_output.contains("recipient_key_ref"),
                "debug output should preserve safe recipient metadata"
            );
            assert!(
                !debug_output.contains("aead_key"),
                "debug output must not expose key field labels"
            );
            assert!(
                !debug_output.contains("legacy_aead_key"),
                "debug output must not expose legacy key field labels"
            );
        });
    }

    #[test]
    fn decrypt_accepts_legacy_v1_sha256_kdf_ciphertext_for_compatibility() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let sender_key_ref = "kamn:did:agent:alice#key-agreement-1";
            let recipient_key_ref = "kamn:did:agent:bob#key-agreement-1";

            let engine = DirectMessageCryptoEngine::new(sender_key_ref, recipient_key_ref)
                .expect("engine init should succeed");
            let sealed = legacy_v1_ciphertext(sender_key_ref, recipient_key_ref, "legacy-v1", 41);

            let plaintext = engine
                .decrypt(&sealed)
                .expect("legacy-v1 decrypt must succeed");
            assert_eq!(plaintext, "legacy-v1");
        });
    }

    #[test]
    fn display_messages_remain_stable_for_reason_taxonomy() {
        assert_eq!(
            DirectMessageCryptoError::InvalidKeyRef("sender").to_string(),
            "sender key reference must include #key-agreement"
        );
        assert_eq!(
            DirectMessageCryptoError::KeyRefMismatch("recipient").to_string(),
            "recipient key reference mismatch"
        );
        assert_eq!(
            DirectMessageCryptoError::KeyDerivationFailed.to_string(),
            "direct message key derivation failed"
        );
    }

    #[test]
    fn constructor_rejects_empty_sender_key_reference() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            assert_eq!(
                DirectMessageCryptoEngine::new("   ", "kamn:did:agent:bob#key-agreement-1"),
                Err(DirectMessageCryptoError::EmptyKeyRef("sender"))
            );
        });
    }

    #[test]
    fn constructor_rejects_empty_recipient_key_reference() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            assert_eq!(
                DirectMessageCryptoEngine::new("kamn:did:agent:alice#key-agreement-1", ""),
                Err(DirectMessageCryptoError::EmptyKeyRef("recipient"))
            );
        });
    }

    #[test]
    fn constructor_rejects_seed_hex_with_invalid_length() {
        with_key_agreement_seed(Some("abcd"), || {
            assert_eq!(
                DirectMessageCryptoEngine::new(
                    "kamn:did:agent:alice#key-agreement-1",
                    "kamn:did:agent:bob#key-agreement-1",
                ),
                Err(DirectMessageCryptoError::InvalidKeyAgreementMasterSeed)
            );
        });
    }

    #[test]
    fn constructor_rejects_seed_hex_with_invalid_characters() {
        with_key_agreement_seed(
            Some("zz112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"),
            || {
                assert_eq!(
                    DirectMessageCryptoEngine::new(
                        "kamn:did:agent:alice#key-agreement-1",
                        "kamn:did:agent:bob#key-agreement-1",
                    ),
                    Err(DirectMessageCryptoError::InvalidKeyAgreementMasterSeed)
                );
            },
        );
    }

    #[test]
    fn encrypt_rejects_empty_payload() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine = DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            )
            .expect("engine init should succeed");
            assert_eq!(
                engine.encrypt("", 1),
                Err(DirectMessageCryptoError::EmptyPayload)
            );
        });
    }

    #[test]
    fn encrypt_rejects_zero_nonce() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine = DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            )
            .expect("engine init should succeed");
            assert_eq!(
                engine.encrypt("payload", 0),
                Err(DirectMessageCryptoError::InvalidNonce(0))
            );
        });
    }

    #[test]
    fn decrypt_rejects_zero_nonce_in_ciphertext() {
        with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
            let mut engine = DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            )
            .expect("engine init should succeed");
            let mut sealed = engine.encrypt("payload", 22).expect("encrypt");
            sealed.nonce = 0;
            assert_eq!(
                engine.decrypt(&sealed),
                Err(DirectMessageCryptoError::InvalidNonce(0))
            );
        });
    }

    #[test]
    fn hex_decode_rejects_odd_length_inputs() {
        assert_eq!(
            super::hex_decode("abc"),
            Err(DirectMessageCryptoError::InvalidCiphertextEncoding)
        );
    }

    #[test]
    fn canonical_direct_message_aad_contains_expected_fields() {
        let aad = super::canonical_direct_message_aad(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
            99,
        );
        assert_eq!(
            aad,
            "X25519|XChaCha20-Poly1305|kamn:did:agent:alice#key-agreement-1|kamn:did:agent:bob#key-agreement-1|99"
        );
    }

    #[test]
    fn direct_message_nonce_bytes_are_deterministic_and_nonce_sensitive() {
        let nonce_7_first = super::direct_message_nonce_bytes(7);
        let nonce_7_second = super::direct_message_nonce_bytes(7);
        let nonce_8 = super::direct_message_nonce_bytes(8);

        assert_eq!(nonce_7_first, nonce_7_second);
        assert_ne!(nonce_7_first, nonce_8);
    }
}
