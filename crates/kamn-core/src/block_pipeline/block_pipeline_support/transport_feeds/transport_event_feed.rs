use crate::p2p_transport::{PeerGossipFrame, PeerLifecycleTransport};
use crate::transaction::BaselineTransaction;
use crate::BlockPipelineError;
use std::collections::BTreeSet;

use super::super::{CanonicalCommitRecord, GossipIngressAdapter};
use super::traits::{TransportCanonicalCandidateFeed, TransportMempoolFeed};

/// Transport-backed feed that drains one peer inbox and decodes transport ingress records.
#[derive(Debug, Clone)]
pub struct TransportEventMempoolFeed<TTransport> {
    transport: TTransport,
    local_peer_id: String,
    required_topics: Option<BTreeSet<String>>,
    pending_transactions: Vec<BaselineTransaction>,
    pending_candidates: Vec<CanonicalCommitRecord>,
}

impl<TTransport> TransportEventMempoolFeed<TTransport> {
    /// Creates a transport-event feed bound to one local peer inbox.
    pub fn new(
        transport: TTransport,
        local_peer_id: &str,
        required_topics: Option<Vec<String>>,
    ) -> Result<Self, BlockPipelineError> {
        let required_topics = normalize_required_topics(local_peer_id, required_topics)?;
        Ok(Self {
            transport,
            local_peer_id: local_peer_id.to_owned(),
            required_topics,
            pending_transactions: Vec::new(),
            pending_candidates: Vec::new(),
        })
    }

    fn decode_inbox_if_needed(&mut self) -> Result<(), BlockPipelineError>
    where
        TTransport: PeerLifecycleTransport,
    {
        if !self.pending_transactions.is_empty() || !self.pending_candidates.is_empty() {
            return Ok(());
        }
        let frames = drain_transport_inbox(&self.transport, self.local_peer_id.as_str())?;
        if frames.is_empty() {
            return Ok(());
        }
        validate_required_topics(self.required_topics.as_ref(), &frames)?;
        let decoded = GossipIngressAdapter::decode_frames(&frames).map_err(|error| {
            BlockPipelineError::TransportFeed(format!("{}:{}", error.reason_code(), error))
        })?;
        self.pending_transactions = decoded.transactions;
        self.pending_candidates = decoded.canonical_candidates;
        Ok(())
    }
}

impl<TTransport> TransportMempoolFeed for TransportEventMempoolFeed<TTransport>
where
    TTransport: PeerLifecycleTransport,
{
    fn drain_pending_transactions(
        &mut self,
    ) -> Result<Vec<BaselineTransaction>, BlockPipelineError> {
        self.decode_inbox_if_needed()?;
        Ok(std::mem::take(&mut self.pending_transactions))
    }
}

impl<TTransport> TransportCanonicalCandidateFeed for TransportEventMempoolFeed<TTransport>
where
    TTransport: PeerLifecycleTransport,
{
    fn drain_canonical_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        self.decode_inbox_if_needed()?;
        Ok(std::mem::take(&mut self.pending_candidates))
    }
}

fn normalize_required_topics(
    local_peer_id: &str,
    required_topics: Option<Vec<String>>,
) -> Result<Option<BTreeSet<String>>, BlockPipelineError> {
    if local_peer_id.trim().is_empty() {
        return Err(BlockPipelineError::TransportFeed(
            "transport feed local peer id is empty (transport_feed_local_peer_id_invalid)"
                .to_owned(),
        ));
    }
    required_topics.map(normalize_topic_set).transpose()
}

fn normalize_topic_set(topics: Vec<String>) -> Result<BTreeSet<String>, BlockPipelineError> {
    if topics.is_empty() {
        return Err(BlockPipelineError::TransportFeed(
            "transport feed required topics list is empty (transport_feed_topics_invalid)"
                .to_owned(),
        ));
    }
    let mut normalized = BTreeSet::new();
    for topic in topics {
        if topic.trim().is_empty() {
            return Err(BlockPipelineError::TransportFeed(
                "transport feed required topic is empty (transport_feed_topics_invalid)".to_owned(),
            ));
        }
        normalized.insert(topic.trim().to_owned());
    }
    Ok(normalized)
}

fn drain_transport_inbox<TTransport>(
    transport: &TTransport,
    local_peer_id: &str,
) -> Result<Vec<PeerGossipFrame>, BlockPipelineError>
where
    TTransport: PeerLifecycleTransport,
{
    transport.drain_inbox(local_peer_id).map_err(|error| {
        BlockPipelineError::TransportFeed(format!(
            "transport feed inbox drain failed: {error} (transport_feed_inbox_drain_failed)"
        ))
    })
}

fn validate_required_topics(
    required_topics: Option<&BTreeSet<String>>,
    frames: &[PeerGossipFrame],
) -> Result<(), BlockPipelineError> {
    if let Some(required_topics) = required_topics {
        for frame in frames {
            if !required_topics.contains(frame.topic.as_str()) {
                return Err(BlockPipelineError::TransportFeed(format!(
                    "transport frame topic mismatch: found {} (transport_candidate_topic_mismatch)",
                    frame.topic
                )));
            }
        }
    }
    Ok(())
}
