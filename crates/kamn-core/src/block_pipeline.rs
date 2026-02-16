//! Mempool block production and consensus validation pipeline contracts.

use crate::config::NodeRole;
use crate::p2p_transport::{PeerGossipFrame, PeerLifecycleTransport};
use crate::runtime::{
    ApproverAttestation, ApproverQuorumDecision, ApproverQuorumError, ApproverQuorumEvaluator,
    ApproverQuorumInput, ListenerAttestation, ListenerQuorumDecision, ListenerQuorumError,
    ListenerQuorumEvaluator, ListenerQuorumInput,
};
use crate::smoke::{ProducedBlock, RoleSmokeNetwork, SmokeError};
use crate::transaction::BaselineTransaction;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

mod commit_store;
mod evidence;
mod fork_choice;
mod gossip_ingress;
mod validation;
pub use commit_store::{
    CanonicalCommitStore, FileCanonicalCommitStore, InMemoryCanonicalCommitStore,
    SqliteCanonicalCommitStore,
};
pub use evidence::{
    build_canonical_replay_evidence_bundle, build_transport_convergence_evidence_bundle,
    CanonicalCandidateDecision, CanonicalCandidateOutcome, CanonicalReplayEvidenceBundle,
    TransportConvergenceEvidenceBundle,
};
pub use fork_choice::{
    AcceptAllForkChoiceHook, DeterministicCompetingBranchForkChoiceHook, ForkChoiceDecision,
    ForkChoiceHook,
};
pub use gossip_ingress::{
    decode_transport_candidate_payload, decode_transport_canonical_candidate_payload,
    encode_transport_candidate_payload, encode_transport_canonical_candidate_payload,
    encode_transport_commit_report_payload, GossipIngressAdapter, GossipIngressBatch,
    GossipIngressError, GossipIngressRecord,
};
use validation::extract_error_reason_marker;

/// Input contract for one consensus-validation and block-commit round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockConsensusRoundInput {
    /// Listener quorum event identifier.
    pub listener_event_id: String,
    /// Listener quorum event sequence.
    pub listener_event_sequence: u64,
    /// Approver outbound action identifier.
    pub outbound_action_id: String,
    /// Listener votes as `(listener_did, attestation_id)`.
    pub listener_votes: Vec<(String, String)>,
    /// Approver votes as `(approver_did, attestation_id, payload_digest_override)`.
    pub approver_votes: Vec<(String, String, Option<String>)>,
}

/// Commit report emitted when a consensus round commits a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPipelineCommitReport {
    /// Committed block payload.
    pub block: ProducedBlock,
    /// Listener quorum decision used for admission.
    pub listener_decision: ListenerQuorumDecision,
    /// Approver quorum decision used for authorization.
    pub approver_decision: ApproverQuorumDecision,
    /// Deterministic payload digest used by approver quorum validation.
    pub payload_digest: String,
}

/// Error variants for block pipeline validation and commit flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockPipelineError {
    /// Listener quorum validation failed.
    Listener(ListenerQuorumError),
    /// Approver quorum validation failed.
    Approver(ApproverQuorumError),
    /// Smoke network transport/guard operation failed.
    Smoke(SmokeError),
    /// Pipeline attempted a consensus round with an empty mempool.
    EmptyMempool,
    /// Approver payload digest override mismatched deterministic block digest.
    ConsensusPayloadDigestMismatch {
        /// Expected deterministic digest.
        expected: String,
        /// Found override digest.
        found: String,
    },
    /// Transport feed returned an error while draining mempool candidates.
    TransportFeed(String),
    /// Canonical commit store returned an error while persisting/listing records.
    CommitStore(String),
    /// Fork-choice hook rejected canonical candidate block.
    ForkChoiceRejected {
        /// Deterministic reason code supplied by fork-choice hook.
        reason_code: String,
    },
    /// Restart/replay lineage drift detected for canonical commit persistence.
    ReplayDrift {
        /// Deterministic replay drift reason code.
        reason_code: String,
        /// Human-readable drift details.
        detail: String,
    },
}

impl BlockPipelineError {
    /// Returns deterministic reason code marker for policy and recovery matrix checks.
    pub fn reason_code(&self) -> String {
        match self {
            Self::Listener(_) => "block_pipeline_listener_error".to_owned(),
            Self::Approver(_) => "block_pipeline_approver_error".to_owned(),
            Self::Smoke(_) => "block_pipeline_smoke_error".to_owned(),
            Self::EmptyMempool => "block_pipeline_empty_mempool".to_owned(),
            Self::ConsensusPayloadDigestMismatch { .. } => {
                "block_pipeline_payload_digest_mismatch".to_owned()
            }
            Self::TransportFeed(detail) => extract_error_reason_marker(detail)
                .unwrap_or_else(|| "block_pipeline_transport_feed_error".to_owned()),
            Self::CommitStore(detail) => extract_error_reason_marker(detail)
                .unwrap_or_else(|| "block_pipeline_commit_store_error".to_owned()),
            Self::ForkChoiceRejected { reason_code } => reason_code.clone(),
            Self::ReplayDrift { reason_code, .. } => reason_code.clone(),
        }
    }
}

impl From<ListenerQuorumError> for BlockPipelineError {
    fn from(value: ListenerQuorumError) -> Self {
        Self::Listener(value)
    }
}

impl From<ApproverQuorumError> for BlockPipelineError {
    fn from(value: ApproverQuorumError) -> Self {
        Self::Approver(value)
    }
}

impl From<SmokeError> for BlockPipelineError {
    fn from(value: SmokeError) -> Self {
        Self::Smoke(value)
    }
}

impl Display for BlockPipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Listener(error) => write!(f, "{error}"),
            Self::Approver(error) => write!(f, "{error}"),
            Self::Smoke(error) => write!(f, "{error}"),
            Self::EmptyMempool => write!(
                f,
                "block pipeline requires at least one mempool transaction"
            ),
            Self::ConsensusPayloadDigestMismatch { expected, found } => {
                write!(
                    f,
                    "block pipeline payload digest mismatch: expected {expected}, found {found}"
                )
            }
            Self::TransportFeed(detail) => {
                write!(f, "block pipeline transport feed error: {detail}")
            }
            Self::CommitStore(detail) => write!(f, "block pipeline commit store error: {detail}"),
            Self::ForkChoiceRejected { reason_code } => {
                write!(
                    f,
                    "block pipeline fork-choice rejected candidate: {reason_code}"
                )
            }
            Self::ReplayDrift {
                reason_code,
                detail,
            } => {
                write!(f, "block pipeline replay drift: {detail} ({reason_code})")
            }
        }
    }
}

impl Error for BlockPipelineError {}

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
    fn from_commit_report(report: &BlockPipelineCommitReport) -> Self {
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
        if local_peer_id.trim().is_empty() {
            return Err(BlockPipelineError::TransportFeed(
                "transport feed local peer id is empty (transport_feed_local_peer_id_invalid)"
                    .to_owned(),
            ));
        }
        let required_topics = match required_topics {
            Some(topics) => {
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
                            "transport feed required topic is empty (transport_feed_topics_invalid)"
                                .to_owned(),
                        ));
                    }
                    normalized.insert(topic.trim().to_owned());
                }
                Some(normalized)
            }
            None => None,
        };

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

        let frames = self
            .transport
            .drain_inbox(self.local_peer_id.as_str())
            .map_err(|error| {
                BlockPipelineError::TransportFeed(format!(
                    "transport feed inbox drain failed: {error} (transport_feed_inbox_drain_failed)"
                ))
            })?;
        if frames.is_empty() {
            return Ok(());
        }

        if let Some(required_topics) = self.required_topics.as_ref() {
            for frame in &frames {
                if !required_topics.contains(frame.topic.as_str()) {
                    return Err(BlockPipelineError::TransportFeed(format!(
                        "transport frame topic mismatch: found {} (transport_candidate_topic_mismatch)",
                        frame.topic
                    )));
                }
            }
        }

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

/// Deterministic mempool->consensus->commit pipeline for processor runtime flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockPipeline {
    network: RoleSmokeNetwork,
    listener_evaluator: ListenerQuorumEvaluator,
    approver_required_approvals: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Transport-fed block pipeline that persists canonical commit records.
pub struct TransportFedBlockPipeline<TFeed, TStore, THook> {
    pipeline: MempoolBlockPipeline,
    transport_feed: TFeed,
    commit_store: TStore,
    fork_choice_hook: THook,
}

impl<TFeed, TStore, THook> TransportFedBlockPipeline<TFeed, TStore, THook>
where
    TFeed: TransportMempoolFeed + TransportCanonicalCandidateFeed,
    TStore: CanonicalCommitStore,
    THook: ForkChoiceHook,
{
    /// Builds transport-fed block pipeline with supplied transport, persistence, and fork-choice hook.
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
        transport_feed: TFeed,
        commit_store: TStore,
        fork_choice_hook: THook,
    ) -> Result<Self, BlockPipelineError> {
        Ok(Self {
            pipeline: MempoolBlockPipeline::new(
                gossip_enabled,
                listener_required_confirmations,
                approver_required_approvals,
            )?,
            transport_feed,
            commit_store,
            fork_choice_hook,
        })
    }

    /// Reconciles transport-received canonical block candidates through fork-choice.
    pub fn reconcile_transport_candidates(
        &mut self,
    ) -> Result<Vec<CanonicalCandidateOutcome>, BlockPipelineError> {
        let mut candidates = self.transport_feed.drain_canonical_candidates()?;
        sort_canonical_candidates_for_reconciliation(&mut candidates);

        let mut outcomes = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let block_height = candidate.block_height;
            let payload_digest = candidate.payload_digest.clone();
            match self.fork_choice_hook.evaluate_candidate(&candidate)? {
                ForkChoiceDecision::Accept => {
                    self.commit_store.persist_canonical_commit(candidate)?;
                    outcomes.push(CanonicalCandidateOutcome {
                        block_height,
                        payload_digest,
                        decision: CanonicalCandidateDecision::Accepted,
                    });
                }
                ForkChoiceDecision::Reject { reason_code } => {
                    outcomes.push(CanonicalCandidateOutcome {
                        block_height,
                        payload_digest,
                        decision: CanonicalCandidateDecision::Rejected { reason_code },
                    });
                }
            }
        }
        Ok(outcomes)
    }

    /// Runs one transport-fed consensus round and persists canonical commit record.
    pub fn run_transport_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
        let _candidate_outcomes = self.reconcile_transport_candidates()?;
        let mut candidates = self.transport_feed.drain_pending_transactions()?;
        if candidates.is_empty() {
            return Err(BlockPipelineError::EmptyMempool);
        }
        sort_candidates_for_ingress(&mut candidates);
        for candidate in candidates {
            self.pipeline.submit_transaction(candidate)?;
        }

        let report = self.pipeline.run_consensus_round(input)?;
        let commit_record = CanonicalCommitRecord::from_commit_report(&report);
        match self.fork_choice_hook.evaluate_candidate(&commit_record)? {
            ForkChoiceDecision::Accept => {}
            ForkChoiceDecision::Reject { reason_code } => {
                return Err(BlockPipelineError::ForkChoiceRejected { reason_code });
            }
        }
        self.commit_store.persist_canonical_commit(commit_record)?;
        Ok(report)
    }

    /// Lists canonical commit records from configured persistence backend.
    pub fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        self.commit_store.list_canonical_commits()
    }
}

fn sort_candidates_for_ingress(candidates: &mut [BaselineTransaction]) {
    candidates.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.sender.cmp(&right.sender))
    });
}

fn sort_canonical_candidates_for_reconciliation(candidates: &mut [CanonicalCommitRecord]) {
    candidates.sort_by(|left, right| {
        left.block_height
            .cmp(&right.block_height)
            .then_with(|| left.payload_digest.cmp(&right.payload_digest))
            .then_with(|| {
                left.producer_role
                    .as_str()
                    .cmp(right.producer_role.as_str())
            })
            .then_with(|| left.transaction_ids.cmp(&right.transaction_ids))
    });
}

impl MempoolBlockPipeline {
    /// Builds a new block pipeline with explicit listener/approver quorum thresholds.
    pub fn new(
        gossip_enabled: bool,
        listener_required_confirmations: usize,
        approver_required_approvals: usize,
    ) -> Result<Self, BlockPipelineError> {
        let listener_evaluator = ListenerQuorumEvaluator::new(listener_required_confirmations)?;
        // Validate approver threshold during construction.
        let _ = ApproverQuorumEvaluator::new(approver_required_approvals)?;

        Ok(Self {
            network: RoleSmokeNetwork::new(gossip_enabled),
            listener_evaluator,
            approver_required_approvals,
        })
    }

    /// Returns expected state hash for the next mempool submission.
    pub fn expected_state_hash(&self) -> &str {
        self.network.expected_state_hash()
    }

    /// Returns current processor mempool length.
    pub fn processor_mempool_len(&self) -> usize {
        self.network.processor.mempool_len()
    }

    /// Submits a transaction into the pipeline mempool using existing guardrails.
    pub fn submit_transaction(
        &mut self,
        tx: BaselineTransaction,
    ) -> Result<(), BlockPipelineError> {
        self.network.submit_transaction(tx)?;
        Ok(())
    }

    /// Runs listener/approver consensus validation and commits one produced block.
    pub fn run_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
        let pending = self.network.processor_mempool_snapshot();
        if pending.is_empty() {
            return Err(BlockPipelineError::EmptyMempool);
        }

        let payload_digest = payload_digest_for_transactions(&pending);
        for (_, _, override_digest) in &input.approver_votes {
            if let Some(found) = override_digest {
                if found != &payload_digest {
                    return Err(BlockPipelineError::ConsensusPayloadDigestMismatch {
                        expected: payload_digest.clone(),
                        found: found.clone(),
                    });
                }
            }
        }

        let listener_attestations = input
            .listener_votes
            .into_iter()
            .map(|(listener_did, attestation_id)| {
                ListenerAttestation::new(&listener_did, &attestation_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let listener_input = ListenerQuorumInput::new(
            &input.listener_event_id,
            input.listener_event_sequence,
            listener_attestations,
        )?;
        let listener_decision = self.listener_evaluator.evaluate(listener_input)?;

        let approver_attestations = input
            .approver_votes
            .into_iter()
            .map(|(approver_did, attestation_id, override_digest)| {
                let digest = override_digest.unwrap_or_else(|| payload_digest.clone());
                ApproverAttestation::new(&approver_did, &digest, &attestation_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let approver_input = ApproverQuorumInput::new(
            &input.outbound_action_id,
            &payload_digest,
            approver_attestations,
        )?;
        let approver_evaluator = ApproverQuorumEvaluator::new(self.approver_required_approvals)?;
        let approver_decision = approver_evaluator.authorize(approver_input)?;

        let block = self.network.produce_block()?;

        Ok(BlockPipelineCommitReport {
            block,
            listener_decision,
            approver_decision,
            payload_digest,
        })
    }
}

fn payload_digest_for_transactions(transactions: &[BaselineTransaction]) -> String {
    let mut ordered = transactions.to_vec();
    ordered.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.sender.cmp(&right.sender))
    });

    let mut digest = String::from("block-payload");
    for tx in ordered {
        digest.push('|');
        digest.push_str(&tx.id);
        digest.push(':');
        digest.push_str(&tx.sender);
        digest.push(':');
        digest.push_str(&tx.nonce.to_string());
        digest.push(':');
        digest.push_str(&tx.state_hash);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::{
        payload_digest_for_transactions, BlockPipelineError, CanonicalCommitRecord,
        DeterministicCompetingBranchForkChoiceHook, ForkChoiceDecision, ForkChoiceHook,
        MempoolBlockPipeline,
    };
    use crate::config::NodeRole;
    use crate::transaction::BaselineTransaction;

    #[test]
    fn constructor_rejects_zero_listener_quorum_threshold() {
        let result = MempoolBlockPipeline::new(true, 0, 1);
        assert!(matches!(result, Err(BlockPipelineError::Listener(_))));
    }

    #[test]
    fn constructor_rejects_zero_approver_quorum_threshold() {
        let result = MempoolBlockPipeline::new(true, 1, 0);
        assert!(matches!(result, Err(BlockPipelineError::Approver(_))));
    }

    #[test]
    fn regression_consensus_round_rejects_empty_mempool() {
        // Regression: #2927
        let mut pipeline = MempoolBlockPipeline::new(true, 1, 1).expect("pipeline builds");
        let result = pipeline.run_consensus_round(super::BlockConsensusRoundInput {
            listener_event_id: "event-1".to_owned(),
            listener_event_sequence: 1,
            outbound_action_id: "outbound-1".to_owned(),
            listener_votes: vec![("kamn:did:listener:alpha".to_owned(), "att-1".to_owned())],
            approver_votes: vec![(
                "kamn:did:agent:approver-alpha".to_owned(),
                "att-1".to_owned(),
                None,
            )],
        });
        assert_eq!(result, Err(BlockPipelineError::EmptyMempool));
    }

    #[test]
    fn payload_digest_is_deterministic_across_orderings() {
        let tx1 = BaselineTransaction::signed("tx-1", "agent-a", 1, "p1", "state:genesis");
        let tx2 = BaselineTransaction::signed("tx-2", "agent-b", 1, "p2", "state:genesis");
        let digest_a = payload_digest_for_transactions(&[tx1.clone(), tx2.clone()]);
        let digest_b = payload_digest_for_transactions(&[tx2, tx1]);
        assert_eq!(digest_a, digest_b);
    }

    #[test]
    fn deterministic_competing_branch_hook_rejects_stale_candidate_height() {
        let seeded_head = CanonicalCommitRecord {
            block_height: 8,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-z".to_owned(),
            transaction_ids: vec!["tx-z".to_owned()],
        };
        let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(seeded_head);
        let stale_candidate = CanonicalCommitRecord {
            block_height: 7,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-a".to_owned(),
            transaction_ids: vec!["tx-a".to_owned()],
        };

        let decision = hook
            .evaluate_candidate(&stale_candidate)
            .expect("hook should evaluate stale candidate");
        assert_eq!(
            decision,
            ForkChoiceDecision::Reject {
                reason_code: "fork_choice_stale_block_height".to_owned(),
            }
        );
        assert_eq!(
            hook.canonical_head()
                .expect("seeded canonical head should remain set")
                .payload_digest,
            "digest-z"
        );
    }

    #[test]
    fn deterministic_competing_branch_hook_prefers_lexicographically_lower_digest_on_tie() {
        let mut hook = DeterministicCompetingBranchForkChoiceHook::new();
        let branch_high = CanonicalCommitRecord {
            block_height: 5,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-b".to_owned(),
            transaction_ids: vec!["tx-b".to_owned()],
        };
        let branch_low = CanonicalCommitRecord {
            block_height: 5,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-a".to_owned(),
            transaction_ids: vec!["tx-a".to_owned()],
        };

        let first = hook
            .evaluate_candidate(&branch_high)
            .expect("first branch should evaluate");
        let second = hook
            .evaluate_candidate(&branch_low)
            .expect("second branch should evaluate");

        assert_eq!(first, ForkChoiceDecision::Accept);
        assert_eq!(second, ForkChoiceDecision::Accept);
        assert_eq!(
            hook.canonical_head()
                .expect("head should be selected after tie break")
                .payload_digest,
            "digest-a"
        );
    }

    #[test]
    fn deterministic_competing_branch_hook_rejects_duplicate_candidate() {
        let mut hook = DeterministicCompetingBranchForkChoiceHook::new();
        let candidate = CanonicalCommitRecord {
            block_height: 11,
            producer_role: NodeRole::Processor,
            payload_digest: "digest-11".to_owned(),
            transaction_ids: vec!["tx-11".to_owned()],
        };

        let first = hook
            .evaluate_candidate(&candidate)
            .expect("first candidate should evaluate");
        let second = hook
            .evaluate_candidate(&candidate)
            .expect("duplicate candidate should evaluate");

        assert_eq!(first, ForkChoiceDecision::Accept);
        assert_eq!(
            second,
            ForkChoiceDecision::Reject {
                reason_code: "fork_choice_duplicate_candidate".to_owned(),
            }
        );
    }

    #[test]
    fn block_pipeline_error_reason_code_extracts_commit_store_marker() {
        let error = BlockPipelineError::CommitStore(
            "commit store read failed (canonical_commit_store_io)".to_owned(),
        );
        assert_eq!(error.reason_code(), "canonical_commit_store_io");
    }

    #[test]
    fn block_pipeline_error_reason_code_uses_stable_fallback_when_marker_missing() {
        let error = BlockPipelineError::CommitStore("opaque commit store failure".to_owned());
        assert_eq!(error.reason_code(), "block_pipeline_commit_store_error");
    }
}
