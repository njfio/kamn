use super::*;

#[test]
fn direct_message_hkdf_derivation_is_deterministic_and_distinct_from_legacy_v1() {
    let shared_secret = [0x5au8; 32];
    let hkdf_key_a = derive_direct_message_aead_key(&shared_secret).expect("hkdf key should derive");
    let hkdf_key_b = derive_direct_message_aead_key(&shared_secret).expect("hkdf key should derive");
    let legacy_key = derive_direct_message_aead_key_legacy(&shared_secret);

    assert_eq!(hkdf_key_a, hkdf_key_b);
    assert_ne!(hkdf_key_a, legacy_key);
}

#[test]
fn direct_message_derivation_backend_markers_and_manual_helper_removal_contract() {
    assert_eq!(DIRECT_MESSAGE_HKDF_BACKEND_MARKER, "rustcrypto.hkdf.sha256.v1");
    assert_eq!(DIRECT_MESSAGE_HMAC_BACKEND_MARKER, "rustcrypto.hmac.sha256.v1");
    assert!(!SOURCE.contains("\nfn hkdf_sha256_derive_32("), "manual hkdf helper must be removed");
    assert!(!SOURCE.contains("\nfn hmac_sha256("), "manual hmac helper must be removed");
}

#[test]
fn spec_c09_direct_message_engine_source_contract_enforces_non_clone_redacted_debug_and_drop_zeroize() {
    let engine_source = include_str!("../engine.rs");
    let struct_marker = "pub struct DirectMessageCryptoEngine";
    let struct_index = engine_source
        .find(struct_marker)
        .expect("engine struct declaration should exist");
    let derive_window_start = struct_index.saturating_sub(160);
    let derive_window = &engine_source[derive_window_start..struct_index];
    assert!(!derive_window.contains("Clone"), "direct-message engine must not derive Clone");
    assert!(
        engine_source.contains("impl fmt::Debug for DirectMessageCryptoEngine"),
        "direct-message engine must define custom Debug"
    );
    assert!(
        engine_source.contains("impl Drop for DirectMessageCryptoEngine"),
        "direct-message engine must define Drop"
    );
    assert!(
        engine_source.contains("self.aead_key.zeroize();"),
        "direct-message engine Drop must zeroize aead_key"
    );
    assert!(
        engine_source.contains("self.legacy_aead_key.zeroize();"),
        "direct-message engine Drop must zeroize legacy_aead_key"
    );
    assert!(
        include_str!("../key_agreement.rs").contains("seed_hex.zeroize();"),
        "direct-message master seed loader must zeroize env-loaded hex buffer"
    );
}

#[test]
fn spec_c10_direct_message_engine_debug_output_redacts_sensitive_key_material() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine init should succeed");
        let debug_output = format!("{engine:?}");
        assert!(debug_output.contains("sender_key_ref"));
        assert!(debug_output.contains("recipient_key_ref"));
        assert!(!debug_output.contains("aead_key"));
        assert!(!debug_output.contains("legacy_aead_key"));
    });
}
