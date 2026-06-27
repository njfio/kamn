use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use kamn_crypto::direct_message_crypto::{
    DirectMessageCiphertext, DirectMessageCryptoEngine, DirectMessageCryptoError,
    DIRECT_MESSAGE_CIPHER_ALGORITHM, DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
use sha2::{Digest, Sha256, Sha512};
use std::sync::{Mutex, OnceLock};
use x25519_dalek::{PublicKey, StaticSecret};

const TEST_KEY_SEED_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const DIRECT_MESSAGE_AEAD_KDF_SALT_V2: &[u8] = b"kamn:direct-message:aead-key:hkdf-salt:v2";
const DIRECT_MESSAGE_AEAD_KDF_INFO_V2: &[u8] = b"kamn:direct-message:aead-key:hkdf-info:v2";
const SENDER_KEY_REF: &str = "kamn:did:agent:alice#key-agreement-1";
const RECIPIENT_KEY_REF: &str = "kamn:did:agent:bob#key-agreement-1";
const ENGINE_SOURCE: &str = include_str!("../src/direct_message_crypto/engine.rs");
const KEY_AGREEMENT_SOURCE: &str = include_str!("../src/direct_message_crypto/key_agreement.rs");

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

fn parse_seed_hex() -> [u8; 32] {
    let mut seed = [0u8; 32];
    for (index, chunk) in TEST_KEY_SEED_HEX.as_bytes().chunks_exact(2).enumerate() {
        let encoded = std::str::from_utf8(chunk).expect("seed hex should be utf8");
        seed[index] = u8::from_str_radix(encoded, 16).expect("seed hex should be valid");
    }
    seed
}

fn derive_private_key(key_ref: &str) -> StaticSecret {
    let mut hasher = Sha512::new();
    hasher.update(b"kamn:x25519:key-ref:v1:");
    hasher.update(parse_seed_hex());
    hasher.update(key_ref.as_bytes());
    let digest = hasher.finalize();
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&digest[..32]);
    StaticSecret::from(key_bytes)
}

fn derive_shared_secret(sender_key_ref: &str, recipient_key_ref: &str) -> [u8; 32] {
    let sender_private = derive_private_key(sender_key_ref);
    let recipient_public = PublicKey::from(&derive_private_key(recipient_key_ref));
    sender_private.diffie_hellman(&recipient_public).to_bytes()
}

fn derive_aead_key(shared_secret: &[u8; 32]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(DIRECT_MESSAGE_AEAD_KDF_SALT_V2), shared_secret);
    let mut key = [0u8; 32];
    hkdf.expand(DIRECT_MESSAGE_AEAD_KDF_INFO_V2, &mut key)
        .expect("hkdf should derive");
    key
}

fn direct_message_nonce_bytes(nonce: u64) -> [u8; 24] {
    let mut out = [0u8; 24];
    out[..8].copy_from_slice(&nonce.to_le_bytes());
    let mut hasher = Sha256::new();
    hasher.update(b"kamn:direct-message:nonce:v1:");
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    out[8..].copy_from_slice(&digest[..16]);
    out
}

fn canonical_aad(sender_key_ref: &str, recipient_key_ref: &str, nonce: u64) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
        DIRECT_MESSAGE_CIPHER_ALGORITHM,
        sender_key_ref,
        recipient_key_ref,
        nonce
    )
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

fn ciphertext_with_raw_plaintext(
    sender_key_ref: &str,
    recipient_key_ref: &str,
    nonce: u64,
    plaintext: &[u8],
) -> DirectMessageCiphertext {
    let shared_secret = derive_shared_secret(sender_key_ref, recipient_key_ref);
    let key = derive_aead_key(&shared_secret);
    let cipher = XChaCha20Poly1305::new((&key).into());
    let xnonce = XNonce::from(direct_message_nonce_bytes(nonce));
    let aad = canonical_aad(sender_key_ref, recipient_key_ref, nonce);
    let payload = Payload {
        msg: plaintext,
        aad: aad.as_bytes(),
    };
    let mut sealed = cipher
        .encrypt(&xnonce, payload)
        .expect("encryption should succeed");
    let auth_tag = sealed.split_off(sealed.len() - 16);
    DirectMessageCiphertext {
        key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
        cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
        sender_key_ref: sender_key_ref.to_owned(),
        recipient_key_ref: recipient_key_ref.to_owned(),
        nonce,
        ciphertext: hex_encode(&sealed),
        auth_tag: hex_encode(&auth_tag),
    }
}

#[test]
fn integration_decrypt_rejects_invalid_ciphertext_hex() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
        let mut sealed = engine
            .encrypt("payload", 101)
            .expect("encrypt should succeed");
        sealed.ciphertext = "zz".to_owned();
        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::InvalidCiphertextEncoding)
        );
    });
}

#[test]
fn integration_decrypt_rejects_invalid_auth_tag_hex_as_integrity_failure() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
        let mut sealed = engine
            .encrypt("payload", 102)
            .expect("encrypt should succeed");
        sealed.auth_tag = "zz".to_owned();
        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::IntegrityCheckFailed)
        );
    });
}

#[test]
fn integration_decrypt_rejects_invalid_utf8_plaintext_output() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
        let sealed =
            ciphertext_with_raw_plaintext(SENDER_KEY_REF, RECIPIENT_KEY_REF, 103, &[0xff, 0xfe]);
        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::InvalidCiphertextEncoding)
        );
    });
}

#[test]
fn contract_encrypt_maps_aead_failure_to_encryption_failed() {
    assert!(
        ENGINE_SOURCE.contains(".map_err(|_| DirectMessageCryptoError::EncryptionFailed)?"),
        "encrypt path must preserve EncryptionFailed mapping"
    );
}

#[test]
fn contract_hkdf_failure_maps_to_key_derivation_failed() {
    assert!(
        KEY_AGREEMENT_SOURCE
            .contains(".map_err(|_| DirectMessageCryptoError::KeyDerivationFailed)"),
        "hkdf failure path must preserve KeyDerivationFailed mapping"
    );
}
