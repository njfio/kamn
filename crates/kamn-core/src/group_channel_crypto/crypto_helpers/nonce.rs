use super::super::{GROUP_MESSAGE_NONCE_INFO_V1, GROUP_MESSAGE_NONCE_INFO_V2};
use sha2::{Digest, Sha256};

pub(crate) fn group_nonce_bytes(sender_did: &str, generation: u64, nonce: u64) -> [u8; 24] {
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

pub(crate) fn legacy_raw_prefix_group_nonce_bytes(
    sender_did: &str,
    generation: u64,
    nonce: u64,
) -> [u8; 24] {
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
