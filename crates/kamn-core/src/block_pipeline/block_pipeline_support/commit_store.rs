use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical commit record persisted after fork-choice acceptance.
pub struct CanonicalCommitRecord {
    /// Committed block height.
    pub block_height: u64,
    /// Producer role for the committed block.
    pub producer_role: NodeRole,
    /// Deterministic block payload digest.
    pub payload_digest: String,
    /// Ordered committed transaction identifiers.
    pub transaction_ids: Vec<String>,
}

impl CanonicalCommitRecord {
    pub(crate) fn from_commit_report(report: &BlockPipelineCommitReport) -> Self {
        Self {
            block_height: report.block.height,
            producer_role: report.block.producer.clone(),
            payload_digest: report.payload_digest.clone(),
            transaction_ids: report
                .block
                .transactions
                .iter()
                .map(|tx| tx.id.clone())
                .collect(),
        }
    }
}

pub use super::super::commit_store::{
    CanonicalCommitStore, FileCanonicalCommitStore, InMemoryCanonicalCommitStore,
    SqliteCanonicalCommitStore,
};
