use super::super::{GroupChannelCryptoEngine, GroupChannelCryptoError};
use super::support::{legacy_v1_ciphertext, with_key_agreement_seed, TEST_KEY_SEED_HEX};

#[test]
fn constructor_rejects_empty_channel_id() {
    assert_eq!(
        GroupChannelCryptoEngine::new(""),
        Err(GroupChannelCryptoError::EmptyChannelId)
    );
}

#[test]
fn distribution_rejects_empty_recipients() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let mut engine = GroupChannelCryptoEngine::new("channel:group:1")
            .expect("expected test fixture operation to succeed");
        assert_eq!(
            engine.distribute_sender_key(
                "kamn:did:agent:alice",
                "kamn:did:agent:alice#sender-key-1",
                Vec::new(),
            ),
            Err(GroupChannelCryptoError::EmptyRecipients)
        );
    });
}

#[test]
fn rotate_marks_previous_generation_inactive() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let (engine, first, second) = rotated_generations();
        assert_generation_activity(&engine, first.key_generation, false);
        assert_generation_activity(&engine, second.key_generation, true);
    });
}

#[test]
fn encrypt_decrypt_roundtrip_requires_authorized_recipient() {
    with_key_agreement_seed(Some(TEST_KEY_SEED_HEX), || {
        let (mut engine, distribution) = engine_with_rotated_sender_key();
        assert_sender_key_rotation_state(&engine, &distribution);
        let sealed = engine
            .encrypt("kamn:did:agent:alice", "group payload", 33)
            .expect("expected test fixture operation to succeed");
        assert_active_generation_decrypts(&engine, &sealed);
        assert_debug_output_redacts_sender_key_ref(&engine);
        let legacy = legacy_v1_ciphertext(
            "channel:group:1",
            "kamn:did:agent:alice",
            "kamn:did:agent:alice#sender-key-2",
            2,
            34,
            "legacy payload",
        );
        assert_eq!(
            engine
                .decrypt("kamn:did:agent:bob", &legacy)
                .expect("expected test fixture operation to succeed"),
            "legacy payload"
        );
    });
}

fn engine_with_rotated_sender_key() -> (
    GroupChannelCryptoEngine,
    super::super::SenderKeyDistributionRecord,
) {
    let mut engine = GroupChannelCryptoEngine::new("channel:group:1")
        .expect("expected test fixture operation to succeed");
    let distribution = engine
        .distribute_sender_key(
            "kamn:did:agent:alice",
            "kamn:did:agent:alice#sender-key-1",
            vec!["kamn:did:agent:bob".to_owned()],
        )
        .expect("expected test fixture operation to succeed");
    engine
        .rotate_sender_key(
            "kamn:did:agent:alice",
            "kamn:did:agent:alice#sender-key-2",
            vec!["kamn:did:agent:bob".to_owned()],
        )
        .expect("expected test fixture operation to succeed");
    (engine, distribution)
}

fn assert_sender_key_rotation_state(
    engine: &GroupChannelCryptoEngine,
    distribution: &super::super::SenderKeyDistributionRecord,
) {
    assert_eq!(
        engine
            .active_sender_key_generation("kamn:did:agent:alice")
            .expect("expected test fixture operation to succeed"),
        2
    );
    assert_eq!(
        engine
            .sender_key_record("kamn:did:agent:alice", distribution.key_generation)
            .expect("expected test fixture operation to succeed")
            .sender_key_ref,
        "kamn:did:agent:alice#sender-key-1"
    );
}

fn assert_active_generation_decrypts(
    engine: &GroupChannelCryptoEngine,
    sealed: &super::super::GroupMessageCiphertext,
) {
    assert_eq!(
        engine
            .decrypt("kamn:did:agent:bob", sealed)
            .expect("expected test fixture operation to succeed"),
        "group payload"
    );
    assert_eq!(sealed.key_generation, 2);
}

fn assert_debug_output_redacts_sender_key_ref(engine: &GroupChannelCryptoEngine) {
    let debug_output = format!("{engine:?}");
    assert!(debug_output.contains("used_nonce_count: 1"));
    assert!(!debug_output.contains("sender-key-2"));
}

fn rotated_generations() -> (
    GroupChannelCryptoEngine,
    super::super::SenderKeyDistributionRecord,
    super::super::SenderKeyDistributionRecord,
) {
    let mut engine = GroupChannelCryptoEngine::new("channel:group:1")
        .expect("expected test fixture operation to succeed");
    let first = engine
        .distribute_sender_key(
            "kamn:did:agent:alice",
            "kamn:did:agent:alice#sender-key-1",
            vec!["kamn:did:agent:bob".to_owned()],
        )
        .expect("expected test fixture operation to succeed");
    let second = engine
        .rotate_sender_key(
            "kamn:did:agent:alice",
            "kamn:did:agent:alice#sender-key-2",
            vec!["kamn:did:agent:bob".to_owned()],
        )
        .expect("expected test fixture operation to succeed");
    (engine, first, second)
}

fn assert_generation_activity(
    engine: &GroupChannelCryptoEngine,
    generation: u64,
    expected_active: bool,
) {
    assert_eq!(
        engine
            .sender_key_record("kamn:did:agent:alice", generation)
            .expect("expected test fixture operation to succeed")
            .active,
        expected_active
    );
}
