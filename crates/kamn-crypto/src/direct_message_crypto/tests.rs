use super::{
    canonical_direct_message_aad, derive_direct_message_aead_key,
    derive_direct_message_aead_key_legacy, derive_x25519_shared_secret, direct_message_nonce_bytes,
    hex_decode, hex_encode, legacy_direct_message_nonce_bytes_raw_prefix_v1,
    load_key_agreement_master_seed, DirectMessageCiphertext, DirectMessageCryptoEngine,
    DirectMessageCryptoError, DIRECT_MESSAGE_CIPHER_ALGORITHM, DIRECT_MESSAGE_HKDF_BACKEND_MARKER,
    DIRECT_MESSAGE_HMAC_BACKEND_MARKER, DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
    KEY_AGREEMENT_MASTER_SEED_ENV,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use std::sync::{Mutex, OnceLock};

const TEST_KEY_SEED_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const SOURCE: &str = include_str!("../direct_message_crypto.rs");

fn key_agreement_seed_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_key_agreement_seed<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
    let _guard = key_agreement_seed_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    let previous = std::env::var(KEY_AGREEMENT_MASTER_SEED_ENV).ok();
    match value {
        Some(seed) => std::env::set_var(KEY_AGREEMENT_MASTER_SEED_ENV, seed),
        None => std::env::remove_var(KEY_AGREEMENT_MASTER_SEED_ENV),
    }

    let output = run();

    match previous {
        Some(seed) => std::env::set_var(KEY_AGREEMENT_MASTER_SEED_ENV, seed),
        None => std::env::remove_var(KEY_AGREEMENT_MASTER_SEED_ENV),
    }

    output
}

fn legacy_v1_ciphertext(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    plaintext: &str,
    nonce: u64,
) -> DirectMessageCiphertext {
    build_legacy_ciphertext(
        sender_key_ref,
        recipient_key_ref,
        plaintext,
        nonce,
        derive_direct_message_aead_key_legacy,
        legacy_direct_message_nonce_bytes_raw_prefix_v1(nonce),
    )
}

fn legacy_v2_raw_prefix_ciphertext(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    plaintext: &str,
    nonce: u64,
) -> DirectMessageCiphertext {
    build_legacy_ciphertext(
        sender_key_ref,
        recipient_key_ref,
        plaintext,
        nonce,
        |shared_secret| {
            derive_direct_message_aead_key(shared_secret).expect("hkdf derive should work")
        },
        legacy_direct_message_nonce_bytes_raw_prefix_v1(nonce),
    )
}

fn build_legacy_ciphertext(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    plaintext: &str,
    nonce: u64,
    derive_key: impl FnOnce(&[u8; 32]) -> [u8; 32],
    nonce_bytes: [u8; 24],
) -> DirectMessageCiphertext {
    let master_seed = load_key_agreement_master_seed().expect("master seed should be available");
    let shared_secret =
        derive_x25519_shared_secret(sender_key_ref, recipient_key_ref, &master_seed);
    let aead_key = derive_key(&shared_secret);
    let aad = canonical_direct_message_aad(sender_key_ref, recipient_key_ref, nonce);
    let payload = Payload {
        msg: plaintext.as_bytes(),
        aad: aad.as_bytes(),
    };
    let mut sealed = XChaCha20Poly1305::new((&aead_key).into())
        .encrypt(&XNonce::from(nonce_bytes), payload)
        .expect("legacy encryption should succeed");
    let auth_tag = sealed.split_off(sealed.len() - 16);
    build_ciphertext(sender_key_ref, recipient_key_ref, nonce, sealed, auth_tag)
}

fn build_ciphertext(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    nonce: u64,
    ciphertext: Vec<u8>,
    auth_tag: Vec<u8>,
) -> DirectMessageCiphertext {
    DirectMessageCiphertext {
        key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
        cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
        sender_key_ref: sender_key_ref.to_owned(),
        recipient_key_ref: recipient_key_ref.to_owned(),
        nonce,
        ciphertext: hex_encode(&ciphertext),
        auth_tag: hex_encode(&auth_tag),
    }
}

mod basic_tests;
mod compatibility_tests;
mod contract_tests;
mod validation_tests;
