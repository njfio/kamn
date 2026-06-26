use super::{
    support::{batch_digest, leaf_digest, node_digest, validate_leaves},
    DataLayerM1Error, DataLayerM1MerkleInclusionProof, DataLayerM1MerkleLeaf,
    DataLayerM1MerkleProofStep, DataLayerM1ProofSiblingSide,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Merkle Batch.
pub struct DataLayerM1MerkleBatch {
    /// Batch id carried by this public contract model.
    pub batch_id: String,
    /// Merkle root carried by this public contract model.
    pub merkle_root: String,
    /// Message count carried by this public contract model.
    pub message_count: usize,
    /// First message id carried by this public contract model.
    pub first_message_id: String,
    /// Last message id carried by this public contract model.
    pub last_message_id: String,
    /// Tree height carried by this public contract model.
    pub tree_height: u16,
    pub(crate) leaves: Vec<DataLayerM1MerkleLeaf>,
    pub(crate) levels: Vec<Vec<String>>,
}

impl DataLayerM1MerkleBatch {
    /// Creates or updates state through the assemble contract operation.
    pub fn assemble(mut leaves: Vec<DataLayerM1MerkleLeaf>) -> Result<Self, DataLayerM1Error> {
        sort_leaves(&mut leaves)?;
        let levels = build_levels(&leaves);
        build_batch(leaves, levels)
    }

    /// Runs the leaves contract operation.
    pub fn leaves(&self) -> &[DataLayerM1MerkleLeaf] {
        &self.leaves
    }

    /// Runs the inclusion proof contract operation.
    pub fn inclusion_proof(
        &self,
        message_id: &str,
    ) -> Result<DataLayerM1MerkleInclusionProof, DataLayerM1Error> {
        let position = find_leaf_position(&self.leaves, message_id)?;
        Ok(build_inclusion_proof(self, position))
    }
}

fn sort_leaves(leaves: &mut [DataLayerM1MerkleLeaf]) -> Result<(), DataLayerM1Error> {
    if leaves.is_empty() {
        return Err(DataLayerM1Error::EmptyBatch);
    }
    leaves.sort_by(|left, right| {
        left.leaf_index
            .cmp(&right.leaf_index)
            .then(left.message_id.cmp(&right.message_id))
    });
    validate_leaves(leaves)
}

fn build_levels(leaves: &[DataLayerM1MerkleLeaf]) -> Vec<Vec<String>> {
    let mut levels = vec![leaves.iter().map(leaf_digest).collect::<Vec<_>>()];
    while let Some(current) = levels.last().filter(|level| level.len() > 1) {
        let next = build_next_level(levels.len() - 1, current);
        levels.push(next);
    }
    levels
}

fn build_next_level(level_index: usize, current: &[String]) -> Vec<String> {
    current
        .chunks(2)
        .map(|pair| {
            let left = pair[0].as_str();
            let right = pair.get(1).unwrap_or(&pair[0]).as_str();
            node_digest(level_index, left, right)
        })
        .collect()
}

fn build_batch(
    leaves: Vec<DataLayerM1MerkleLeaf>,
    levels: Vec<Vec<String>>,
) -> Result<DataLayerM1MerkleBatch, DataLayerM1Error> {
    let first_message_id = leaves[0].message_id.clone();
    let last_message_id = leaves[leaves.len() - 1].message_id.clone();
    let merkle_root = resolve_merkle_root(&levels)?;
    let batch_id = batch_digest(
        &merkle_root,
        leaves.len(),
        &first_message_id,
        &last_message_id,
    );
    Ok(DataLayerM1MerkleBatch {
        batch_id,
        merkle_root,
        message_count: leaves.len(),
        first_message_id,
        last_message_id,
        tree_height: levels.len() as u16,
        leaves,
        levels,
    })
}

fn resolve_merkle_root(levels: &[Vec<String>]) -> Result<String, DataLayerM1Error> {
    levels
        .last()
        .and_then(|level| level.first())
        .cloned()
        .ok_or(DataLayerM1Error::EmptyBatch)
}

fn find_leaf_position(
    leaves: &[DataLayerM1MerkleLeaf],
    message_id: &str,
) -> Result<usize, DataLayerM1Error> {
    leaves
        .iter()
        .position(|leaf| leaf.message_id == message_id)
        .ok_or_else(|| DataLayerM1Error::UnknownMessageId(message_id.to_owned()))
}

fn build_inclusion_proof(
    batch: &DataLayerM1MerkleBatch,
    position: usize,
) -> DataLayerM1MerkleInclusionProof {
    let leaf = &batch.leaves[position];
    DataLayerM1MerkleInclusionProof {
        batch_id: batch.batch_id.clone(),
        merkle_root: batch.merkle_root.clone(),
        message_id: leaf.message_id.clone(),
        leaf_index: leaf.leaf_index,
        content_hash: leaf.content_hash.clone(),
        leaf_hash: batch.levels[0][position].clone(),
        steps: build_proof_steps(&batch.levels, position),
    }
}

fn build_proof_steps(
    levels: &[Vec<String>],
    start_index: usize,
) -> Vec<DataLayerM1MerkleProofStep> {
    let mut steps = Vec::new();
    let mut node_index = start_index;
    for level in &levels[..levels.len() - 1] {
        let (sibling_index, sibling_side) = sibling_step(level, node_index);
        steps.push(DataLayerM1MerkleProofStep {
            sibling_hash: level[sibling_index].clone(),
            sibling_side,
        });
        node_index /= 2;
    }
    steps
}

fn sibling_step(level: &[String], node_index: usize) -> (usize, DataLayerM1ProofSiblingSide) {
    if node_index.is_multiple_of(2) {
        let right = if node_index + 1 < level.len() {
            node_index + 1
        } else {
            node_index
        };
        (right, DataLayerM1ProofSiblingSide::Right)
    } else {
        (node_index - 1, DataLayerM1ProofSiblingSide::Left)
    }
}
