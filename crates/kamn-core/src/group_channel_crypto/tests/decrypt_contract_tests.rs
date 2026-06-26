use super::super::{
    derive_group_aead_key, derive_group_shared_secret, GroupChannelCryptoEngine,
    GroupChannelCryptoError,
};
use super::support::{
    decode_ciphertext_parts, legacy_raw_nonce, legacy_v1_ciphertext, with_key_agreement_seed,
    TEST_KEY_SEED_HEX,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

#[test]
fn decrypt_rejects_unauthorized_recipient() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:1").unwrap();
        let distribution = engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .unwrap();
        let sealed = engine
            .encrypt("kamn:did:agent:alice", "group payload", 35)
            .unwrap();
        assert_eq!(
            engine.decrypt("kamn:did:agent:charlie", &sealed),
            Err(GroupChannelCryptoError::RecipientNotAuthorized {
                recipient_did: "kamn:did:agent:charlie".to_owned(),
                sender_did: "kamn:did:agent:alice".to_owned(),
                key_generation: distribution.key_generation,
            })
        );
    });
}

#[test]
fn decrypt_rejects_tampered_signature() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:1").unwrap();
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .unwrap();
        let mut sealed = engine
            .encrypt("kamn:did:agent:alice", "group payload", 37)
            .unwrap();
        let replacement = if sealed.signature.starts_with('0') {
            '1'
        } else {
            '0'
        };
        sealed
            .signature
            .replace_range(0..1, &replacement.to_string());
        let decrypted = engine.decrypt("kamn:did:agent:bob", &sealed);
        assert!(matches!(
            decrypted,
            Err(GroupChannelCryptoError::SignatureMismatch)
                | Err(GroupChannelCryptoError::MissingKeyAgreementMasterSeed)
        ));
    });
}

#[test]
fn decrypt_rejects_zero_nonce_fail_closed() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:1").unwrap();
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .unwrap();
        let mut sealed = engine
            .encrypt("kamn:did:agent:alice", "group payload", 39)
            .unwrap();
        sealed.nonce = 0;
        assert_eq!(
            engine.decrypt("kamn:did:agent:bob", &sealed),
            Err(GroupChannelCryptoError::InvalidNonce(0))
        );
    });
}

#[test]
fn decrypt_accepts_legacy_v1_sha256_kdf_ciphertext_for_compatibility() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let channel_id = "channel:group:legacy";
        let sender_did = "kamn:did:agent:alice";
        let sender_key_ref = "kamn:did:agent:alice#sender-key-1";
        let recipient_did = "kamn:did:agent:bob";
        let mut engine = GroupChannelCryptoEngine::new(channel_id).unwrap();
        let distribution = engine
            .distribute_sender_key(sender_did, sender_key_ref, vec![recipient_did.to_owned()])
            .unwrap();
        let sealed = legacy_v1_ciphertext(
            channel_id,
            sender_did,
            sender_key_ref,
            distribution.key_generation,
            57,
            "legacy-group-v1",
        );
        assert_eq!(
            engine.decrypt(recipient_did, &sealed).unwrap(),
            "legacy-group-v1"
        );
    });
}

#[test]
fn encrypt_output_does_not_authenticate_under_legacy_raw_prefix_nonce_layout() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let channel_id = "channel:group:nonce-layout";
        let sender_did = "kamn:did:agent:alice";
        let sender_key_ref = "kamn:did:agent:alice#sender-key-1";
        let recipient_did = "kamn:did:agent:bob";
        let nonce = 57;
        let mut engine = GroupChannelCryptoEngine::new(channel_id).unwrap();
        let distribution = engine
            .distribute_sender_key(sender_did, sender_key_ref, vec![recipient_did.to_owned()])
            .unwrap();
        let sealed = engine
            .encrypt(sender_did, "group-nonce-layout", nonce)
            .unwrap();
        let master_seed = super::super::load_key_agreement_master_seed().unwrap();
        let shared_secret = derive_group_shared_secret(
            channel_id,
            sender_key_ref,
            distribution.key_generation,
            &master_seed,
        );
        let aead_key =
            derive_group_aead_key(&shared_secret, channel_id, distribution.key_generation).unwrap();
        let cipher = XChaCha20Poly1305::new((&aead_key).into());
        let xnonce = XNonce::from(legacy_raw_nonce(
            sender_did,
            distribution.key_generation,
            nonce,
        ));
        let decrypted = cipher.decrypt(
            &xnonce,
            Payload {
                msg: &decode_ciphertext_parts(&sealed),
                aad: &[],
            },
        );
        assert!(decrypted.is_err());
    });
}
