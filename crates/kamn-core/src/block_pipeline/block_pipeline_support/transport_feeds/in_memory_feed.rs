use super::super::*;
use super::traits::{TransportCanonicalCandidateFeed, TransportMempoolFeed};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory transport feed used for deterministic tests and local contract lanes.
pub struct InMemoryTransportMempoolFeed {
    pending: Vec<BaselineTransaction>,
}

impl InMemoryTransportMempoolFeed {
    /// Creates in-memory transport feed with deterministic pending queue.
    pub fn new(pending: Vec<BaselineTransaction>) -> Self {
        Self { pending }
    }

    /// Pushes candidate transaction into pending queue.
    pub fn push(&mut self, tx: BaselineTransaction) {
        self.pending.push(tx);
    }
}

impl TransportMempoolFeed for InMemoryTransportMempoolFeed {
    fn drain_pending_transactions(
        &mut self,
    ) -> Result<Vec<BaselineTransaction>, BlockPipelineError> {
        Ok(std::mem::take(&mut self.pending))
    }
}

impl TransportCanonicalCandidateFeed for InMemoryTransportMempoolFeed {
    fn drain_canonical_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        Ok(Vec::new())
    }
}
