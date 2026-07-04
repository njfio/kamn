use crate::p2p_transport::PeerGossipFrame;
use crate::transaction::BaselineTransaction;
use crate::BlockPipelineError;

use super::super::{CanonicalCommitRecord, GossipIngressAdapter};
use super::traits::{TransportCanonicalCandidateFeed, TransportMempoolFeed};

/// Transport feed adapter that decodes gossip frames into mempool candidates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GossipFrameTransportMempoolFeed {
    pending_frames: Vec<PeerGossipFrame>,
    pending_transactions: Vec<BaselineTransaction>,
    canonical_candidates: Vec<CanonicalCommitRecord>,
}

impl GossipFrameTransportMempoolFeed {
    /// Builds feed with pending gossip frames.
    pub fn new(pending_frames: Vec<PeerGossipFrame>) -> Self {
        Self {
            pending_frames,
            pending_transactions: Vec::new(),
            canonical_candidates: Vec::new(),
        }
    }

    /// Drains normalized canonical block candidates decoded during last feed drain.
    pub fn drain_canonical_candidates(&mut self) -> Vec<CanonicalCommitRecord> {
        std::mem::take(&mut self.canonical_candidates)
    }

    fn decode_pending_frames_if_needed(&mut self) -> Result<(), BlockPipelineError> {
        if self.pending_frames.is_empty() {
            return Ok(());
        }
        let decoded =
            GossipIngressAdapter::decode_frames(&self.pending_frames).map_err(|error| {
                BlockPipelineError::TransportFeed(format!("{}:{}", error.reason_code(), error))
            })?;
        self.pending_frames.clear();
        self.pending_transactions.extend(decoded.transactions);
        self.canonical_candidates
            .extend(decoded.canonical_candidates);
        Ok(())
    }
}

impl TransportMempoolFeed for GossipFrameTransportMempoolFeed {
    fn drain_pending_transactions(
        &mut self,
    ) -> Result<Vec<BaselineTransaction>, BlockPipelineError> {
        self.decode_pending_frames_if_needed()?;
        Ok(std::mem::take(&mut self.pending_transactions))
    }
}

impl TransportCanonicalCandidateFeed for GossipFrameTransportMempoolFeed {
    fn drain_canonical_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        self.decode_pending_frames_if_needed()?;
        Ok(std::mem::take(&mut self.canonical_candidates))
    }
}
