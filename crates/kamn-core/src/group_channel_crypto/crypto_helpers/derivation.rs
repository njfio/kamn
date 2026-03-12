use super::super::{
    GroupChannelCryptoError, GROUP_MESSAGE_AEAD_KDF_INFO_PREFIX_V2,
    GROUP_MESSAGE_AEAD_KDF_SALT_V2,
};
use sha2::{Digest, Sha256, Sha512};
use x25519_dalek::{PublicKey, StaticSecret};

pub(crate) fn derive_group_shared_secret(
    channel_id: &str,
    sender_key_ref: &str,
    generation: u64,
    master_seed: &[u8; 32],
) -> [u8; 32] {
    let sender_private = derive_x25519_private_key(master_seed, sender_key_ref);
    let material_ref = format!("{channel_id}#group-key-material-{generation}");
    let channel_public = derive_x25519_public_key(master_seed, material_ref.as_str());
    sender_private.diffie_hellman(&channel_public).to_bytes()
}

pub(crate) fn derive_group_aead_key(
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
    kamn_crypto::hkdf_sha256::derive_key_32(GROUP_MESSAGE_AEAD_KDF_SALT_V2, shared_secret, &info)
        .map_err(|_| GroupChannelCryptoError::KeyDerivationFailed)
}

pub(crate) fn derive_group_aead_key_legacy(
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
