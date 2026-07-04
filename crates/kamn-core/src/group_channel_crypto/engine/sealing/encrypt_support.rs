use super::super::GroupChannelCryptoEngine;
use crate::group_channel_crypto::{
    compute_signature, GroupMessageCiphertext, GROUP_MESSAGE_CIPHER_ALGORITHM,
    GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM,
};

pub(super) struct EncryptionMaterial {
    pub(super) key_generation: u64,
    pub(super) shared_secret: [u8; 32],
    pub(super) aead_key: [u8; 32],
}

pub(super) fn build_ciphertext(
    engine: &GroupChannelCryptoEngine,
    sender_did: &str,
    nonce: u64,
    encryption: EncryptionMaterial,
    sealed: (String, String),
) -> GroupMessageCiphertext {
    let key_generation = encryption.key_generation;
    let signature = build_signature(engine, sender_did, nonce, &encryption, &sealed);
    build_group_message(engine, sender_did, nonce, key_generation, sealed, signature)
}

fn build_signature(
    engine: &GroupChannelCryptoEngine,
    sender_did: &str,
    nonce: u64,
    encryption: &EncryptionMaterial,
    sealed: &(String, String),
) -> String {
    let (ciphertext, auth_tag) = sealed;
    ciphertext_signature(engine, sender_did, nonce, encryption, ciphertext, auth_tag)
}

fn build_group_message(
    engine: &GroupChannelCryptoEngine,
    sender_did: &str,
    nonce: u64,
    key_generation: u64,
    sealed: (String, String),
    signature: String,
) -> GroupMessageCiphertext {
    let (ciphertext, auth_tag) = sealed;
    ciphertext_message(
        engine,
        sender_did,
        nonce,
        key_generation,
        ciphertext,
        auth_tag,
        signature,
    )
}

fn ciphertext_signature(
    engine: &GroupChannelCryptoEngine,
    sender_did: &str,
    nonce: u64,
    encryption: &EncryptionMaterial,
    ciphertext: &str,
    auth_tag: &str,
) -> String {
    compute_signature(
        &encryption.shared_secret,
        &engine.channel_id,
        sender_did,
        encryption.key_generation,
        nonce,
        ciphertext,
        auth_tag,
    )
}

fn ciphertext_message(
    engine: &GroupChannelCryptoEngine,
    sender_did: &str,
    nonce: u64,
    key_generation: u64,
    ciphertext: String,
    auth_tag: String,
    signature: String,
) -> GroupMessageCiphertext {
    GroupMessageCiphertext {
        key_derivation_algorithm: GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM.to_owned(),
        cipher_algorithm: GROUP_MESSAGE_CIPHER_ALGORITHM.to_owned(),
        channel_id: engine.channel_id.clone(),
        sender_did: sender_did.to_owned(),
        key_generation,
        nonce,
        ciphertext,
        auth_tag,
        signature,
    }
}
