use kamn_crypto::direct_message_crypto::{
    DirectMessageCryptoEngine, DirectMessageCryptoError, DIRECT_MESSAGE_HKDF_BACKEND_MARKER,
    DIRECT_MESSAGE_HMAC_BACKEND_MARKER,
};
use std::sync::{Mutex, OnceLock};

const TEST_KEY_SEED_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";

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

#[test]
fn integration_encrypt_decrypt_roundtrip_contract() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        let sealed = engine
            .encrypt("integration-roundtrip", 91)
            .expect("encrypt");
        let plaintext = engine.decrypt(&sealed).expect("decrypt");
        assert_eq!(plaintext, "integration-roundtrip");
    });
}

#[test]
fn integration_decrypt_rejects_sender_key_ref_mismatch() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        let mut sealed = engine.encrypt("integration-mismatch", 92).expect("encrypt");
        sealed.sender_key_ref = "kamn:did:agent:mallory#key-agreement-1".to_owned();

        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::KeyRefMismatch("sender"))
        );
    });
}

#[test]
fn integration_decrypt_rejects_recipient_key_ref_mismatch() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        let mut sealed = engine.encrypt("integration-mismatch", 93).expect("encrypt");
        sealed.recipient_key_ref = "kamn:did:agent:eve#key-agreement-1".to_owned();

        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::KeyRefMismatch("recipient"))
        );
    });
}

#[test]
fn integration_derivation_backend_markers_are_exposed_via_public_api() {
    assert_eq!(
        DIRECT_MESSAGE_HKDF_BACKEND_MARKER,
        "rustcrypto.hkdf.sha256.v1"
    );
    assert_eq!(
        DIRECT_MESSAGE_HMAC_BACKEND_MARKER,
        "rustcrypto.hmac.sha256.v1"
    );
}
