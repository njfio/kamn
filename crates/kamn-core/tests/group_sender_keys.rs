use kamn_core::{GroupChannelCryptoEngine, GroupChannelCryptoError};
use std::sync::{Mutex, OnceLock};

const KEY_AGREEMENT_MASTER_SEED_ENV: &str = "KAMN_KEY_AGREEMENT_MASTER_SEED_HEX";
const TEST_KEY_SEED_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn with_key_agreement_seed<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock");
    let previous = std::env::var(KEY_AGREEMENT_MASTER_SEED_ENV).ok();
    match value {
        Some(seed) => std::env::set_var(KEY_AGREEMENT_MASTER_SEED_ENV, seed),
        None => std::env::remove_var(KEY_AGREEMENT_MASTER_SEED_ENV),
    }
    let result = f();
    match previous {
        Some(seed) => std::env::set_var(KEY_AGREEMENT_MASTER_SEED_ENV, seed),
        None => std::env::remove_var(KEY_AGREEMENT_MASTER_SEED_ENV),
    }
    drop(guard);
    result
}

#[test]
fn sender_key_distribution_allows_authorized_group_round_trip() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:alpha").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec![
                    "kamn:did:agent:bob".to_owned(),
                    "kamn:did:agent:carol".to_owned(),
                ],
            )
            .expect("sender key distribution should succeed");

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "hello group", 7)
            .expect("encrypt should succeed");
        let plaintext = engine
            .decrypt("kamn:did:agent:bob", &sealed)
            .expect("decrypt should succeed");

        assert_eq!(plaintext, "hello group");
    });
}

#[test]
fn sender_key_rotation_advances_generation() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:beta").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("initial key distribution should succeed");

        let rotated = engine
            .rotate_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-2",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("key rotation should succeed");

        assert_eq!(rotated.key_generation, 2);

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "post-rotation", 9)
            .expect("encrypt should succeed");
        assert_eq!(sealed.key_generation, 2);
    });
}

#[test]
fn integration_multiple_senders_are_isolated() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:gamma").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("alice distribution should succeed");
        engine
            .distribute_sender_key(
                "kamn:did:agent:dave",
                "kamn:did:agent:dave#sender-key-1",
                vec!["kamn:did:agent:erin".to_owned()],
            )
            .expect("dave distribution should succeed");

        let alice_msg = engine
            .encrypt("kamn:did:agent:alice", "alice-msg", 11)
            .expect("alice encrypt should succeed");
        let dave_msg = engine
            .encrypt("kamn:did:agent:dave", "dave-msg", 13)
            .expect("dave encrypt should succeed");

        assert_eq!(
            engine
                .decrypt("kamn:did:agent:bob", &alice_msg)
                .expect("bob decrypts alice"),
            "alice-msg"
        );
        assert_eq!(
            engine
                .decrypt("kamn:did:agent:erin", &dave_msg)
                .expect("erin decrypts dave"),
            "dave-msg"
        );
    });
}

#[test]
fn regression_unauthorized_recipient_cannot_decrypt() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:delta").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "secret", 15)
            .expect("encrypt should succeed");

        assert_eq!(
            engine.decrypt("kamn:did:agent:mallory", &sealed),
            Err(GroupChannelCryptoError::RecipientNotAuthorized {
                recipient_did: "kamn:did:agent:mallory".to_owned(),
                sender_did: "kamn:did:agent:alice".to_owned(),
                key_generation: 1,
            })
        );
    });
}

#[test]
fn group_sender_keys_reject_invalid_sender_did_with_structured_marker() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:epsilon")
            .expect("engine should initialize");

        assert_eq!(
            engine.distribute_sender_key(
                "not-a-did",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            ),
            Err(GroupChannelCryptoError::InvalidDid {
                field: "sender_did",
                reason_code: "group_channel_crypto_invalid_sender_did",
                detail: "invalid agent did prefix: not-a-did".to_owned(),
            })
        );
    });
}

#[test]
fn group_sender_keys_debug_output_is_redacted_after_live_round_trip() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine =
            GroupChannelCryptoEngine::new("channel:group:zeta").expect("engine should initialize");

        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-secret",
                vec![
                    "kamn:did:agent:bob".to_owned(),
                    "kamn:did:agent:carol".to_owned(),
                ],
            )
            .expect("distribution should succeed");

        engine
            .encrypt("kamn:did:agent:alice", "redacted hello", 17)
            .expect("encrypt should succeed");

        let debug_output = format!("{engine:?}");

        assert!(
            debug_output.contains("GroupChannelCryptoEngine"),
            "debug output should identify the engine type: {debug_output}"
        );
        assert!(
            debug_output.contains("used_nonce_count: 1"),
            "debug output should expose the redacted nonce summary: {debug_output}"
        );
        assert!(
            !debug_output.contains("sender-key-secret"),
            "debug output must not expose sender key refs: {debug_output}"
        );
        assert!(
            !debug_output.contains("kamn:did:agent:bob"),
            "debug output must not expose recipient allowlists: {debug_output}"
        );
    });
}

#[test]
fn cached_group_seed_allows_repeated_operations_after_env_removal() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:cached")
            .expect("engine should initialize");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed");

        let first = engine
            .encrypt("kamn:did:agent:alice", "payload-a", 1)
            .expect("first encrypt should succeed");

        std::env::remove_var(KEY_AGREEMENT_MASTER_SEED_ENV);

        let second = engine
            .encrypt("kamn:did:agent:alice", "payload-b", 2)
            .expect("cached seed should allow second encrypt");
        assert_eq!(
            engine
                .decrypt("kamn:did:agent:bob", &first)
                .expect("cached seed should allow decrypt after env removal"),
            "payload-a"
        );
        assert_eq!(
            engine
                .decrypt("kamn:did:agent:bob", &second)
                .expect("cached seed should allow decrypt of later ciphertext"),
            "payload-b"
        );
    });
}

#[test]
fn constructor_stays_env_independent_until_first_group_operation() {
    with_key_agreement_seed(None, || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:late-seed")
            .expect("constructor should not require env seed");
        engine
            .distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                vec!["kamn:did:agent:bob".to_owned()],
            )
            .expect("distribution should succeed without env seed");

        std::env::set_var(KEY_AGREEMENT_MASTER_SEED_ENV, TEST_KEY_SEED_HEX);

        let sealed = engine
            .encrypt("kamn:did:agent:alice", "late-seed", 3)
            .expect("encrypt should succeed once env seed is set");
        assert_eq!(
            engine
                .decrypt("kamn:did:agent:bob", &sealed)
                .expect("decrypt should succeed once env seed is cached"),
            "late-seed"
        );
    });
}
