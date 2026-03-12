use super::*;

#[test]
fn constructor_rejects_invalid_key_reference() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        assert_eq!(
            DirectMessageCryptoEngine::new("did:alice#keys-1", "did:bob#key-agreement-1"),
            Err(DirectMessageCryptoError::InvalidKeyRef("sender"))
        );
    });
}

#[test]
fn decrypt_rejects_algorithm_mismatch() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "did:alice#key-agreement-1",
            "did:bob#key-agreement-1",
        )
        .expect("engine init failed");
        let mut sealed = engine.encrypt("payload", 1).expect("encrypt failed");
        sealed.cipher_algorithm = "AES-GCM".to_owned();

        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::AlgorithmMismatch)
        );
    });
}

#[test]
fn encrypt_decrypt_roundtrip_succeeds_for_valid_payload() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init failed");
        let sealed = engine.encrypt("hello-secure-world", 7).expect("encrypt failed");
        let plaintext = engine.decrypt(&sealed).expect("decrypt failed");
        assert_eq!(plaintext, "hello-secure-world");
    });
}

#[test]
fn encrypt_rejects_nonce_reuse_for_same_engine_instance() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init failed");
        engine.encrypt("payload", 11).expect("initial encrypt failed unexpectedly");

        assert_eq!(
            engine.encrypt("payload-2", 11),
            Err(DirectMessageCryptoError::NonceReuse(11))
        );
    });
}

#[test]
fn decrypt_rejects_tampered_ciphertext_with_integrity_error() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init failed");
        let mut sealed = engine.encrypt("payload", 13).expect("encrypt failed");
        sealed.ciphertext.replace_range(..1, "f");

        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::IntegrityCheckFailed)
        );
    });
}

#[test]
fn regression_constructor_accepts_without_insecure_fixture_opt_in() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let result = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        );
        assert!(result.is_ok());
    });
}

#[test]
fn constructor_requires_key_agreement_seed() {
    with_key_agreement_seed(None, || {
        let result = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        );
        assert_eq!(
            result,
            Err(DirectMessageCryptoError::MissingKeyAgreementMasterSeed)
        );
    });
}
