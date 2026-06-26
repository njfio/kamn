use super::super::{
    derive_group_aead_key, derive_group_aead_key_legacy, group_nonce_bytes,
    GroupChannelCryptoEngine, GroupChannelCryptoError, GROUP_MESSAGE_HKDF_BACKEND_MARKER,
    GROUP_MESSAGE_HMAC_BACKEND_MARKER,
};
use super::support::{with_key_agreement_seed, PRODUCTION_SOURCE, SOURCE};

#[test]
fn regression_requires_constant_time_group_signature_compare() {
    assert!(PRODUCTION_SOURCE.contains("crate::constant_time_eq::constant_time_eq_str("));
    assert!(!SOURCE.contains(
        ["if expected_signature !=", " sealed.signature {"]
            .concat()
            .as_str()
    ));
}

#[test]
fn regression_constructor_accepts_without_insecure_fixture_opt_in() {
    assert!(GroupChannelCryptoEngine::new("channel:group:locked").is_ok());
}

#[test]
fn encrypt_requires_key_agreement_seed() {
    with_key_agreement_seed(None, || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:1")
            .expect("expected test fixture operation to succeed");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("expected test fixture operation to succeed");
        assert_eq!(
            engine.encrypt("kamn:did:agent:alice", "payload", 1),
            Err(GroupChannelCryptoError::MissingKeyAgreementMasterSeed)
        );
    });
}

#[test]
fn group_message_hkdf_derivation_is_deterministic_and_distinct_from_legacy_v1() {
    let shared_secret = [0x3cu8; 32];
    let hkdf_key_a = derive_group_aead_key(&shared_secret, "channel:test", 9)
        .expect("expected test fixture operation to succeed");
    let hkdf_key_b = derive_group_aead_key(&shared_secret, "channel:test", 9)
        .expect("expected test fixture operation to succeed");
    let legacy_key = derive_group_aead_key_legacy(&shared_secret, "channel:test", 9);
    assert_eq!(hkdf_key_a, hkdf_key_b);
    assert_ne!(hkdf_key_a, legacy_key);
}

#[test]
fn group_message_derivation_backend_markers_and_manual_helper_removal_contract() {
    assert_eq!(
        GROUP_MESSAGE_HKDF_BACKEND_MARKER,
        "rustcrypto.hkdf.sha256.v1"
    );
    assert_eq!(
        GROUP_MESSAGE_HMAC_BACKEND_MARKER,
        "rustcrypto.hmac.sha256.v1"
    );
    assert!(!SOURCE.contains("\nfn hkdf_sha256_derive_32("));
    assert!(!SOURCE.contains("\nfn hmac_sha256("));
}

#[test]
fn spec_c09_group_channel_engine_source_contract_enforces_non_clone_redacted_debug_and_drop_zeroize(
) {
    let production_source = PRODUCTION_SOURCE;
    let derive_line = production_source
        .split("pub struct GroupChannelCryptoEngine")
        .next()
        .and_then(|prefix| prefix.lines().last())
        .unwrap_or_default();
    let derive_window = derive_line.replace(' ', "");
    assert!(!derive_window.contains("Clone"));
    assert!(production_source.contains("impl fmt::Debug for GroupChannelCryptoEngine"));
    assert!(production_source.contains("used_nonce_count"));
    assert!(production_source.contains("impl Drop for GroupChannelCryptoEngine"));
    assert!(production_source.contains("self.channel_id.zeroize();"));
    assert!(production_source.contains("zeroize_sender_key_history(&mut self.sender_key_history);"));
    assert!(production_source.contains("seed_hex.zeroize();"));
}

#[test]
fn group_nonce_bytes_do_not_expose_raw_counter_prefix() {
    let nonce = 0x0102_0304_0506_0708_u64;
    assert_ne!(
        &group_nonce_bytes("kamn:did:agent:alice", 7, nonce)[..8],
        &nonce.to_le_bytes()
    );
}

#[test]
fn display_messages_remain_stable_for_reason_taxonomy() {
    assert_eq!(
        GroupChannelCryptoError::EmptyChannelId.to_string(),
        "channel_id must not be empty"
    );
}
