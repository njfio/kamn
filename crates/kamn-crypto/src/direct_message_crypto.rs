/// Canonical direct-message key-agreement algorithm identifier.
pub const DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM: &str = "X25519";
/// Canonical direct-message cipher algorithm identifier.
pub const DIRECT_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";
pub(crate) const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
pub(crate) const DIRECT_MESSAGE_AEAD_KDF_SALT_V2: &[u8] =
    b"kamn:direct-message:aead-key:hkdf-salt:v2";
pub(crate) const DIRECT_MESSAGE_AEAD_KDF_INFO_V2: &[u8] =
    b"kamn:direct-message:aead-key:hkdf-info:v2";
pub(crate) const DIRECT_MESSAGE_NONCE_INFO_V2: &[u8] = b"kamn:direct-message:nonce:v2:";
pub(crate) const DIRECT_MESSAGE_NONCE_INFO_V1: &[u8] = b"kamn:direct-message:nonce:v1:";
/// Marker asserting HKDF derivation is backed by RustCrypto hkdf crate.
/// Source marker: rustcrypto.hkdf.sha256.v1.
pub const DIRECT_MESSAGE_HKDF_BACKEND_MARKER: &str = crate::hkdf_sha256::HKDF_SHA256_BACKEND_MARKER;
/// Marker asserting HMAC backend semantics are provided by RustCrypto primitives.
/// Source marker: rustcrypto.hmac.sha256.v1.
pub const DIRECT_MESSAGE_HMAC_BACKEND_MARKER: &str = crate::hkdf_sha256::HMAC_SHA256_BACKEND_MARKER;

mod cipher;
mod encoding;
mod engine;
mod errors;
mod key_agreement;
mod models;
mod validation;

pub use engine::DirectMessageCryptoEngine;
pub use errors::DirectMessageCryptoError;
pub use models::DirectMessageCiphertext;

pub(crate) use cipher::{
    canonical_direct_message_aad, decrypt_with_compatibility_candidates, direct_message_nonce_bytes,
};
pub(crate) use encoding::{hex_decode, hex_encode};
pub(crate) use key_agreement::{
    derive_direct_message_aead_key, derive_direct_message_aead_key_legacy,
    derive_x25519_shared_secret, load_key_agreement_master_seed, validate_key_ref,
    validate_key_ref_match,
};
pub(crate) use validation::{
    decode_combined_ciphertext, validate_ciphertext_context, validate_encrypt_request,
};

#[cfg(test)]
pub(crate) use cipher::legacy_direct_message_nonce_bytes_raw_prefix_v1;

#[cfg(test)]
mod tests;
