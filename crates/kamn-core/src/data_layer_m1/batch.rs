use super::{
    support::{batch_digest, leaf_digest, node_digest, validate_leaves},
    DataLayerM1Error, DataLayerM1MerkleInclusionProof,
    DataLayerM1MerkleLeaf, DataLayerM1MerkleProofStep, DataLayerM1ProofSiblingSide,
};

/// Deterministic merkle batch projection over content hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleBatch {
    /// Deterministic batch identifier.
    pub batch_id: String,
    /// Batch merkle root.
    pub merkle_root: String,
    /// Number of messages in the batch.
    pub message_count: usize,
    /// First message identifier in canonical leaf order.
    pub first_message_id: String,
    /// Last message identifier in canonical leaf order.
    pub last_message_id: String,
    /// Merkle tree height (leaf level included).
    pub tree_height: u16,
    pub(crate) leaves: Vec<DataLayerM1MerkleLeaf>,
    pub(crate) levels: Vec<Vec<String>>,
}

impl DataLayerM1MerkleBatch {
    /// Assembles a deterministic merkle batch from message leaves.
    pub fn assemble(mut leaves: Vec<DataLayerM1MerkleLeaf>) -> Result<Self, DataLayerM1Error> {
        if leaves.is_empty() {
            return Err(DataLayerM1Error::EmptyBatch);
        }

        leaves.sort_by(|left, right| {
            left.leaf_index
                .cmp(&right.leaf_index)
                .then(left.message_id.cmp(&right.message_id))
        });

        validate_leaves(&leaves)?;

        let mut levels = Vec::new();
        let mut current = leaves.iter().map(leaf_digest).collect::<Vec<_>>();
        levels.push(current.clone());

        while current.len() > 1 {
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            let mut index = 0usize;
            while index < current.len() {
                let left = current[index].as_str();
                let right = current.get(index + 1).unwrap_or(&current[index]).as_str();
                next.push(node_digest(levels.len() - 1, left, right));
                index += 2;
            }
            levels.push(next.clone());
            current = next;
        }

        let first_message_id = leaves[0].message_id.clone();
        let last_message_id = leaves[leaves.len() - 1].message_id.clone();
        let merkle_root = current[0].clone();
        let batch_id = batch_digest(
            merkle_root.as_str(),
            leaves.len(),
            first_message_id.as_str(),
            last_message_id.as_str(),
        );

        Ok(Self {
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

    /// Returns canonical leaves in deterministic index order.
    pub fn leaves(&self) -> &[DataLayerM1MerkleLeaf] {
        &self.leaves
    }

    /// Builds an inclusion proof for one message in this batch.
    pub fn inclusion_proof(&self, message_id: &str) -> Result<DataLayerM1MerkleInclusionProof, DataLayerM1Error> {
        let position = self
            .leaves
            .iter()
            .position(|leaf| leaf.message_id == message_id)
            .ok_or_else(|| DataLayerM1Error::UnknownMessageId(message_id.to_owned()))?;
        let leaf = &self.leaves[position];

        let mut steps = Vec::new();
        let mut node_index = position;
        for level_index in 0..self.levels.len() - 1 {
            let level = &self.levels[level_index];
            let (sibling_index, sibling_side) = if node_index % 2 == 0 {
                let right = if node_index + 1 < level.len() { node_index + 1 } else { node_index };
                (right, DataLayerM1ProofSiblingSide::Right)
            } else {
                (node_index - 1, DataLayerM1ProofSiblingSide::Left)
            };
            steps.push(DataLayerM1MerkleProofStep {
                sibling_hash: level[sibling_index].clone(),
                sibling_side,
            });
            node_index /= 2;
        }

        Ok(DataLayerM1MerkleInclusionProof {
            batch_id: self.batch_id.clone(),
            merkle_root: self.merkle_root.clone(),
            message_id: leaf.message_id.clone(),
            leaf_index: leaf.leaf_index,
            content_hash: leaf.content_hash.clone(),
            leaf_hash: self.levels[0][position].clone(),
            steps,
        })
    }
}
