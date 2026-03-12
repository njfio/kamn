use super::super::{
    compute_signature, derive_group_aead_key_legacy, derive_group_shared_secret, group_nonce_bytes,
    hex_decode, hex_encode, legacy_raw_prefix_group_nonce_bytes, load_key_agreement_master_seed,
    GroupMessageCiphertext, GROUP_MESSAGE_CIPHER_ALGORITHM, GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM,
    KEY_AGREEMENT_MASTER_SEED_ENV,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

pub(super) const SOURCE: &str = include_str!("../../group_channel_crypto.rs");
pub(super) const PRODUCTION_SOURCE: &str = concat!(
    include_str!("../../group_channel_crypto.rs"),
    include_str!("../models.rs"),
    include_str!("../errors.rs"),
    include_str!("../errors/display.rs"),
    include_str!("../engine.rs"),
    include_str!("../engine/lifecycle.rs"),
    include_str!("../engine/sealing.rs"),
    include_str!("../engine/sealing/encrypt.rs"),
    include_str!("../engine/sealing/decrypt.rs"),
    include_str!("../validation.rs"),
    include_str!("../crypto_helpers.rs"),
    include_str!("../crypto_helpers/derivation.rs"),
    include_str!("../crypto_helpers/encoding.rs"),
    include_str!("../crypto_helpers/nonce.rs"),
    include_str!("../crypto_helpers/signature.rs"),
    include_str!("../crypto_helpers/zeroize_support.rs")
);
pub(super) const TEST_KEY_SEED_HEX: &str =
    "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

pub(super) fn with_key_agreement_seed<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
    let _guard = crate::crypto_test_env_lock::key_agreement_seed_env_lock()
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

pub(super) fn legacy_v1_ciphertext(
    channel_id: &str,
    sender_did: &str,
    sender_key_ref: &str,
    generation: u64,
    nonce: u64,
    plaintext: &str,
) -> GroupMessageCiphertext {
    let (shared_secret, sealed) = legacy_message_parts(
        channel_id,
        sender_did,
        sender_key_ref,
        generation,
        nonce,
        plaintext,
    );
    legacy_ciphertext(
        channel_id,
        sender_did,
        generation,
        nonce,
        &shared_secret,
        sealed,
    )
}

fn legacy_shared_secret(channel_id: &str, sender_key_ref: &str, generation: u64) -> [u8; 32] {
    let master_seed = load_key_agreement_master_seed().expect("master seed should be available");
    derive_group_shared_secret(channel_id, sender_key_ref, generation, &master_seed)
}

fn encrypt_legacy_payload(
    shared_secret: &[u8; 32],
    channel_id: &str,
    sender_did: &str,
    generation: u64,
    nonce: u64,
    plaintext: &str,
) -> (String, String) {
    let legacy_key = derive_group_aead_key_legacy(shared_secret, channel_id, generation);
    let cipher = XChaCha20Poly1305::new((&legacy_key).into());
    let xnonce = XNonce::from(group_nonce_bytes(sender_did, generation, nonce));
    let mut sealed = cipher
        .encrypt(
            &xnonce,
            Payload {
                msg: plaintext.as_bytes(),
                aad: &[],
            },
        )
        .expect("legacy encryption should succeed");
    let auth_tag = sealed.split_off(sealed.len() - 16);
    (hex_encode(&sealed), hex_encode(&auth_tag))
}

fn legacy_ciphertext(
    channel_id: &str,
    sender_did: &str,
    generation: u64,
    nonce: u64,
    shared_secret: &[u8; 32],
    sealed: (String, String),
) -> GroupMessageCiphertext {
    let (ciphertext_hex, auth_tag_hex) = sealed;
    GroupMessageCiphertext {
        key_derivation_algorithm: GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM.to_owned(),
        cipher_algorithm: GROUP_MESSAGE_CIPHER_ALGORITHM.to_owned(),
        channel_id: channel_id.to_owned(),
        sender_did: sender_did.to_owned(),
        key_generation: generation,
        nonce,
        ciphertext: ciphertext_hex.clone(),
        auth_tag: auth_tag_hex.clone(),
        signature: compute_signature(
            shared_secret,
            channel_id,
            sender_did,
            generation,
            nonce,
            ciphertext_hex.as_str(),
            auth_tag_hex.as_str(),
        ),
    }
}

fn legacy_message_parts(
    channel_id: &str,
    sender_did: &str,
    sender_key_ref: &str,
    generation: u64,
    nonce: u64,
    plaintext: &str,
) -> ([u8; 32], (String, String)) {
    let shared_secret = legacy_shared_secret(channel_id, sender_key_ref, generation);
    let sealed = legacy_envelope_parts(
        &shared_secret,
        channel_id,
        sender_did,
        generation,
        nonce,
        plaintext,
    );
    (shared_secret, sealed)
}

fn legacy_envelope_parts(
    shared_secret: &[u8; 32],
    channel_id: &str,
    sender_did: &str,
    generation: u64,
    nonce: u64,
    plaintext: &str,
) -> (String, String) {
    encrypt_legacy_payload(
        shared_secret,
        channel_id,
        sender_did,
        generation,
        nonce,
        plaintext,
    )
}

pub(super) fn legacy_raw_nonce(sender_did: &str, generation: u64, nonce: u64) -> [u8; 24] {
    legacy_raw_prefix_group_nonce_bytes(sender_did, generation, nonce)
}

pub(super) fn decode_ciphertext_parts(sealed: &GroupMessageCiphertext) -> Vec<u8> {
    let mut combined = hex_decode(&sealed.ciphertext).expect("ciphertext hex");
    combined.extend_from_slice(&hex_decode(&sealed.auth_tag).expect("auth tag hex"));
    combined
}
