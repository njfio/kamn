use super::*;

#[test]
fn decrypt_accepts_legacy_v1_sha256_kdf_ciphertext_for_compatibility() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let sender_key_ref = "kamn:did:agent:alice#key-agreement-1";
        let recipient_key_ref = "kamn:did:agent:bob#key-agreement-1";

        let engine = DirectMessageCryptoEngine::new(sender_key_ref, recipient_key_ref)
            .expect("engine init should succeed");
        let sealed = legacy_v1_ciphertext(sender_key_ref, recipient_key_ref, "legacy-v1", 41);

        let plaintext = engine
            .decrypt(&sealed)
            .expect("legacy-v1 decrypt must succeed");
        assert_eq!(plaintext, "legacy-v1");
    });
}

#[test]
fn decrypt_accepts_legacy_raw_prefix_nonce_layout_for_hkdf_v2_compatibility() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let sender_key_ref = "kamn:did:agent:alice#key-agreement-1";
        let recipient_key_ref = "kamn:did:agent:bob#key-agreement-1";

        let engine = DirectMessageCryptoEngine::new(sender_key_ref, recipient_key_ref)
            .expect("engine init should succeed");
        let sealed = legacy_v2_raw_prefix_ciphertext(
            sender_key_ref,
            recipient_key_ref,
            "legacy-raw-prefix",
            43,
        );

        let plaintext = engine
            .decrypt(&sealed)
            .expect("legacy raw-prefix decrypt must succeed");
        assert_eq!(plaintext, "legacy-raw-prefix");
    });
}

#[test]
fn hex_decode_rejects_odd_length_inputs() {
    assert_eq!(
        hex_decode("abc"),
        Err(DirectMessageCryptoError::InvalidCiphertextEncoding)
    );
}

#[test]
fn canonical_direct_message_aad_contains_expected_fields() {
    let aad = canonical_direct_message_aad(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
        99,
    );
    assert_eq!(
        aad,
        "X25519|XChaCha20-Poly1305|kamn:did:agent:alice#key-agreement-1|kamn:did:agent:bob#key-agreement-1|99"
    );
}

#[test]
fn direct_message_nonce_bytes_are_deterministic_and_nonce_sensitive() {
    let sender_key_ref = "kamn:did:agent:alice#key-agreement-1";
    let recipient_key_ref = "kamn:did:agent:bob#key-agreement-1";
    let nonce_7_first = direct_message_nonce_bytes(sender_key_ref, recipient_key_ref, 7);
    let nonce_7_second = direct_message_nonce_bytes(sender_key_ref, recipient_key_ref, 7);
    let nonce_8 = direct_message_nonce_bytes(sender_key_ref, recipient_key_ref, 8);

    assert_eq!(nonce_7_first, nonce_7_second);
    assert_ne!(nonce_7_first, nonce_8);
}

#[test]
fn direct_message_nonce_bytes_do_not_expose_raw_counter_prefix() {
    let nonce = 0x0102_0304_0506_0708_u64;
    let nonce_bytes = direct_message_nonce_bytes(
        "kamn:did:agent:alice#key-agreement-1",
        "kamn:did:agent:bob#key-agreement-1",
        nonce,
    );

    assert_ne!(&nonce_bytes[..8], &nonce.to_le_bytes());
}

#[test]
fn encrypt_output_does_not_authenticate_under_legacy_raw_prefix_nonce_layout() {
    assert_new_encryptions_reject_legacy_raw_prefix_nonce_layout();
}

fn assert_new_encryptions_reject_legacy_raw_prefix_nonce_layout() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let sender_key_ref = "kamn:did:agent:alice#key-agreement-1";
        let recipient_key_ref = "kamn:did:agent:bob#key-agreement-1";
        let nonce = 57;
        let mut engine = DirectMessageCryptoEngine::new(sender_key_ref, recipient_key_ref)
            .expect("engine init should succeed");
        let sealed = engine
            .encrypt("payload", nonce)
            .expect("encrypt should succeed");
        let combined = combined_ciphertext(&sealed);
        let aad = canonical_direct_message_aad(sender_key_ref, recipient_key_ref, nonce);
        let legacy_result = decrypt_with_legacy_raw_prefix_nonce(&engine, nonce, &combined, &aad);

        assert!(legacy_result.is_err());
    });
}

fn combined_ciphertext(sealed: &DirectMessageCiphertext) -> Vec<u8> {
    let ciphertext = hex_decode(sealed.ciphertext.as_str()).expect("ciphertext hex");
    let auth_tag = hex_decode(sealed.auth_tag.as_str()).expect("auth tag hex");
    let mut combined = ciphertext;
    combined.extend_from_slice(&auth_tag);
    combined
}

fn decrypt_with_legacy_raw_prefix_nonce(
    engine: &DirectMessageCryptoEngine,
    nonce: u64,
    combined: &[u8],
    aad: &str,
) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    let xnonce = XNonce::from(legacy_direct_message_nonce_bytes_raw_prefix_v1(nonce));
    XChaCha20Poly1305::new((&engine.aead_key).into()).decrypt(
        &xnonce,
        Payload {
            msg: combined,
            aad: aad.as_bytes(),
        },
    )
}
