use crate::data_layer_hashing::tagged_sha256;
use crate::kolme_runtime_commit::KolmeRuntimeCommitReceipt;
use std::collections::BTreeSet;

use super::{
    DataLayerM1AnchorOutcome, DataLayerM1AnchorOutcomeKind, DataLayerM1AnchorReceipt,
    DataLayerM1Error, DataLayerM1MerkleLeaf, DATA_LAYER_M1_HASH_ALGORITHM,
};

pub(crate) fn validate_leaves(leaves: &[DataLayerM1MerkleLeaf]) -> Result<(), DataLayerM1Error> {
    let mut seen_indexes = BTreeSet::new();
    let mut seen_message_ids = BTreeSet::new();
    for (position, leaf) in leaves.iter().enumerate() {
        validate_leaf_content(leaf)?;
        validate_leaf_uniqueness(leaf, &mut seen_indexes, &mut seen_message_ids)?;
        validate_leaf_position(position, leaf)?;
    }
    Ok(())
}

fn validate_leaf_content(leaf: &DataLayerM1MerkleLeaf) -> Result<(), DataLayerM1Error> {
    if leaf.message_id.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("message_id"));
    }
    if !is_valid_content_hash(leaf.content_hash.as_str()) {
        return Err(DataLayerM1Error::InvalidContentHash(
            leaf.content_hash.clone(),
        ));
    }
    Ok(())
}

fn validate_leaf_uniqueness(
    leaf: &DataLayerM1MerkleLeaf,
    seen_indexes: &mut BTreeSet<u32>,
    seen_message_ids: &mut BTreeSet<String>,
) -> Result<(), DataLayerM1Error> {
    if !seen_indexes.insert(leaf.leaf_index) {
        return Err(DataLayerM1Error::DuplicateLeafIndex {
            leaf_index: leaf.leaf_index,
        });
    }
    if !seen_message_ids.insert(leaf.message_id.clone()) {
        return Err(DataLayerM1Error::DuplicateMessageId(
            leaf.message_id.clone(),
        ));
    }
    Ok(())
}

fn validate_leaf_position(
    position: usize,
    leaf: &DataLayerM1MerkleLeaf,
) -> Result<(), DataLayerM1Error> {
    let expected_index = position as u32;
    if leaf.leaf_index != expected_index {
        return Err(DataLayerM1Error::NonContiguousLeafIndexes {
            expected: expected_index,
            found: leaf.leaf_index,
        });
    }
    Ok(())
}

pub(crate) fn is_valid_content_hash(content_hash: &str) -> bool {
    let trimmed = content_hash.trim();
    trimmed.starts_with("sha256:") && trimmed.len() > "sha256:".len()
}

pub(crate) fn leaf_digest(leaf: &DataLayerM1MerkleLeaf) -> String {
    tagged_digest(
        format!(
            "leaf|index:{}|id:{}|content:{}",
            leaf.leaf_index, leaf.message_id, leaf.content_hash
        )
        .as_str(),
    )
}

pub(crate) fn node_digest(level: usize, left: &str, right: &str) -> String {
    tagged_digest(format!("node|level:{level}|left:{left}|right:{right}").as_str())
}

pub(crate) fn batch_digest(
    merkle_root: &str,
    message_count: usize,
    first: &str,
    last: &str,
) -> String {
    tagged_digest(
        format!("batch|root:{merkle_root}|count:{message_count}|first:{first}|last:{last}")
            .as_str(),
    )
}

pub(crate) fn tagged_digest(value: &str) -> String {
    tagged_sha256(value, DATA_LAYER_M1_HASH_ALGORITHM)
}

pub(crate) fn map_receipt(receipt: KolmeRuntimeCommitReceipt) -> DataLayerM1AnchorReceipt {
    DataLayerM1AnchorReceipt {
        provider: receipt.provider,
        transaction_id: receipt.commit_id,
        finality: receipt.finality,
    }
}

pub(crate) fn anchor_outcome_kind(
    outcome: &DataLayerM1AnchorOutcome,
) -> DataLayerM1AnchorOutcomeKind {
    match outcome {
        DataLayerM1AnchorOutcome::Submitted(_) => DataLayerM1AnchorOutcomeKind::Submitted,
        DataLayerM1AnchorOutcome::Duplicate(_) => DataLayerM1AnchorOutcomeKind::Duplicate,
        DataLayerM1AnchorOutcome::Rejected { .. } => DataLayerM1AnchorOutcomeKind::Rejected,
    }
}
