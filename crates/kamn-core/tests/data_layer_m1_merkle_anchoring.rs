use kamn_core::{
    evaluate_data_layer_m1_anchor_failure_matrix, evaluate_data_layer_m1_inclusion_proof,
    verify_data_layer_m1_inclusion_proof, DataLayerM1AnchorFailureMatrixCase,
    DataLayerM1AnchorFailureMatrixDecision, DataLayerM1AnchorOutcome, DataLayerM1AnchorOutcomeKind,
    DataLayerM1AnchorRetryClass, DataLayerM1Error, DataLayerM1KolmeAnchoringWorker,
    DataLayerM1MerkleBatch, DataLayerM1MerkleLeaf, DataLayerM1ProofVerificationDecision,
    InMemoryKolmeRuntimeCommitClient, KolmeCommitReceiptFinality, KolmeRuntimeCommitClient,
    KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitReceipt,
    KolmeRuntimeCommitRequest, DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
    DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE,
    DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
    DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
};
use std::collections::VecDeque;

fn leaf(message_id: &str, leaf_index: u32, suffix: &str) -> DataLayerM1MerkleLeaf {
    DataLayerM1MerkleLeaf {
        message_id: message_id.to_owned(),
        leaf_index,
        content_hash: format!("sha256:{suffix}"),
    }
}

#[derive(Debug, Clone)]
struct ScriptedKolmeRuntimeCommitClient {
    scripted_outcomes: VecDeque<KolmeRuntimeCommitOutcome>,
}

impl ScriptedKolmeRuntimeCommitClient {
    fn new(scripted_outcomes: Vec<KolmeRuntimeCommitOutcome>) -> Self {
        Self {
            scripted_outcomes: scripted_outcomes.into(),
        }
    }
}

impl KolmeRuntimeCommitClient for ScriptedKolmeRuntimeCommitClient {
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
        request.validate()?;
        Ok(self
            .scripted_outcomes
            .pop_front()
            .unwrap_or(KolmeRuntimeCommitOutcome::Rejected {
                reason: "scripted_outcome_exhausted".to_owned(),
            }))
    }
}

#[test]
fn spec_c01_merkle_batch_root_is_deterministic_across_input_orderings() {
    let canonical = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c01-a", 0, "0001"),
        leaf("msg-c01-b", 1, "0002"),
        leaf("msg-c01-c", 2, "0003"),
    ])
    .expect("canonical batch assembly should succeed");

    let reordered = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c01-c", 2, "0003"),
        leaf("msg-c01-a", 0, "0001"),
        leaf("msg-c01-b", 1, "0002"),
    ])
    .expect("reordered batch assembly should succeed");

    assert_eq!(canonical.merkle_root, reordered.merkle_root);
    assert_eq!(canonical.message_count, 3);
    assert_eq!(canonical.first_message_id, "msg-c01-a");
    assert_eq!(canonical.last_message_id, "msg-c01-c");
}

#[test]
fn spec_c02_inclusion_proof_verifies_against_batch_root() {
    let batch = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c02-a", 0, "1001"),
        leaf("msg-c02-b", 1, "1002"),
        leaf("msg-c02-c", 2, "1003"),
        leaf("msg-c02-d", 3, "1004"),
    ])
    .expect("batch assembly should succeed");

    let proof = batch
        .inclusion_proof("msg-c02-c")
        .expect("proof generation should succeed");
    verify_data_layer_m1_inclusion_proof(&proof).expect("proof verification should pass");
    assert_eq!(proof.message_id, "msg-c02-c");
    assert_eq!(proof.merkle_root, batch.merkle_root);
}

#[test]
fn spec_c03_tampered_or_unknown_proofs_fail_closed() {
    let batch = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c03-a", 0, "2001"),
        leaf("msg-c03-b", 1, "2002"),
        leaf("msg-c03-c", 2, "2003"),
    ])
    .expect("batch assembly should succeed");

    let mut proof = batch
        .inclusion_proof("msg-c03-b")
        .expect("proof generation should succeed");
    proof.steps[0].sibling_hash = "sha256:tampered".to_owned();

    let verify = verify_data_layer_m1_inclusion_proof(&proof);
    assert!(matches!(
        verify,
        Err(DataLayerM1Error::InvalidMerkleProof(_))
    ));

    let unknown = batch.inclusion_proof("msg-c03-missing");
    assert_eq!(
        unknown,
        Err(DataLayerM1Error::UnknownMessageId(
            "msg-c03-missing".to_owned()
        ))
    );
}

#[test]
fn spec_c04_kolme_anchoring_worker_is_idempotent_for_duplicate_batch_submission() {
    let batch = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c04-a", 0, "3001"),
        leaf("msg-c04-b", 1, "3002"),
    ])
    .expect("batch assembly should succeed");

    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory kolme client should initialize");
    let mut worker = DataLayerM1KolmeAnchoringWorker::new(
        client,
        "kamn:did:agent:anchor-worker-1",
        "merkle-anchor-root",
    )
    .expect("anchoring worker should initialize");

    let first = worker
        .anchor_batch(&batch)
        .expect("first anchor should succeed");
    assert_eq!(
        first.retry_class,
        DataLayerM1AnchorRetryClass::NewSubmission
    );
    assert!(matches!(
        first.outcome,
        DataLayerM1AnchorOutcome::Submitted(_)
    ));

    let second = worker
        .anchor_batch(&batch)
        .expect("second anchor should succeed");
    assert_eq!(
        second.retry_class,
        DataLayerM1AnchorRetryClass::FinalizedNoRetry
    );
    assert_eq!(first.idempotency_key, second.idempotency_key);
    assert!(matches!(
        second.outcome,
        DataLayerM1AnchorOutcome::Duplicate(_)
    ));
}

#[test]
fn spec_c05_invalid_merkle_inputs_are_rejected() {
    let empty = DataLayerM1MerkleBatch::assemble(Vec::new());
    assert_eq!(empty, Err(DataLayerM1Error::EmptyBatch));

    let duplicate_index = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c05-a", 0, "4001"),
        leaf("msg-c05-b", 0, "4002"),
    ]);
    assert_eq!(
        duplicate_index,
        Err(DataLayerM1Error::DuplicateLeafIndex { leaf_index: 0 })
    );

    let non_contiguous = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c05-a", 0, "4001"),
        leaf("msg-c05-b", 2, "4002"),
    ]);
    assert_eq!(
        non_contiguous,
        Err(DataLayerM1Error::NonContiguousLeafIndexes {
            expected: 1,
            found: 2
        })
    );
}

#[test]
fn spec_c06_proof_verification_decision_uses_stable_reason_constants() {
    let batch = DataLayerM1MerkleBatch::assemble(vec![
        leaf("msg-c06-a", 0, "6001"),
        leaf("msg-c06-b", 1, "6002"),
    ])
    .expect("batch assembly should succeed");
    let proof = batch
        .inclusion_proof("msg-c06-a")
        .expect("proof generation should succeed");

    let valid = evaluate_data_layer_m1_inclusion_proof(&proof);
    assert_eq!(
        valid,
        DataLayerM1ProofVerificationDecision::Valid {
            reason_code: DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
        }
    );

    let mut tampered = proof;
    tampered.leaf_hash = "sha256:tampered".to_owned();
    let invalid = evaluate_data_layer_m1_inclusion_proof(&tampered);
    assert_eq!(
        invalid,
        DataLayerM1ProofVerificationDecision::Invalid {
            reason_code: DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
            error: DataLayerM1Error::InvalidMerkleProof("leaf hash mismatch"),
        }
    );
}

#[test]
fn spec_c07_anchor_failure_matrix_reports_stable_when_expectations_match() {
    let scripted_client = ScriptedKolmeRuntimeCommitClient::new(vec![
        KolmeRuntimeCommitOutcome::Submitted(KolmeRuntimeCommitReceipt {
            provider: "kolme-scripted".to_owned(),
            commit_id: "commit-c07-submitted".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        }),
        KolmeRuntimeCommitOutcome::Duplicate(KolmeRuntimeCommitReceipt {
            provider: "kolme-scripted".to_owned(),
            commit_id: "commit-c07-duplicate".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        }),
        KolmeRuntimeCommitOutcome::Rejected {
            reason: "kolme-policy-reject".to_owned(),
        },
    ]);
    let mut worker = DataLayerM1KolmeAnchoringWorker::new(
        scripted_client,
        "kamn:did:agent:anchor-worker-c07",
        "merkle-anchor-root",
    )
    .expect("anchoring worker should initialize");

    let submitted = worker
        .anchor_batch(
            &DataLayerM1MerkleBatch::assemble(vec![
                leaf("msg-c07-a", 0, "7001"),
                leaf("msg-c07-b", 1, "7002"),
            ])
            .expect("submitted batch should assemble"),
        )
        .expect("submitted outcome should succeed");
    let duplicate = worker
        .anchor_batch(
            &DataLayerM1MerkleBatch::assemble(vec![
                leaf("msg-c07-c", 0, "7003"),
                leaf("msg-c07-d", 1, "7004"),
            ])
            .expect("duplicate batch should assemble"),
        )
        .expect("duplicate outcome should succeed");
    let rejected = worker
        .anchor_batch(
            &DataLayerM1MerkleBatch::assemble(vec![
                leaf("msg-c07-e", 0, "7005"),
                leaf("msg-c07-f", 1, "7006"),
            ])
            .expect("rejected batch should assemble"),
        )
        .expect("rejected outcome should resolve");

    let report = evaluate_data_layer_m1_anchor_failure_matrix(&[
        DataLayerM1AnchorFailureMatrixCase {
            case_id: "submitted".to_owned(),
            result: submitted,
            expected_retry_class: DataLayerM1AnchorRetryClass::NewSubmission,
            expected_outcome_kind: DataLayerM1AnchorOutcomeKind::Submitted,
        },
        DataLayerM1AnchorFailureMatrixCase {
            case_id: "duplicate-pending".to_owned(),
            result: duplicate,
            expected_retry_class: DataLayerM1AnchorRetryClass::RetryableInFlight,
            expected_outcome_kind: DataLayerM1AnchorOutcomeKind::Duplicate,
        },
        DataLayerM1AnchorFailureMatrixCase {
            case_id: "rejected".to_owned(),
            result: rejected,
            expected_retry_class: DataLayerM1AnchorRetryClass::ConflictNoRetry,
            expected_outcome_kind: DataLayerM1AnchorOutcomeKind::Rejected,
        },
    ])
    .expect("failure matrix should evaluate");
    assert_eq!(
        report.decision,
        DataLayerM1AnchorFailureMatrixDecision::Stable {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE,
        }
    );
    assert_eq!(report.evidence.len(), 3);
    assert!(report.evidence.iter().all(|entry| !entry.mismatch));
}

#[test]
fn spec_c08_anchor_failure_matrix_detects_retry_or_outcome_drift() {
    let drift_case = DataLayerM1AnchorFailureMatrixCase {
        case_id: "drift-submitted".to_owned(),
        result: kamn_core::DataLayerM1AnchorResult {
            batch_id: "batch-drift".to_owned(),
            idempotency_key: "idem-drift".to_owned(),
            retry_class: DataLayerM1AnchorRetryClass::NewSubmission,
            outcome: DataLayerM1AnchorOutcome::Submitted(kamn_core::DataLayerM1AnchorReceipt {
                provider: "kolme-scripted".to_owned(),
                transaction_id: "commit-drift".to_owned(),
                finality: KolmeCommitReceiptFinality::Pending,
            }),
        },
        expected_retry_class: DataLayerM1AnchorRetryClass::FinalizedNoRetry,
        expected_outcome_kind: DataLayerM1AnchorOutcomeKind::Duplicate,
    };

    let report = evaluate_data_layer_m1_anchor_failure_matrix(&[drift_case])
        .expect("matrix should evaluate");
    assert_eq!(
        report.decision,
        DataLayerM1AnchorFailureMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
        }
    );
    assert_eq!(report.evidence.len(), 1);
    assert!(report.evidence[0].mismatch);
}

#[test]
fn spec_c09_anchor_failure_matrix_fails_closed_for_invalid_inputs() {
    let empty = evaluate_data_layer_m1_anchor_failure_matrix(&[]);
    assert_eq!(
        empty,
        Err(DataLayerM1Error::InvalidFailureMatrixInput("cases"))
    );

    let invalid_case_id =
        evaluate_data_layer_m1_anchor_failure_matrix(&[DataLayerM1AnchorFailureMatrixCase {
            case_id: " ".to_owned(),
            result: kamn_core::DataLayerM1AnchorResult {
                batch_id: "batch-invalid".to_owned(),
                idempotency_key: "idem-invalid".to_owned(),
                retry_class: DataLayerM1AnchorRetryClass::ConflictNoRetry,
                outcome: DataLayerM1AnchorOutcome::Rejected {
                    reason: "rejected".to_owned(),
                },
            },
            expected_retry_class: DataLayerM1AnchorRetryClass::ConflictNoRetry,
            expected_outcome_kind: DataLayerM1AnchorOutcomeKind::Rejected,
        }]);
    assert_eq!(
        invalid_case_id,
        Err(DataLayerM1Error::InvalidFailureMatrixInput("case_id"))
    );
}
