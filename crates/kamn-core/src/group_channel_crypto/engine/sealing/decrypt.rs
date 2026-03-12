use super::super::GroupChannelCryptoEngine;
use crate::group_channel_crypto::{
    compute_signature, derive_group_aead_key, derive_group_aead_key_legacy,
    derive_group_shared_secret, group_nonce_bytes, hex_decode, legacy_raw_prefix_group_nonce_bytes,
    validate_did, GroupChannelCryptoError, GroupMessageCiphertext, SenderKeyDistributionRecord,
    GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE, GROUP_MESSAGE_CIPHER_ALGORITHM,
    GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

impl GroupChannelCryptoEngine {
    /// Decrypts a sealed group message for an authorized recipient DID.
    pub fn decrypt(
        &self,
        recipient_did: &str,
        sealed: &GroupMessageCiphertext,
    ) -> Result<String, GroupChannelCryptoError> {
        validate_decrypt_request(self, recipient_did, sealed)?;
        let record = self.sender_key_record(&sealed.sender_did, sealed.key_generation)?;
        authorize_recipient(record, recipient_did, sealed)?;
        let master_seed = self.cached_master_seed()?;
        let shared_secret = derive_group_shared_secret(
            self.channel_id.as_str(),
            record.sender_key_ref.as_str(),
            sealed.key_generation,
            &master_seed,
        );
        validate_signature(&shared_secret, sealed)?;
        let combined = decode_payload(sealed)?;
        let plaintext = try_decrypt_variants(self, &shared_secret, sealed, &combined)?;
        String::from_utf8(plaintext).map_err(|_| GroupChannelCryptoError::InvalidCiphertextEncoding)
    }
}

fn validate_decrypt_request(
    engine: &GroupChannelCryptoEngine,
    recipient_did: &str,
    sealed: &GroupMessageCiphertext,
) -> Result<(), GroupChannelCryptoError> {
    validate_recipient_did(recipient_did)?;
    validate_algorithms(sealed)?;
    validate_channel(engine, sealed)?;
    validate_nonce(sealed.nonce)
}

fn validate_recipient_did(recipient_did: &str) -> Result<(), GroupChannelCryptoError> {
    validate_did(
        recipient_did,
        "recipient_did",
        GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE,
    )
    .map(|_| ())
}

fn validate_algorithms(sealed: &GroupMessageCiphertext) -> Result<(), GroupChannelCryptoError> {
    if sealed.key_derivation_algorithm == GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM
        && sealed.cipher_algorithm == GROUP_MESSAGE_CIPHER_ALGORITHM
    {
        return Ok(());
    }
    Err(GroupChannelCryptoError::AlgorithmMismatch)
}

fn validate_channel(
    engine: &GroupChannelCryptoEngine,
    sealed: &GroupMessageCiphertext,
) -> Result<(), GroupChannelCryptoError> {
    if sealed.channel_id == engine.channel_id {
        return Ok(());
    }
    Err(GroupChannelCryptoError::ChannelMismatch {
        expected: engine.channel_id.clone(),
        actual: sealed.channel_id.clone(),
    })
}

fn validate_nonce(nonce: u64) -> Result<(), GroupChannelCryptoError> {
    if nonce > 0 {
        return Ok(());
    }
    Err(GroupChannelCryptoError::InvalidNonce(nonce))
}

fn authorize_recipient(
    record: &SenderKeyDistributionRecord,
    recipient_did: &str,
    sealed: &GroupMessageCiphertext,
) -> Result<(), GroupChannelCryptoError> {
    if record.recipient_allowlist.contains(recipient_did) {
        return Ok(());
    }
    Err(GroupChannelCryptoError::RecipientNotAuthorized {
        recipient_did: recipient_did.to_owned(),
        sender_did: sealed.sender_did.clone(),
        key_generation: sealed.key_generation,
    })
}

fn validate_signature(
    shared_secret: &[u8; 32],
    sealed: &GroupMessageCiphertext,
) -> Result<(), GroupChannelCryptoError> {
    let expected = compute_signature(
        shared_secret,
        &sealed.channel_id,
        &sealed.sender_did,
        sealed.key_generation,
        sealed.nonce,
        &sealed.ciphertext,
        &sealed.auth_tag,
    );
    if crate::constant_time_eq::constant_time_eq_str(expected.as_str(), sealed.signature.as_str()) {
        return Ok(());
    }
    Err(GroupChannelCryptoError::SignatureMismatch)
}

fn decode_payload(sealed: &GroupMessageCiphertext) -> Result<Vec<u8>, GroupChannelCryptoError> {
    let mut combined = hex_decode(&sealed.ciphertext)?;
    let auth_tag =
        hex_decode(&sealed.auth_tag).map_err(|_| GroupChannelCryptoError::IntegrityCheckFailed)?;
    combined.extend_from_slice(&auth_tag);
    Ok(combined)
}

fn try_decrypt_variants(
    engine: &GroupChannelCryptoEngine,
    shared_secret: &[u8; 32],
    sealed: &GroupMessageCiphertext,
    combined: &[u8],
) -> Result<Vec<u8>, GroupChannelCryptoError> {
    let aead_key_v2 = derive_group_aead_key(
        shared_secret,
        engine.channel_id.as_str(),
        sealed.key_generation,
    )?;
    let aead_key_v1 = derive_group_aead_key_legacy(
        shared_secret,
        engine.channel_id.as_str(),
        sealed.key_generation,
    );
    let nonce_candidates = nonce_candidates(sealed);
    for key in [&aead_key_v2, &aead_key_v1] {
        for nonce_bytes in nonce_candidates {
            if let Ok(value) = decrypt_with(key, nonce_bytes, combined) {
                return Ok(value);
            }
        }
    }
    Err(GroupChannelCryptoError::IntegrityCheckFailed)
}

fn nonce_candidates(sealed: &GroupMessageCiphertext) -> [[u8; 24]; 2] {
    [
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
    ]
}

fn decrypt_with(
    key: &[u8; 32],
    nonce_bytes: [u8; 24],
    combined: &[u8],
) -> Result<Vec<u8>, GroupChannelCryptoError> {
    let xnonce = XNonce::from(nonce_bytes);
    XChaCha20Poly1305::new(key.into())
        .decrypt(
            &xnonce,
            Payload {
                msg: combined,
                aad: &[],
            },
        )
        .map_err(|_| GroupChannelCryptoError::IntegrityCheckFailed)
}
