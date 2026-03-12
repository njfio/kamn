use super::{
    DirectMessageCryptoError, DIRECT_MESSAGE_CIPHER_ALGORITHM,
    DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM, DIRECT_MESSAGE_NONCE_INFO_V1,
    DIRECT_MESSAGE_NONCE_INFO_V2,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};

pub(crate) fn direct_message_nonce_bytes(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    nonce: u64,
) -> [u8; 24] {
    let mut hasher = Sha256::new();
    hasher.update(DIRECT_MESSAGE_NONCE_INFO_V2);
    hasher.update(sender_key_ref.as_bytes());
    hasher.update([0]);
    hasher.update(recipient_key_ref.as_bytes());
    hasher.update([0]);
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 24];
    out.copy_from_slice(&digest[..24]);
    out
}

pub(crate) fn legacy_direct_message_nonce_bytes_raw_prefix_v1(nonce: u64) -> [u8; 24] {
    let mut out = [0u8; 24];
    out[..8].copy_from_slice(&nonce.to_le_bytes());

    let mut hasher = Sha256::new();
    hasher.update(DIRECT_MESSAGE_NONCE_INFO_V1);
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    out[8..].copy_from_slice(&digest[..16]);
    out
}

fn decrypt_with_nonce_bytes(
    key: &[u8; 32],
    nonce_bytes: [u8; 24],
    combined: &[u8],
    aad: &str,
) -> Result<Vec<u8>, DirectMessageCryptoError> {
    let xnonce = XNonce::from(nonce_bytes);
    XChaCha20Poly1305::new(key.into())
        .decrypt(
            &xnonce,
            Payload {
                msg: combined,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| DirectMessageCryptoError::IntegrityCheckFailed)
}

pub(crate) fn decrypt_with_compatibility_candidates(
    aead_key: &[u8; 32],
    legacy_aead_key: &[u8; 32],
    sender_key_ref: &str,
    recipient_key_ref: &str,
    nonce: u64,
    combined: &[u8],
    aad: &str,
) -> Result<Vec<u8>, DirectMessageCryptoError> {
    let nonce_candidates = [
        direct_message_nonce_bytes(sender_key_ref, recipient_key_ref, nonce),
        legacy_direct_message_nonce_bytes_raw_prefix_v1(nonce),
    ];

    for key in [aead_key, legacy_aead_key] {
        for nonce_bytes in nonce_candidates {
            if let Ok(plaintext) = decrypt_with_nonce_bytes(key, nonce_bytes, combined, aad) {
                return Ok(plaintext);
            }
        }
    }

    Err(DirectMessageCryptoError::IntegrityCheckFailed)
}

pub(crate) fn canonical_direct_message_aad(
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
