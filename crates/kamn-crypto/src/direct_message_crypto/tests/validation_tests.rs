use super::*;

#[test]
fn display_messages_remain_stable_for_reason_taxonomy() {
    assert_eq!(
        DirectMessageCryptoError::InvalidKeyRef("sender").to_string(),
        "sender key reference must include #key-agreement"
    );
    assert_eq!(
        DirectMessageCryptoError::KeyRefMismatch("recipient").to_string(),
        "recipient key reference mismatch"
    );
    assert_eq!(
        DirectMessageCryptoError::KeyDerivationFailed.to_string(),
        "direct message key derivation failed"
    );
}

#[test]
fn constructor_rejects_empty_sender_key_reference() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        assert_eq!(
            DirectMessageCryptoEngine::new("   ", "kamn:did:agent:bob#key-agreement-1"),
            Err(DirectMessageCryptoError::EmptyKeyRef("sender"))
        );
    });
}

#[test]
fn constructor_rejects_empty_recipient_key_reference() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        assert_eq!(
            DirectMessageCryptoEngine::new("kamn:did:agent:alice#key-agreement-1", ""),
            Err(DirectMessageCryptoError::EmptyKeyRef("recipient"))
        );
    });
}

#[test]
fn constructor_rejects_seed_hex_with_invalid_length() {
    with_key_agreement_seed(Some("abcd"), || {
        assert_eq!(
            DirectMessageCryptoEngine::new(
                "kamn:did:agent:alice#key-agreement-1",
                "kamn:did:agent:bob#key-agreement-1",
            ),
            Err(DirectMessageCryptoError::InvalidKeyAgreementMasterSeed)
        );
    });
}

#[test]
fn constructor_rejects_seed_hex_with_invalid_characters() {
    with_key_agreement_seed(
        Some("zz112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"),
        || {
            assert_eq!(
                DirectMessageCryptoEngine::new(
                    "kamn:did:agent:alice#key-agreement-1",
                    "kamn:did:agent:bob#key-agreement-1",
                ),
                Err(DirectMessageCryptoError::InvalidKeyAgreementMasterSeed)
            );
        },
    );
}

#[test]
fn encrypt_rejects_empty_payload() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        assert_eq!(
            engine.encrypt("", 1),
            Err(DirectMessageCryptoError::EmptyPayload)
        );
    });
}

#[test]
fn encrypt_rejects_zero_nonce() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        assert_eq!(
            engine.encrypt("payload", 0),
            Err(DirectMessageCryptoError::InvalidNonce(0))
        );
    });
}

#[test]
fn decrypt_rejects_zero_nonce_in_ciphertext() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        let mut sealed = engine.encrypt("payload", 22).expect("encrypt");
        sealed.nonce = 0;
        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::InvalidNonce(0))
        );
    });
}
