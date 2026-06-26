//! Group channel sender-key lifecycle and message protection contracts.

mod crypto_helpers;
mod engine;
mod errors;
mod models;
#[cfg(test)]
mod tests;
mod validation;

pub use engine::GroupChannelCryptoEngine;
pub use errors::GroupChannelCryptoError;
pub use models::{GroupMessageCiphertext, SenderKeyDistributionRecord};

use crypto_helpers::{
    compute_signature, derive_group_aead_key, derive_group_aead_key_legacy,
    derive_group_shared_secret, group_nonce_bytes, hex_decode, hex_encode,
    legacy_raw_prefix_group_nonce_bytes, zeroize_sender_key_history,
    zeroize_u64_keyed_sender_history,
};
use validation::{
    load_key_agreement_master_seed, validate_did, validate_recipients, validate_sender_key_ref,
};

/// Key-derivation algorithm identifier stamped on group ciphertext envelopes.
pub const GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM: &str = "X25519";
/// Cipher profile identifier stamped on group ciphertext envelopes.
pub const GROUP_MESSAGE_CIPHER_ALGORITHM: &str = "XChaCha20-Poly1305";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const GROUP_MESSAGE_AEAD_KDF_SALT_V2: &[u8] = b"kamn:group-message:aead-key:hkdf-salt:v2";
const GROUP_MESSAGE_AEAD_KDF_INFO_PREFIX_V2: &[u8] = b"kamn:group-message:aead-key:hkdf-info:v2:";
const GROUP_MESSAGE_NONCE_INFO_V2: &[u8] = b"kamn:group-message:nonce:v2:";
const GROUP_MESSAGE_NONCE_INFO_V1: &[u8] = b"kamn:group-message:nonce:v1:";
/// Marker asserting HKDF derivation is backed by RustCrypto hkdf crate.
pub const GROUP_MESSAGE_HKDF_BACKEND_MARKER: &str =
    kamn_crypto::hkdf_sha256::HKDF_SHA256_BACKEND_MARKER;
/// Marker asserting HMAC backend semantics are provided by RustCrypto primitives.
pub const GROUP_MESSAGE_HMAC_BACKEND_MARKER: &str =
    kamn_crypto::hkdf_sha256::HMAC_SHA256_BACKEND_MARKER;
const GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE: &str =
    "group_channel_crypto_invalid_sender_did";
const GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE: &str =
    "group_channel_crypto_invalid_recipient_did";
