use kamn_core::{
    CanonicalMessageEnvelope, DataLayerM0AppendOnlyLedger, DataLayerM0Error,
    DataLayerM0RecordInput, DataLayerM0WrappedKey, DirectMessageCryptoEngine, EnvelopeEncryption,
    EnvelopeHeader, EnvelopeMetadata, EnvelopeProof, CANONICAL_ENCRYPTION_ALGORITHM,
    CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE, DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD,
};
use std::collections::BTreeMap;

fn valid_envelope(message_id: &str, recipients: Vec<&str>) -> CanonicalMessageEnvelope {
    let mut body = BTreeMap::new();
    body.insert("content".to_owned(), "hello".to_owned());

    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: message_id.to_owned(),
            type_name: CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: recipients.into_iter().map(str::to_owned).collect(),
            created: "2026-02-18T00:00:00Z".to_owned(),
            expires: "2026-02-18T01:00:00Z".to_owned(),
            thread_id: Some("thread-1".to_owned()),
            parent_id: None,
            nonce: 1,
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "normal".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
                recipient_keys: vec![
                    "did:key:z6MkrecipientA#key-agreement-1".to_owned(),
                    "did:key:z6MkrecipientB#key-agreement-1".to_owned(),
                ],
            },
        },
        body,
        attachments: Vec::new(),
        proof: EnvelopeProof {
            type_name: "DataIntegrityProof".to_owned(),
            created: "2026-02-18T00:00:00Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#key-1".to_owned(),
            proof_purpose: CANONICAL_PROOF_PURPOSE.to_owned(),
            proof_value: "sig:valid".to_owned(),
        },
    }
}

fn valid_input(
    message_id: &str,
    recipients: Vec<&str>,
    wrapped_keys: Vec<(&str, &str)>,
) -> DataLayerM0RecordInput {
    let envelope = valid_envelope(message_id, recipients);
    let mut crypto = DirectMessageCryptoEngine::new(
        "did:key:z6Mksender#key-agreement-1",
        "did:key:z6Mkrecipient#key-agreement-1",
    )
    .expect("crypto engine should initialize");
    let ciphertext = crypto
        .encrypt("hello world", 7)
        .expect("ciphertext should encrypt");

    DataLayerM0RecordInput {
        envelope,
        ciphertext,
        wrapped_keys: wrapped_keys
            .into_iter()
            .map(|(did, wrapped_cek)| DataLayerM0WrappedKey {
                did: did.to_owned(),
                wrapped_cek: wrapped_cek.to_owned(),
            })
            .collect(),
        compression_codec: DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD.to_owned(),
        compression_dict_id: Some(1),
        content_size_bytes: 256,
        compressed_size_bytes: 128,
    }
}

#[test]
fn spec_c01_record_hashes_are_deterministic_across_orderings() {
    let mut first_ledger = DataLayerM0AppendOnlyLedger::new();
    let mut second_ledger = DataLayerM0AppendOnlyLedger::new();

    let first = first_ledger
        .append(valid_input(
            "msg-c01",
            vec!["kamn:did:agent:recipient-b", "kamn:did:agent:recipient-a"],
            vec![
                ("kamn:did:owner:recipient-b", "wrap-b"),
                ("kamn:did:owner:recipient-a", "wrap-a"),
            ],
        ))
        .expect("first append should succeed");

    let second = second_ledger
        .append(valid_input(
            "msg-c01",
            vec!["kamn:did:agent:recipient-a", "kamn:did:agent:recipient-b"],
            vec![
                ("kamn:did:owner:recipient-a", "wrap-a"),
                ("kamn:did:owner:recipient-b", "wrap-b"),
            ],
        ))
        .expect("second append should succeed");

    assert_eq!(first.content_hash, second.content_hash);
    assert_eq!(first.envelope_aad_hash, second.envelope_aad_hash);
}

#[test]
fn spec_c02_append_rejects_duplicate_message_id() {
    let mut ledger = DataLayerM0AppendOnlyLedger::new();
    ledger
        .append(valid_input(
            "msg-c02",
            vec!["kamn:did:agent:recipient-a"],
            vec![("kamn:did:owner:recipient-a", "wrap-a")],
        ))
        .expect("first append should succeed");

    let duplicate = ledger.append(valid_input(
        "msg-c02",
        vec!["kamn:did:agent:recipient-a"],
        vec![("kamn:did:owner:recipient-a", "wrap-a")],
    ));
    assert_eq!(
        duplicate,
        Err(DataLayerM0Error::DuplicateMessageId("msg-c02".to_owned()))
    );
}

#[test]
fn spec_c03_hash_chain_verification_detects_tampering() {
    let mut ledger = DataLayerM0AppendOnlyLedger::new();
    ledger
        .append(valid_input(
            "msg-c03-a",
            vec!["kamn:did:agent:recipient-a"],
            vec![("kamn:did:owner:recipient-a", "wrap-a")],
        ))
        .expect("first append should succeed");
    ledger
        .append(valid_input(
            "msg-c03-b",
            vec!["kamn:did:agent:recipient-b"],
            vec![("kamn:did:owner:recipient-b", "wrap-b")],
        ))
        .expect("second append should succeed");

    ledger
        .verify_hash_chain()
        .expect("untampered chain should verify");
    ledger
        .replace_content_hash_unchecked("msg-c03-a", "sha256:tampered")
        .expect("test tamper helper should succeed");

    let verify = ledger.verify_hash_chain();
    assert!(matches!(
        verify,
        Err(DataLayerM0Error::InvalidHashChainLink { .. })
    ));
}

#[test]
fn spec_c04_invalid_compression_metadata_is_rejected() {
    let mut ledger = DataLayerM0AppendOnlyLedger::new();
    let mut input = valid_input(
        "msg-c04",
        vec!["kamn:did:agent:recipient-a"],
        vec![("kamn:did:owner:recipient-a", "wrap-a")],
    );
    input.compression_codec = "lz4".to_owned();

    let error = ledger.append(input);
    assert_eq!(
        error,
        Err(DataLayerM0Error::InvalidCompressionCodec("lz4".to_owned()))
    );
}
