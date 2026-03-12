use super::{
    DirectMessageCryptoError, DIRECT_MESSAGE_AEAD_KDF_INFO_V2,
    DIRECT_MESSAGE_AEAD_KDF_SALT_V2, KEY_AGREEMENT_MASTER_SEED_ENV,
};
use sha2::{Digest, Sha256, Sha512};
use std::env;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

pub(crate) fn validate_key_ref(
    role: &'static str,
    key_ref: &str,
) -> Result<(), DirectMessageCryptoError> {
    if key_ref.trim().is_empty() {
        return Err(DirectMessageCryptoError::EmptyKeyRef(role));
    }
    if !key_ref.contains("#key-agreement") {
        return Err(DirectMessageCryptoError::InvalidKeyRef(role));
    }
    Ok(())
}

pub(crate) fn validate_key_ref_match(
    role: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), DirectMessageCryptoError> {
    if expected != actual {
        return Err(DirectMessageCryptoError::KeyRefMismatch(role));
    }
    Ok(())
}

pub(crate) fn load_key_agreement_master_seed() -> Result<[u8; 32], DirectMessageCryptoError> {
    let mut seed_hex =
        env::var(KEY_AGREEMENT_MASTER_SEED_ENV).map_err(|_| DirectMessageCryptoError::MissingKeyAgreementMasterSeed)?;
    let seed = parse_fixed_hex_32(seed_hex.trim());
    seed_hex.zeroize();
    seed
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

pub(crate) fn derive_x25519_shared_secret(
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

pub(crate) fn derive_direct_message_aead_key(
    shared_secret: &[u8; 32],
) -> Result<[u8; 32], DirectMessageCryptoError> {
    crate::hkdf_sha256::derive_key_32(
        DIRECT_MESSAGE_AEAD_KDF_SALT_V2,
        shared_secret,
        DIRECT_MESSAGE_AEAD_KDF_INFO_V2,
    )
    .map_err(|_| DirectMessageCryptoError::KeyDerivationFailed)
}

pub(crate) fn derive_direct_message_aead_key_legacy(shared_secret: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"kamn:direct-message:aead-key:v1:");
    hasher.update(shared_secret);
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest[..32]);
    key
}
