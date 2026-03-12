use super::super::GroupChannelCryptoEngine;
use super::encrypt_support::{build_ciphertext, EncryptionMaterial};
use crate::group_channel_crypto::{
    derive_group_aead_key, derive_group_shared_secret, group_nonce_bytes, validate_did,
    GroupChannelCryptoError, GroupMessageCiphertext,
    GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

impl GroupChannelCryptoEngine {
    /// Encrypts plaintext for a sender using the active sender-key generation.
    pub fn encrypt(
        &mut self,
        sender_did: &str,
        plaintext: &str,
        nonce: u64,
    ) -> Result<GroupMessageCiphertext, GroupChannelCryptoError> {
        validate_encrypt_request(sender_did, plaintext, nonce)?;
        let encryption = prepare_encryption(self, sender_did, nonce)?;
        let sealed = seal_payload(
            &encryption.aead_key,
            sender_did,
            encryption.key_generation,
            plaintext,
            nonce,
        )?;
        Ok(build_ciphertext(
            self, sender_did, nonce, encryption, sealed,
        ))
    }
}

fn prepare_encryption(
    engine: &mut GroupChannelCryptoEngine,
    sender_did: &str,
    nonce: u64,
) -> Result<EncryptionMaterial, GroupChannelCryptoError> {
    let active_generation = engine.active_sender_key_generation(sender_did)?;
    let record = engine
        .sender_key_record(sender_did, active_generation)?
        .clone();
    reserve_nonce(engine, sender_did, record.key_generation, nonce)?;
    let shared_secret = derive_shared_secret(engine, &record)?;
    let aead_key = derive_group_aead_key(
        &shared_secret,
        engine.channel_id.as_str(),
        record.key_generation,
    )?;
    Ok(EncryptionMaterial {
        key_generation: record.key_generation,
        shared_secret,
        aead_key,
    })
}

fn derive_shared_secret(
    engine: &GroupChannelCryptoEngine,
    record: &crate::group_channel_crypto::SenderKeyDistributionRecord,
) -> Result<[u8; 32], GroupChannelCryptoError> {
    let master_seed = engine.cached_master_seed()?;
    Ok(derive_group_shared_secret(
        engine.channel_id.as_str(),
        record.sender_key_ref.as_str(),
        record.key_generation,
        &master_seed,
    ))
}

fn validate_encrypt_request(
    sender_did: &str,
    plaintext: &str,
    nonce: u64,
) -> Result<(), GroupChannelCryptoError> {
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
    Ok(())
}

fn reserve_nonce(
    engine: &mut GroupChannelCryptoEngine,
    sender_did: &str,
    generation: u64,
    nonce: u64,
) -> Result<(), GroupChannelCryptoError> {
    let nonce_key = (sender_did.to_owned(), generation, nonce);
    if engine.used_nonces.insert(nonce_key) {
        return Ok(());
    }
    Err(GroupChannelCryptoError::NonceReuse(nonce))
}

fn seal_payload(
    aead_key: &[u8; 32],
    sender_did: &str,
    generation: u64,
    plaintext: &str,
    nonce: u64,
) -> Result<(String, String), GroupChannelCryptoError> {
    let xnonce = XNonce::from(group_nonce_bytes(sender_did, generation, nonce));
    let cipher = XChaCha20Poly1305::new(aead_key.into());
    let mut sealed = cipher
        .encrypt(
            &xnonce,
            Payload {
                msg: plaintext.as_bytes(),
                aad: &[],
            },
        )
        .map_err(|_| GroupChannelCryptoError::EncryptionFailed)?;
    let auth_tag = sealed.split_off(sealed.len() - 16);
    Ok((
        crate::group_channel_crypto::hex_encode(&sealed),
        crate::group_channel_crypto::hex_encode(&auth_tag),
    ))
}
