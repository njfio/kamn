use kamn_core::{
    evaluate_data_layer_m0_conformance_matrix, CanonicalMessageEnvelope,
    DataLayerM0AppendOnlyLedger, DataLayerM0ConformanceInvariant, DataLayerM0ConformanceMatrixCase,
    DataLayerM0ConformanceMatrixDecision, DataLayerM0Error, DataLayerM0RecordInput,
    DataLayerM0WrappedKey, DirectMessageCryptoEngine, EnvelopeEncryption, EnvelopeHeader,
    EnvelopeMetadata, EnvelopeProof, CANONICAL_ENCRYPTION_ALGORITHM,
    CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE, DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD,
    DATA_LAYER_M0_CONFORMANCE_MATRIX_DRIFT_REASON_CODE,
    DATA_LAYER_M0_CONFORMANCE_MATRIX_STABLE_REASON_CODE,
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

#[test]
fn spec_c05_m0_conformance_matrix_reports_stable_for_foundation_invariants() {
    let mut determinism_ledger_a = DataLayerM0AppendOnlyLedger::new();
    let determinism_a = determinism_ledger_a
        .append(valid_input(
            "msg-c05-det",
            vec!["kamn:did:agent:recipient-b", "kamn:did:agent:recipient-a"],
            vec![
                ("kamn:did:owner:recipient-b", "wrap-b"),
                ("kamn:did:owner:recipient-a", "wrap-a"),
            ],
        ))
        .expect("determinism append a should succeed");
    let mut determinism_ledger_b = DataLayerM0AppendOnlyLedger::new();
    let determinism_b = determinism_ledger_b
        .append(valid_input(
            "msg-c05-det",
            vec!["kamn:did:agent:recipient-a", "kamn:did:agent:recipient-b"],
            vec![
                ("kamn:did:owner:recipient-a", "wrap-a"),
                ("kamn:did:owner:recipient-b", "wrap-b"),
            ],
        ))
        .expect("determinism append b should succeed");
    let envelope_crypto_passed = determinism_a.content_hash == determinism_b.content_hash
        && determinism_a.envelope_aad_hash == determinism_b.envelope_aad_hash;

    let mut append_only_ledger = DataLayerM0AppendOnlyLedger::new();
    append_only_ledger
        .append(valid_input(
            "msg-c05-append",
            vec!["kamn:did:agent:recipient-a"],
            vec![("kamn:did:owner:recipient-a", "wrap-a")],
        ))
        .expect("append-only seed should succeed");
    let duplicate_append = append_only_ledger.append(valid_input(
        "msg-c05-append",
        vec!["kamn:did:agent:recipient-a"],
        vec![("kamn:did:owner:recipient-a", "wrap-a")],
    ));
    let append_only_passed = matches!(
        duplicate_append,
        Err(DataLayerM0Error::DuplicateMessageId(_))
    );

    let mut hash_chain_ledger = DataLayerM0AppendOnlyLedger::new();
    hash_chain_ledger
        .append(valid_input(
            "msg-c05-chain-a",
            vec!["kamn:did:agent:recipient-a"],
            vec![("kamn:did:owner:recipient-a", "wrap-a")],
        ))
        .expect("hash-chain seed append should succeed");
    hash_chain_ledger
        .append(valid_input(
            "msg-c05-chain-b",
            vec!["kamn:did:agent:recipient-b"],
            vec![("kamn:did:owner:recipient-b", "wrap-b")],
        ))
        .expect("hash-chain second append should succeed");
    hash_chain_ledger
        .verify_hash_chain()
        .expect("untampered chain should verify");
    hash_chain_ledger
        .replace_content_hash_unchecked("msg-c05-chain-a", "sha256:tampered")
        .expect("tamper helper should succeed");
    let hash_chain_passed = matches!(
        hash_chain_ledger.verify_hash_chain(),
        Err(DataLayerM0Error::InvalidHashChainLink { .. })
    );

    let report = evaluate_data_layer_m0_conformance_matrix(&[
        DataLayerM0ConformanceMatrixCase {
            case_id: "envelope-crypto".to_owned(),
            invariant: DataLayerM0ConformanceInvariant::EnvelopeCryptoDeterministic,
            observed_passed: envelope_crypto_passed,
            expected_passed: true,
        },
        DataLayerM0ConformanceMatrixCase {
            case_id: "append-only".to_owned(),
            invariant: DataLayerM0ConformanceInvariant::AppendOnlyDuplicateRejected,
            observed_passed: append_only_passed,
            expected_passed: true,
        },
        DataLayerM0ConformanceMatrixCase {
            case_id: "hash-chain".to_owned(),
            invariant: DataLayerM0ConformanceInvariant::HashChainTamperDetected,
            observed_passed: hash_chain_passed,
            expected_passed: true,
        },
    ])
    .expect("conformance matrix should evaluate");

    assert_eq!(
        report.decision,
        DataLayerM0ConformanceMatrixDecision::Stable {
            reason_code: DATA_LAYER_M0_CONFORMANCE_MATRIX_STABLE_REASON_CODE,
        }
    );
    assert_eq!(report.evidence.len(), 3);
    assert!(report.evidence.iter().all(|entry| !entry.mismatch));
}

#[test]
fn spec_c06_m0_conformance_matrix_detects_invariant_drift() {
    let report = evaluate_data_layer_m0_conformance_matrix(&[DataLayerM0ConformanceMatrixCase {
        case_id: "envelope-crypto-drift".to_owned(),
        invariant: DataLayerM0ConformanceInvariant::EnvelopeCryptoDeterministic,
        observed_passed: false,
        expected_passed: true,
    }])
    .expect("conformance matrix should evaluate");

    assert_eq!(
        report.decision,
        DataLayerM0ConformanceMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M0_CONFORMANCE_MATRIX_DRIFT_REASON_CODE,
        }
    );
    assert_eq!(report.evidence.len(), 1);
    assert!(report.evidence[0].mismatch);
}

#[test]
fn spec_c07_m0_conformance_matrix_fails_closed_for_invalid_inputs() {
    let empty = evaluate_data_layer_m0_conformance_matrix(&[]);
    assert_eq!(
        empty,
        Err(DataLayerM0Error::InvalidConformanceMatrixInput("cases"))
    );

    let invalid_case_id =
        evaluate_data_layer_m0_conformance_matrix(&[DataLayerM0ConformanceMatrixCase {
            case_id: " ".to_owned(),
            invariant: DataLayerM0ConformanceInvariant::HashChainTamperDetected,
            observed_passed: true,
            expected_passed: true,
        }]);
    assert_eq!(
        invalid_case_id,
        Err(DataLayerM0Error::InvalidConformanceMatrixInput("case_id"))
    );
}
