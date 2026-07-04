use super::{
    verify_data_layer_m1_inclusion_proof, DataLayerM1Error, DataLayerM1MerkleBatch,
    DataLayerM1MerkleLeaf,
};

fn fixture_leaves() -> Vec<DataLayerM1MerkleLeaf> {
    vec![
        DataLayerM1MerkleLeaf {
            message_id: "msg-1".to_owned(),
            leaf_index: 0,
            content_hash: "sha256:a1".to_owned(),
        },
        DataLayerM1MerkleLeaf {
            message_id: "msg-2".to_owned(),
            leaf_index: 1,
            content_hash: "sha256:b2".to_owned(),
        },
    ]
}

#[test]
fn unit_data_layer_m1_merkle_batch_proof_verifies() {
    let batch = DataLayerM1MerkleBatch::assemble(fixture_leaves())
        .expect("merkle batch should assemble deterministically");
    let proof = batch
        .inclusion_proof("msg-1")
        .expect("inclusion proof should build for known message");
    verify_data_layer_m1_inclusion_proof(&proof).expect("freshly generated proof should verify");
}

#[test]
fn unit_data_layer_m1_merkle_batch_rejects_tampered_root() {
    let batch = DataLayerM1MerkleBatch::assemble(fixture_leaves())
        .expect("merkle batch should assemble deterministically");
    let mut proof = batch
        .inclusion_proof("msg-1")
        .expect("inclusion proof should build for known message");
    proof.merkle_root = "sha256:tampered".to_owned();
    let error = verify_data_layer_m1_inclusion_proof(&proof)
        .expect_err("tampered proof root must fail verification");
    assert!(matches!(error, DataLayerM1Error::InvalidMerkleProof(_)));
}
