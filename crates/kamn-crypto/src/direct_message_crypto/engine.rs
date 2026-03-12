use super::{
    canonical_direct_message_aad, decode_combined_ciphertext,
    decrypt_with_compatibility_candidates, derive_direct_message_aead_key,
    derive_direct_message_aead_key_legacy, derive_x25519_shared_secret,
    direct_message_nonce_bytes, load_key_agreement_master_seed, validate_ciphertext_context,
    validate_encrypt_request, validate_key_ref, DirectMessageCiphertext,
    DirectMessageCryptoError, DIRECT_MESSAGE_CIPHER_ALGORITHM,
    DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use std::collections::BTreeSet;
use std::fmt;
use zeroize::Zeroize;

/// Direct-message crypto engine with nonce reuse protection.
#[derive(PartialEq, Eq)]
pub struct DirectMessageCryptoEngine {
    sender_key_ref: String,
    recipient_key_ref: String,
    pub(crate) aead_key: [u8; 32],
    legacy_aead_key: [u8; 32],
    used_nonces: BTreeSet<u64>,
}

impl fmt::Debug for DirectMessageCryptoEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DirectMessageCryptoEngine")
            .field("sender_key_ref", &self.sender_key_ref)
            .field("recipient_key_ref", &self.recipient_key_ref)
            .field("used_nonce_count", &self.used_nonces.len())
            .finish()
    }
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
        validate_encrypt_request(&mut self.used_nonces, plaintext, nonce)?;

        let cipher = XChaCha20Poly1305::new((&self.aead_key).into());
        let nonce_bytes = direct_message_nonce_bytes(
            self.sender_key_ref.as_str(),
            self.recipient_key_ref.as_str(),
            nonce,
        );
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
            ciphertext: super::hex_encode(&sealed),
            auth_tag: super::hex_encode(&auth_tag),
        })
    }

    /// Decrypts ciphertext after algorithm and integrity validation.
    pub fn decrypt(
        &self,
        sealed: &DirectMessageCiphertext,
    ) -> Result<String, DirectMessageCryptoError> {
        validate_ciphertext_context(
            self.sender_key_ref.as_str(),
            self.recipient_key_ref.as_str(),
            sealed,
        )?;
        let combined = decode_combined_ciphertext(sealed)?;
        let aad = canonical_direct_message_aad(
            sealed.sender_key_ref.as_str(),
            sealed.recipient_key_ref.as_str(),
            sealed.nonce,
        );
        let plaintext = decrypt_with_compatibility_candidates(
            &self.aead_key,
            &self.legacy_aead_key,
            sealed.sender_key_ref.as_str(),
            sealed.recipient_key_ref.as_str(),
            sealed.nonce,
            &combined,
            aad.as_str(),
        )?;
        String::from_utf8(plaintext)
            .map_err(|_| DirectMessageCryptoError::InvalidCiphertextEncoding)
    }
}

impl Drop for DirectMessageCryptoEngine {
    fn drop(&mut self) {
        self.aead_key.zeroize();
        self.legacy_aead_key.zeroize();
        self.used_nonces.clear();
    }
}
