use kamn_core::{DirectMessageCryptoEngine, GroupChannelCryptoEngine, GroupChannelCryptoError};
use std::sync::{Mutex, OnceLock};

const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const TEST_KEY_SEED_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn with_key_agreement_master_seed<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock should not be poisoned");

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
fn spec_c01_issue_5921_direct_message_uses_canonical_algorithm_labels() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("direct-message engine should initialize without insecure fixture env");

        let sealed = engine
            .encrypt("hello secure world", 7)
            .expect("direct-message encrypt should succeed");

        assert_eq!(sealed.key_agreement_algorithm, "X25519");
        assert_eq!(sealed.cipher_algorithm, "XChaCha20-Poly1305");
    });
}

#[test]
fn spec_c02_issue_5921_group_channel_uses_canonical_algorithm_labels() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:alpha").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("sender key distribution should succeed");

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "hello group", 9)
            .expect("group encrypt should succeed");

        assert_eq!(sealed.key_derivation_algorithm, "X25519");
        assert_eq!(sealed.cipher_algorithm, "XChaCha20-Poly1305");
    });
}

#[test]
fn spec_c03_issue_5921_tampered_group_auth_tag_is_rejected() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:beta").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("sender key distribution should succeed");

        let mut sealed = engine
            .encrypt("kamn:did:agent:alice", "secret", 11)
            .expect("group encrypt should succeed");
        sealed.auth_tag = "00".repeat(16);

        match engine.decrypt("kamn:did:agent:bob", &sealed) {
            Err(GroupChannelCryptoError::IntegrityCheckFailed)
            | Err(GroupChannelCryptoError::SignatureMismatch) => {}
            other => panic!("expected tamper rejection, got {other:?}"),
        }
    });
}

#[test]
fn spec_c04_issue_5921_group_encryption_rejects_unauthorized_recipient() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:gamma").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("sender key distribution should succeed");

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "secret", 13)
            .expect("group encrypt should succeed");

        match engine.decrypt("kamn:did:agent:mallory", &sealed) {
            Err(GroupChannelCryptoError::RecipientNotAuthorized { .. }) => {}
            other => panic!("expected unauthorized recipient error, got {other:?}"),
        }
    });
}

#[test]
fn regression_issue_5921_seed_is_required_for_direct_message_engine() {
    with_key_agreement_master_seed(None, || {
        let result = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        );
        assert!(result.is_err());
    });
}

#[test]
fn regression_issue_5921_invalid_seed_is_rejected_for_direct_message_engine() {
    with_key_agreement_master_seed(Some("invalid-seed"), || {
        let result = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        );
        assert!(result.is_err());
    });
}

#[test]
fn regression_issue_5921_seed_is_required_for_group_message_encryption() {
    with_key_agreement_master_seed(None, || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:seed").expect("engine should initialize");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");
        let result = engine.encrypt("kamn:did:agent:alice", "payload", 71);
        assert!(result.is_err());
    });
}

#[test]
fn regression_issue_5921_invalid_seed_is_rejected_for_group_message_encryption() {
    with_key_agreement_master_seed(Some("invalid-seed"), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:seed-invalid")
            .expect("engine should initialize");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");
        let result = engine.encrypt("kamn:did:agent:alice", "payload", 73);
        assert!(result.is_err());
    });
}

#[test]
fn regression_issue_5921_direct_message_key_context_changes_ciphertext() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut alice_to_bob = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine should initialize");
        let mut alice_to_carol = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:carol#key-agreement-1",
        )
        .expect("engine should initialize");

        let sealed_bob = alice_to_bob
            .encrypt("same payload", 21)
            .expect("encrypt should succeed");
        let sealed_carol = alice_to_carol
            .encrypt("same payload", 21)
            .expect("encrypt should succeed");

        assert_ne!(sealed_bob.ciphertext, sealed_carol.ciphertext);
        assert_ne!(sealed_bob.auth_tag, sealed_carol.auth_tag);
    });
}

#[test]
fn regression_issue_5921_direct_message_nonce_changes_ciphertext() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine should initialize");

        let first = engine
            .encrypt("same payload", 31)
            .expect("encrypt should succeed");
        let second = engine
            .encrypt("same payload", 32)
            .expect("encrypt should succeed");

        assert_ne!(first.ciphertext, second.ciphertext);
        assert_ne!(first.auth_tag, second.auth_tag);
    });
}

#[test]
fn regression_issue_5921_direct_message_aad_binds_key_references() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = DirectMessageCryptoEngine::new(
            "kamn:did:agent:alice#key-agreement-1",
            "kamn:did:agent:bob#key-agreement-1",
        )
        .expect("engine should initialize");
        let mut sealed = engine
            .encrypt("payload", 41)
            .expect("encrypt should succeed");
        sealed.sender_key_ref = "kamn:did:agent:mallory#key-agreement-1".to_owned();

        let result = engine.decrypt(&sealed);
        assert!(result.is_err());
    });
}

#[test]
fn regression_issue_5921_group_nonce_and_signature_vary_per_message() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:nonce").expect("engine should initialize");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");

        let first = engine
            .encrypt("kamn:did:agent:alice", "same payload", 51)
            .expect("encrypt should succeed");
        let second = engine
            .encrypt("kamn:did:agent:alice", "same payload", 52)
            .expect("encrypt should succeed");

        assert_ne!(first.ciphertext, second.ciphertext);
        assert_ne!(first.auth_tag, second.auth_tag);
        assert_ne!(first.signature, second.signature);
    });
}

#[test]
fn regression_issue_5921_group_generation_changes_ciphertext() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:generation")
            .expect("engine should initialize");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");
        let first = engine
            .encrypt("kamn:did:agent:alice", "same payload", 61)
            .expect("encrypt should succeed");

        engine
            .rotate_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-2",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("rotation should succeed");
        let second = engine
            .encrypt("kamn:did:agent:alice", "same payload", 61)
            .expect("encrypt should succeed");

        assert_ne!(first.ciphertext, second.ciphertext);
        assert_ne!(first.auth_tag, second.auth_tag);
    });
}

#[test]
fn regression_issue_5921_group_channel_context_changes_ciphertext() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine_alpha = GroupChannelCryptoEngine::new("channel:group:alpha-context")
            .expect("engine should initialize");
        let mut engine_beta = GroupChannelCryptoEngine::new("channel:group:beta-context")
            .expect("engine should initialize");

        engine_alpha
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");
        engine_beta
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");

        let alpha = engine_alpha
            .encrypt("kamn:did:agent:alice", "same payload", 81)
            .expect("encrypt should succeed");
        let beta = engine_beta
            .encrypt("kamn:did:agent:alice", "same payload", 81)
            .expect("encrypt should succeed");

        assert_ne!(alpha.ciphertext, beta.ciphertext);
        assert_ne!(alpha.auth_tag, beta.auth_tag);
    });
}

#[test]
fn regression_issue_5921_group_sender_key_ref_changes_ciphertext() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine_key_1 = GroupChannelCryptoEngine::new("channel:group:keyref")
            .expect("engine should initialize");
        let mut engine_key_2 = GroupChannelCryptoEngine::new("channel:group:keyref")
            .expect("engine should initialize");

        engine_key_1
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");
        engine_key_2
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-9",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");

        let sealed_key_1 = engine_key_1
            .encrypt("kamn:did:agent:alice", "same payload", 91)
            .expect("encrypt should succeed");
        let sealed_key_2 = engine_key_2
            .encrypt("kamn:did:agent:alice", "same payload", 91)
            .expect("encrypt should succeed");

        assert_ne!(sealed_key_1.ciphertext, sealed_key_2.ciphertext);
        assert_ne!(sealed_key_1.auth_tag, sealed_key_2.auth_tag);
    });
}

#[test]
fn regression_issue_5921_group_auth_tag_is_fixed_poly1305_size() {
    with_key_agreement_master_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:auth-tag")
            .expect("engine should initialize");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "payload", 101)
            .expect("encrypt should succeed");
        assert_eq!(sealed.auth_tag.len(), 32);
    });
}
