use kamn_core::{
    verify_data_layer_m1_inclusion_proof, DataLayerM1AnchorOutcome, DataLayerM1AnchorRetryClass,
    DataLayerM1Error, DataLayerM1KolmeAnchoringWorker, DataLayerM1MerkleBatch,
    DataLayerM1MerkleLeaf, InMemoryKolmeRuntimeCommitClient,
};

fn leaf(message_id: &str, leaf_index: u32, suffix: &str) -> DataLayerM1MerkleLeaf {
    DataLayerM1MerkleLeaf {
        message_id: message_id.to_owned(),
        leaf_index,
        content_hash: format!("sha256:{suffix}"),
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
