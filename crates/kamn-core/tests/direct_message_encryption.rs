use kamn_core::{DirectMessageCryptoEngine, DirectMessageCryptoError};
use std::sync::OnceLock;

const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";

fn ensure_key_agreement_master_seed() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::set_var(
            KEY_AGREEMENT_MASTER_SEED_ENV,
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        );
    });
}

#[test]
fn encrypt_decrypt_round_trip_preserves_plaintext() {
    ensure_key_agreement_master_seed();
    let mut engine = DirectMessageCryptoEngine::new(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
    )
    .expect("engine should initialize");

    let sealed = engine
        .encrypt("hello secure world", 7)
        .expect("encrypt should succeed");
    let plaintext = engine.decrypt(&sealed).expect("decrypt should succeed");

    assert_eq!(plaintext, "hello secure world");
}

#[test]
fn tampered_ciphertext_fails_integrity_check() {
    ensure_key_agreement_master_seed();
    let mut engine = DirectMessageCryptoEngine::new(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
    )
    .expect("engine should initialize");
    let mut sealed = engine
        .encrypt("payload", 9)
        .expect("encrypt should succeed");
    sealed.ciphertext = "00".to_owned();

    assert_eq!(
        engine.decrypt(&sealed),
        Err(DirectMessageCryptoError::IntegrityCheckFailed)
    );
}

#[test]
fn nonce_reuse_is_rejected() {
    ensure_key_agreement_master_seed();
    let mut engine = DirectMessageCryptoEngine::new(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
    )
    .expect("engine should initialize");
    engine
        .encrypt("first", 3)
        .expect("first encrypt should succeed");

    assert_eq!(
        engine.encrypt("second", 3),
        Err(DirectMessageCryptoError::NonceReuse(3))
    );
}

#[test]
fn empty_payload_is_rejected() {
    ensure_key_agreement_master_seed();
    let mut engine = DirectMessageCryptoEngine::new(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
    )
    .expect("engine should initialize");
    assert_eq!(
        engine.encrypt("", 5),
        Err(DirectMessageCryptoError::EmptyPayload)
    );
}

#[test]
fn tampered_auth_tag_is_rejected() {
    ensure_key_agreement_master_seed();
    let mut engine = DirectMessageCryptoEngine::new(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
    )
    .expect("engine should initialize");
    let mut sealed = engine
        .encrypt("payload", 11)
        .expect("encrypt should succeed");
    sealed.auth_tag = "deadbeef".to_owned();

    // Regression: #125
    assert_eq!(
        engine.decrypt(&sealed),
        Err(DirectMessageCryptoError::IntegrityCheckFailed)
    );
}
