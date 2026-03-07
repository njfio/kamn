#[path = "support/direct_message_crypto_support.rs"]
mod support;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use kamn_crypto::direct_message_crypto::{
    DirectMessageCryptoEngine, DirectMessageCryptoError, DIRECT_MESSAGE_HKDF_BACKEND_MARKER,
    DIRECT_MESSAGE_HMAC_BACKEND_MARKER,
};
use support::{
    RECIPIENT_KEY_REF, SENDER_KEY_REF, TEST_KEY_SEED_HEX, canonical_aad, derive_aead_key,
    derive_shared_secret, hex_decode, legacy_raw_prefix_nonce_bytes_v1,
    legacy_v2_raw_prefix_ciphertext, with_key_agreement_seed,
};

#[test]
fn integration_encrypt_decrypt_roundtrip_contract() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
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
        let mut engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
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
        let mut engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
        let mut sealed = engine.encrypt("integration-mismatch", 93).expect("encrypt");
        sealed.recipient_key_ref = "kamn:did:agent:eve#key-agreement-1".to_owned();

        assert_eq!(
            engine.decrypt(&sealed),
            Err(DirectMessageCryptoError::KeyRefMismatch("recipient"))
        );
    });
}

#[test]
fn integration_encrypt_output_no_longer_authenticates_under_legacy_raw_prefix_nonce_layout() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let nonce = 94;
        let shared_secret = derive_shared_secret(SENDER_KEY_REF, RECIPIENT_KEY_REF);
        let key = derive_aead_key(&shared_secret);
        let mut engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
        let sealed = engine.encrypt("integration-nonce-layout", nonce).expect("encrypt");

        let mut combined = hex_decode(sealed.ciphertext.as_str());
        combined.extend_from_slice(&hex_decode(sealed.auth_tag.as_str()));
        let aad = canonical_aad(SENDER_KEY_REF, RECIPIENT_KEY_REF, nonce);
        let xnonce = XNonce::from(legacy_raw_prefix_nonce_bytes_v1(nonce));
        let legacy_attempt = XChaCha20Poly1305::new((&key).into()).decrypt(
            &xnonce,
            Payload {
                msg: &combined,
                aad: aad.as_bytes(),
            },
        );

        assert!(
            legacy_attempt.is_err(),
            "current encryptions must not authenticate under the legacy raw-prefix nonce layout"
        );
    });
}

#[test]
fn integration_decrypt_accepts_legacy_raw_prefix_nonce_layout_ciphertext() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let engine =
            DirectMessageCryptoEngine::new(SENDER_KEY_REF, RECIPIENT_KEY_REF).expect("engine");
        let sealed = legacy_v2_raw_prefix_ciphertext(
            SENDER_KEY_REF,
            RECIPIENT_KEY_REF,
            "legacy-raw-prefix",
            95,
        );

        let plaintext = engine.decrypt(&sealed).expect("decrypt");
        assert_eq!(plaintext, "legacy-raw-prefix");
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
