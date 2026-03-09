use kamn_crypto::direct_message_crypto::{DirectMessageCryptoEngine, DirectMessageCryptoError};
use std::sync::{Mutex, OnceLock};

const TEST_KEY_SEED_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const SENDER_KEY_REF: &str = "kamn:did:agent:alice#key-agreement-1";
const RECIPIENT_KEY_REF: &str = "kamn:did:agent:bob#key-agreement-1";
const EMPTY_PAYLOAD_NONCE: u64 = 41;
const ZERO_NONCE: u64 = 0;
const VALID_DECRYPT_NONCE: u64 = 42;
const REUSED_NONCE: u64 = 43;

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

fn engine() -> DirectMessageCryptoEngine {
    DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF)
        .expect("engine init should succeed")
}

fn assert_encrypt_error(plaintext: &str, nonce: u64, expected: DirectMessageCryptoError) {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = engine();
        assert_eq!(engine.encrypt(plaintext, nonce), Err(expected));
    });
}

fn encrypt_payload(
    engine: &mut DirectMessageCryptoEngine,
    nonce: u64,
) -> kamn_crypto::direct_message_crypto::DirectMessageCiphertext {
    engine.encrypt("payload", nonce).expect("encrypt")
}

#[test]
fn integration_encrypt_rejects_empty_plaintext_payload() {
    assert_encrypt_error(
        "",
        EMPTY_PAYLOAD_NONCE,
        DirectMessageCryptoError::EmptyPayload,
    );
}

#[test]
fn integration_encrypt_rejects_zero_nonce() {
    assert_encrypt_error(
        "payload",
        ZERO_NONCE,
        DirectMessageCryptoError::InvalidNonce(ZERO_NONCE),
    );
}

#[test]
fn integration_decrypt_rejects_zero_nonce_ciphertext() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = engine();
        let mut sealed = encrypt_payload(&mut engine, VALID_DECRYPT_NONCE);
        sealed.nonce = ZERO_NONCE;
        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::InvalidNonce(ZERO_NONCE))
        );
    });
}

#[test]
fn integration_encrypt_rejects_nonce_reuse() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = engine();
        engine
            .encrypt("payload", REUSED_NONCE)
            .expect("first encrypt");
        assert_eq!(
            engine.encrypt("payload", REUSED_NONCE),
            Err(DirectMessageCryptoError::NonceReuse(REUSED_NONCE))
        );
    });
}
