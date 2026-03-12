use super::encoding::hex_encode;
use sha2::{Digest, Sha256};

pub(crate) fn compute_signature(
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
