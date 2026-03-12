use crate::transaction::BaselineTransaction;
use crate::BlockPipelineError;

use super::super::CanonicalCommitRecord;

/// Transport feed abstraction for draining pending mempool candidates.
pub trait TransportMempoolFeed {
    /// Drains pending transport candidates in implementation-defined order.
    fn drain_pending_transactions(
        &mut self,
    ) -> Result<Vec<BaselineTransaction>, BlockPipelineError>;
}

/// Transport feed abstraction for draining received canonical block candidates.
pub trait TransportCanonicalCandidateFeed {
    /// Drains canonical block candidates discovered via transport gossip.
    fn drain_canonical_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError>;
}
