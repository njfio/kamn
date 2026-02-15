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
use std::error::Error;
use std::fmt::{Display, Formatter};

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
        }
    }
}

impl Error for BlockPipelineError {}

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

const TRANSPORT_CANDIDATE_PAYLOAD_CODEC_VERSION: &str = "txwire:v1";
const TRANSPORT_CANDIDATE_PAYLOAD_FIELD_COUNT: usize = 7;
const TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR: char = '|';

/// Encodes a baseline transaction into deterministic transport-candidate wire payload.
pub fn encode_transport_candidate_payload(
    tx: &BaselineTransaction,
) -> Result<String, BlockPipelineError> {
    validate_transport_candidate_field("id", tx.id.as_str())?;
    validate_transport_candidate_field("sender", tx.sender.as_str())?;
    validate_transport_candidate_field("payload", tx.payload.as_str())?;
    validate_transport_candidate_field("state_hash", tx.state_hash.as_str())?;
    validate_transport_candidate_field("signature", tx.signature.as_str())?;
    if tx.nonce == 0 {
        return Err(BlockPipelineError::TransportFeed(
            "transport candidate nonce must be positive (transport_candidate_nonce_invalid)"
                .to_owned(),
        ));
    }

    Ok(format!(
        "{TRANSPORT_CANDIDATE_PAYLOAD_CODEC_VERSION}{TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR}{}{TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR}{}{TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR}{}{TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR}{}{TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR}{}{TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR}{}",
        tx.id, tx.sender, tx.nonce, tx.payload, tx.state_hash, tx.signature
    ))
}

/// Decodes deterministic transport-candidate wire payload into baseline transaction.
pub fn decode_transport_candidate_payload(
    payload: &str,
) -> Result<BaselineTransaction, BlockPipelineError> {
    let fields = payload
        .split(TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR)
        .collect::<Vec<_>>();
    if fields.len() != TRANSPORT_CANDIDATE_PAYLOAD_FIELD_COUNT {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate payload malformed: expected {} fields, found {} (transport_candidate_payload_malformed)",
            TRANSPORT_CANDIDATE_PAYLOAD_FIELD_COUNT,
            fields.len()
        )));
    }
    if fields[0] != TRANSPORT_CANDIDATE_PAYLOAD_CODEC_VERSION {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate codec version mismatch: expected {TRANSPORT_CANDIDATE_PAYLOAD_CODEC_VERSION}, found {} (transport_candidate_codec_version_mismatch)",
            fields[0]
        )));
    }

    let id = fields[1];
    let sender = fields[2];
    let nonce = fields[3].parse::<u64>().map_err(|_| {
        BlockPipelineError::TransportFeed(format!(
            "transport candidate nonce is invalid: {} (transport_candidate_nonce_invalid)",
            fields[3]
        ))
    })?;
    let payload = fields[4];
    let state_hash = fields[5];
    let signature = fields[6];

    validate_transport_candidate_field("id", id)?;
    validate_transport_candidate_field("sender", sender)?;
    validate_transport_candidate_field("payload", payload)?;
    validate_transport_candidate_field("state_hash", state_hash)?;
    validate_transport_candidate_field("signature", signature)?;
    if nonce == 0 {
        return Err(BlockPipelineError::TransportFeed(
            "transport candidate nonce must be positive (transport_candidate_nonce_invalid)"
                .to_owned(),
        ));
    }

    Ok(BaselineTransaction {
        id: id.to_owned(),
        sender: sender.to_owned(),
        nonce,
        payload: payload.to_owned(),
        state_hash: state_hash.to_owned(),
        signature: signature.to_owned(),
    })
}

fn validate_transport_candidate_field(label: &str, value: &str) -> Result<(), BlockPipelineError> {
    if value.trim().is_empty() {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate {label} is empty (transport_candidate_field_empty)"
        )));
    }
    if value.contains(TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR) {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate {label} contains reserved separator '{}' (transport_candidate_field_separator_conflict)",
            TRANSPORT_CANDIDATE_PAYLOAD_SEPARATOR
        )));
    }
    Ok(())
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

/// Transport-backed mempool feed that drains inbound gossip frames for one local peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportEventMempoolFeed<TTransport> {
    transport: TTransport,
    local_peer_id: String,
    required_topic: Option<String>,
}

impl<TTransport> TransportEventMempoolFeed<TTransport> {
    /// Creates a transport-event feed bound to one local peer inbox.
    pub fn new(
        transport: TTransport,
        local_peer_id: &str,
        required_topic: Option<&str>,
    ) -> Result<Self, BlockPipelineError> {
        if local_peer_id.trim().is_empty() {
            return Err(BlockPipelineError::TransportFeed(
                "transport feed local peer id is empty (transport_feed_local_peer_id_invalid)"
                    .to_owned(),
            ));
        }
        let required_topic = match required_topic {
            Some(topic) if topic.trim().is_empty() => {
                return Err(BlockPipelineError::TransportFeed(
                    "transport feed required topic is empty (transport_feed_topic_invalid)"
                        .to_owned(),
                ));
            }
            Some(topic) => Some(topic.to_owned()),
            None => None,
        };
        Ok(Self {
            transport,
            local_peer_id: local_peer_id.to_owned(),
            required_topic,
        })
    }

    fn decode_frame(
        &self,
        frame: PeerGossipFrame,
    ) -> Result<BaselineTransaction, BlockPipelineError> {
        if let Some(required_topic) = self.required_topic.as_deref() {
            if frame.topic != required_topic {
                return Err(BlockPipelineError::TransportFeed(format!(
                    "transport frame topic mismatch: expected {required_topic}, found {} (transport_candidate_topic_mismatch)",
                    frame.topic
                )));
            }
        }
        decode_transport_candidate_payload(frame.payload.as_str())
    }
}

impl<TTransport> TransportMempoolFeed for TransportEventMempoolFeed<TTransport>
where
    TTransport: PeerLifecycleTransport,
{
    fn drain_pending_transactions(
        &mut self,
    ) -> Result<Vec<BaselineTransaction>, BlockPipelineError> {
        let frames = self
            .transport
            .drain_inbox(self.local_peer_id.as_str())
            .map_err(|error| {
                BlockPipelineError::TransportFeed(format!(
                    "transport feed inbox drain failed: {error} (transport_feed_inbox_drain_failed)"
                ))
            })?;
        let mut transactions = Vec::with_capacity(frames.len());
        for frame in frames {
            transactions.push(self.decode_frame(frame)?);
        }
        Ok(transactions)
    }
}

/// Canonical commit persistence interface used by transport-fed block pipeline.
pub trait CanonicalCommitStore {
    /// Persists canonical commit record after fork-choice acceptance.
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError>;

    /// Lists canonical commit records in persistence order.
    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory canonical commit store for deterministic tests and local runtime probes.
pub struct InMemoryCanonicalCommitStore {
    records: Vec<CanonicalCommitRecord>,
}

impl CanonicalCommitStore for InMemoryCanonicalCommitStore {
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError> {
        self.records.push(record);
        Ok(())
    }

    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        Ok(self.records.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Fork-choice decision for canonical candidate commit.
pub enum ForkChoiceDecision {
    /// Candidate is accepted as canonical.
    Accept,
    /// Candidate is rejected with deterministic reason code.
    Reject {
        /// Deterministic rejection reason code.
        reason_code: String,
    },
}

/// Fork-choice evaluation hook for candidate canonical commit records.
pub trait ForkChoiceHook {
    /// Evaluates candidate canonical commit and returns deterministic decision.
    fn evaluate_candidate(
        &mut self,
        record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Fork-choice hook that accepts every candidate.
pub struct AcceptAllForkChoiceHook;

impl ForkChoiceHook for AcceptAllForkChoiceHook {
    fn evaluate_candidate(
        &mut self,
        _record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError> {
        Ok(ForkChoiceDecision::Accept)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Deterministic fork-choice hook for competing branch candidates.
///
/// Selection rules:
/// - Higher `block_height` always wins.
/// - At equal `block_height`, lexicographically lower `payload_digest` wins.
/// - Identical height+digest is treated as duplicate candidate and rejected.
pub struct DeterministicCompetingBranchForkChoiceHook {
    canonical_head: Option<CanonicalCommitRecord>,
}

impl DeterministicCompetingBranchForkChoiceHook {
    /// Creates deterministic competing-branch hook with empty head.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates deterministic competing-branch hook seeded with existing canonical head.
    pub fn with_canonical_head(canonical_head: CanonicalCommitRecord) -> Self {
        Self {
            canonical_head: Some(canonical_head),
        }
    }

    /// Returns currently selected canonical head, if any.
    pub fn canonical_head(&self) -> Option<&CanonicalCommitRecord> {
        self.canonical_head.as_ref()
    }
}

impl ForkChoiceHook for DeterministicCompetingBranchForkChoiceHook {
    fn evaluate_candidate(
        &mut self,
        record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError> {
        let Some(head) = self.canonical_head.as_ref() else {
            self.canonical_head = Some(record.clone());
            return Ok(ForkChoiceDecision::Accept);
        };

        if record.block_height > head.block_height {
            self.canonical_head = Some(record.clone());
            return Ok(ForkChoiceDecision::Accept);
        }
        if record.block_height < head.block_height {
            return Ok(ForkChoiceDecision::Reject {
                reason_code: "fork_choice_stale_block_height".to_owned(),
            });
        }

        if record.payload_digest == head.payload_digest {
            return Ok(ForkChoiceDecision::Reject {
                reason_code: "fork_choice_duplicate_candidate".to_owned(),
            });
        }

        if record.payload_digest < head.payload_digest {
            self.canonical_head = Some(record.clone());
            return Ok(ForkChoiceDecision::Accept);
        }

        Ok(ForkChoiceDecision::Reject {
            reason_code: "fork_choice_tie_break_loser".to_owned(),
        })
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
    TFeed: TransportMempoolFeed,
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

    /// Runs one transport-fed consensus round and persists canonical commit record.
    pub fn run_transport_consensus_round(
        &mut self,
        input: BlockConsensusRoundInput,
    ) -> Result<BlockPipelineCommitReport, BlockPipelineError> {
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
}
