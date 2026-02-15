//! Mempool block production and consensus validation pipeline contracts.

use crate::config::NodeRole;
use crate::p2p_transport::{PeerGossipFrame, PeerLifecycleTransport};
use crate::runtime::{
    ApproverAttestation, ApproverQuorumDecision, ApproverQuorumError, ApproverQuorumEvaluator,
    ApproverQuorumInput, ListenerAttestation, ListenerQuorumDecision, ListenerQuorumError,
    ListenerQuorumEvaluator, ListenerQuorumInput,
};
use crate::smoke::{ProducedBlock, RoleSmokeNetwork, SmokeError};
use crate::sqlite_store_backend::{SqliteStoreBackend, SqliteStoreBackendError};
use crate::transaction::BaselineTransaction;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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

const TOPIC_MESSAGES_LEGACY: &str = "messages";
const TOPIC_MESSAGES_V1: &str = "kamn/messages/v1";
const TOPIC_BLOCKS_LEGACY: &str = "blocks";
const TOPIC_BLOCKS_V1: &str = "kamn/blocks/v1";

/// Encodes a baseline transaction into deterministic transport-candidate wire payload.
pub fn encode_transport_candidate_payload(
    tx: &BaselineTransaction,
) -> Result<String, BlockPipelineError> {
    validate_transport_payload_field_value("id", tx.id.as_str())?;
    validate_transport_payload_field_value("sender", tx.sender.as_str())?;
    validate_transport_payload_field_value("state_hash", tx.state_hash.as_str())?;
    validate_transport_payload_field_value("payload", tx.payload.as_str())?;
    validate_transport_payload_field_value("signature", tx.signature.as_str())?;
    if tx.nonce == 0 {
        return Err(BlockPipelineError::TransportFeed(
            "transport candidate nonce must be positive (transport_candidate_nonce_invalid)"
                .to_owned(),
        ));
    }

    Ok(format!(
        "id={}\nsender={}\nnonce={}\nstate_hash={}\npayload={}\nsignature={}",
        tx.id, tx.sender, tx.nonce, tx.state_hash, tx.payload, tx.signature
    ))
}

/// Decodes deterministic transport-candidate wire payload into baseline transaction.
pub fn decode_transport_candidate_payload(
    payload: &str,
) -> Result<BaselineTransaction, BlockPipelineError> {
    let frame = PeerGossipFrame {
        topic: TOPIC_MESSAGES_V1.to_owned(),
        sender_peer_id: "transport-candidate-decode-source".to_owned(),
        recipient_peer_id: "transport-candidate-decode-target".to_owned(),
        payload: payload.to_owned(),
    };
    match GossipIngressAdapter::decode_frame(&frame) {
        Ok(GossipIngressRecord::Transaction(tx)) => Ok(tx),
        Ok(GossipIngressRecord::BlockCandidate(_)) => Err(BlockPipelineError::TransportFeed(
            "transport candidate decode yielded block candidate payload (transport_candidate_payload_kind_invalid)"
                .to_owned(),
        )),
        Err(error) => Err(BlockPipelineError::TransportFeed(format!(
            "{}:{}",
            error.reason_code(),
            error
        ))),
    }
}

/// Encodes canonical commit candidate into deterministic transport block payload.
pub fn encode_transport_canonical_candidate_payload(
    record: &CanonicalCommitRecord,
) -> Result<String, BlockPipelineError> {
    if record.block_height == 0 {
        return Err(BlockPipelineError::TransportFeed(
            "transport canonical candidate block_height must be positive (transport_candidate_block_height_invalid)"
                .to_owned(),
        ));
    }
    validate_transport_payload_field_value("payload_digest", record.payload_digest.as_str())?;
    if record.transaction_ids.is_empty() {
        return Err(BlockPipelineError::TransportFeed(
            "transport canonical candidate transaction_ids must not be empty (transport_candidate_transaction_ids_invalid)"
                .to_owned(),
        ));
    }
    let mut seen_ids = BTreeSet::new();
    for tx_id in &record.transaction_ids {
        validate_transport_payload_field_value("transaction_id", tx_id.as_str())?;
        if tx_id.contains(',') {
            return Err(BlockPipelineError::TransportFeed(
                "transport canonical candidate transaction_id contains reserved separator ',' (transport_candidate_transaction_id_invalid)"
                    .to_owned(),
            ));
        }
        if !seen_ids.insert(tx_id) {
            return Err(BlockPipelineError::TransportFeed(
                "transport canonical candidate transaction_id is duplicated (transport_candidate_transaction_id_invalid)"
                    .to_owned(),
            ));
        }
    }
    let transaction_ids = record.transaction_ids.join(",");
    Ok(format!(
        "block_height={}\nproducer_role={}\npayload_digest={}\ntransaction_ids={}",
        record.block_height,
        record.producer_role.as_str(),
        record.payload_digest,
        transaction_ids
    ))
}

/// Decodes deterministic transport canonical-candidate payload into canonical commit record.
pub fn decode_transport_canonical_candidate_payload(
    payload: &str,
) -> Result<CanonicalCommitRecord, BlockPipelineError> {
    let frame = PeerGossipFrame {
        topic: TOPIC_BLOCKS_V1.to_owned(),
        sender_peer_id: "transport-candidate-decode-source".to_owned(),
        recipient_peer_id: "transport-candidate-decode-target".to_owned(),
        payload: payload.to_owned(),
    };
    match GossipIngressAdapter::decode_frame(&frame) {
        Ok(GossipIngressRecord::BlockCandidate(record)) => Ok(record),
        Ok(GossipIngressRecord::Transaction(_)) => Err(BlockPipelineError::TransportFeed(
            "transport canonical candidate decode yielded transaction payload (transport_candidate_payload_kind_invalid)"
                .to_owned(),
        )),
        Err(error) => Err(BlockPipelineError::TransportFeed(format!(
            "{}:{}",
            error.reason_code(),
            error
        ))),
    }
}

/// Encodes committed consensus round report into deterministic transport canonical-candidate payload.
pub fn encode_transport_commit_report_payload(
    report: &BlockPipelineCommitReport,
) -> Result<String, BlockPipelineError> {
    let canonical_record = CanonicalCommitRecord::from_commit_report(report);
    encode_transport_canonical_candidate_payload(&canonical_record)
}

fn validate_transport_payload_field_value(
    field: &str,
    value: &str,
) -> Result<(), BlockPipelineError> {
    if value.trim().is_empty() {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate field is empty: {field} (transport_candidate_field_empty)"
        )));
    }
    if value.contains('\n') || value.contains('\r') {
        return Err(BlockPipelineError::TransportFeed(format!(
            "transport candidate field contains line break: {field} (transport_candidate_field_line_break)"
        )));
    }
    Ok(())
}

/// Decoded gossip ingress record classified by payload intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GossipIngressRecord {
    /// Transaction payload normalized for mempool ingress.
    Transaction(BaselineTransaction),
    /// Canonical block candidate normalized for fork-choice/persistence paths.
    BlockCandidate(CanonicalCommitRecord),
}

impl GossipIngressRecord {
    /// Returns transaction payload when record classification is `Transaction`.
    pub fn into_transaction(self) -> Option<BaselineTransaction> {
        match self {
            Self::Transaction(tx) => Some(tx),
            Self::BlockCandidate(_) => None,
        }
    }

    /// Returns block candidate payload when record classification is `BlockCandidate`.
    pub fn into_block_candidate(self) -> Option<CanonicalCommitRecord> {
        match self {
            Self::Transaction(_) => None,
            Self::BlockCandidate(record) => Some(record),
        }
    }
}

/// Batch decode output for transaction and canonical block candidate payloads.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GossipIngressBatch {
    /// Decoded transaction payloads in input order.
    pub transactions: Vec<BaselineTransaction>,
    /// Decoded canonical block candidates in input order.
    pub canonical_candidates: Vec<CanonicalCommitRecord>,
}

/// Deterministic decode failure for gossip ingress payload normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipIngressError {
    reason_code: &'static str,
    detail: String,
}

impl GossipIngressError {
    fn new(reason_code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            reason_code,
            detail: detail.into(),
        }
    }

    /// Returns deterministic reason code for fail-closed policy checks.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl Display for GossipIngressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.detail, self.reason_code)
    }
}

impl Error for GossipIngressError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GossipIngressTopicKind {
    Transaction,
    Block,
}

/// Deterministic topic+payload ingress adapter for transport-fed pipeline paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GossipIngressAdapter;

impl GossipIngressAdapter {
    /// Decodes one frame into normalized transaction or canonical block candidate record.
    pub fn decode_frame(
        frame: &PeerGossipFrame,
    ) -> Result<GossipIngressRecord, GossipIngressError> {
        let topic_kind = classify_gossip_topic(frame.topic.as_str())?;
        let payload_fields = parse_payload_fields(frame.payload.as_str())?;

        match topic_kind {
            GossipIngressTopicKind::Transaction => decode_transaction_record(&payload_fields),
            GossipIngressTopicKind::Block => decode_block_candidate_record(&payload_fields),
        }
    }

    /// Decodes many frames into transaction and block candidate batches.
    pub fn decode_frames(
        frames: &[PeerGossipFrame],
    ) -> Result<GossipIngressBatch, GossipIngressError> {
        let mut batch = GossipIngressBatch::default();
        for frame in frames {
            match Self::decode_frame(frame)? {
                GossipIngressRecord::Transaction(tx) => batch.transactions.push(tx),
                GossipIngressRecord::BlockCandidate(record) => {
                    batch.canonical_candidates.push(record);
                }
            }
        }
        Ok(batch)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Restart/replay evidence bundle for canonical commit lineage continuity.
pub struct CanonicalReplayEvidenceBundle {
    /// Versioned schema marker for policy and contract-lane checks.
    pub schema_version: String,
    /// Canonical head block height at restart boundary.
    pub restart_boundary_block_height: u64,
    /// Canonical head block height after replay checkpoint validation.
    pub replay_checkpoint_block_height: u64,
    /// Canonical commit count captured before restart.
    pub pre_restart_commit_count: usize,
    /// Canonical commit count observed after restart/replay.
    pub post_restart_commit_count: usize,
    /// Deterministic continuity status marker (`verified` on success).
    pub continuity_status: String,
}

/// Deterministic decision outcome for one transport candidate reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalCandidateDecision {
    /// Candidate was selected as canonical and persisted.
    Accepted,
    /// Candidate was rejected by fork-choice with deterministic reason code.
    Rejected {
        /// Deterministic fork-choice rejection reason code.
        reason_code: String,
    },
}

/// Reconciliation report for one transport-provided canonical candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCandidateOutcome {
    /// Candidate block height.
    pub block_height: u64,
    /// Candidate payload digest.
    pub payload_digest: String,
    /// Deterministic reconciliation decision.
    pub decision: CanonicalCandidateDecision,
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
/// File-backed canonical commit store for restart/replay persistence checks.
pub struct FileCanonicalCommitStore {
    path: PathBuf,
}

impl FileCanonicalCommitStore {
    /// Creates file-backed canonical commit store from path.
    pub fn new(path: PathBuf) -> Result<Self, BlockPipelineError> {
        if path.as_os_str().is_empty() {
            return Err(BlockPipelineError::CommitStore(
                "canonical commit store path is empty (canonical_commit_store_path_invalid)"
                    .to_owned(),
            ));
        }
        Ok(Self { path })
    }
}

impl CanonicalCommitStore for FileCanonicalCommitStore {
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError> {
        let existing = self.list_canonical_commits()?;
        if let Some(last) = existing.last() {
            if record.block_height <= last.block_height {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit block height regression: previous {}, found {} (canonical_commit_store_block_height_regression)",
                    last.block_height, record.block_height
                )));
            }
        }

        let serialized = serialize_canonical_commit_record(&record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| {
                BlockPipelineError::CommitStore(format!(
                    "canonical commit store append failed: {error} (canonical_commit_store_io)"
                ))
            })?;
        file.write_all(serialized.as_bytes()).map_err(|error| {
            BlockPipelineError::CommitStore(format!(
                "canonical commit store write failed: {error} (canonical_commit_store_io)"
            ))
        })
    }

    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let payload = fs::read_to_string(&self.path).map_err(|error| {
            BlockPipelineError::CommitStore(format!(
                "canonical commit store read failed: {error} (canonical_commit_store_io)"
            ))
        })?;
        let mut records: Vec<CanonicalCommitRecord> = Vec::new();
        for raw_line in payload.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            let record = parse_canonical_commit_record(line)?;
            if let Some(previous) = records.last() {
                if record.block_height <= previous.block_height {
                    return Err(BlockPipelineError::CommitStore(format!(
                        "canonical commit block height regression in persisted lineage: previous {}, found {} (canonical_commit_store_block_height_regression)",
                        previous.block_height, record.block_height
                    )));
                }
            }
            records.push(record);
        }
        Ok(records)
    }
}

const CANONICAL_COMMIT_SQLITE_META_NAMESPACE: &str = "canonical_commit_store_meta";
const CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE: &str = "canonical_commit_store_entries";
const CANONICAL_COMMIT_SQLITE_SCHEMA_KEY: &str = "schema_version";
const CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
/// Sqlite-backed canonical commit store with strict schema/version guards.
pub struct SqliteCanonicalCommitStore {
    backend: SqliteStoreBackend,
}

impl SqliteCanonicalCommitStore {
    /// Creates sqlite-backed canonical commit store from path.
    pub fn new(path: PathBuf) -> Result<Self, BlockPipelineError> {
        let backend =
            SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_commit_store_error)?;
        let mut store = Self { backend };
        store.bootstrap_and_validate_schema_version()?;
        Ok(store)
    }

    fn bootstrap_and_validate_schema_version(&mut self) -> Result<(), BlockPipelineError> {
        let current = self
            .backend
            .get(
                CANONICAL_COMMIT_SQLITE_META_NAMESPACE,
                CANONICAL_COMMIT_SQLITE_SCHEMA_KEY,
            )
            .map_err(map_sqlite_commit_store_error)?;

        if let Some(bytes) = current {
            let schema_raw = String::from_utf8(bytes).map_err(|_| {
                BlockPipelineError::CommitStore(
                    "canonical commit sqlite schema value is not utf-8 (canonical_commit_store_sqlite_schema_invalid)"
                        .to_owned(),
                )
            })?;
            let found = schema_raw.parse::<u32>().map_err(|_| {
                BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite schema value is invalid: {schema_raw} (canonical_commit_store_sqlite_schema_invalid)"
                ))
            })?;
            if found != CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite schema mismatch: expected {}, found {} (canonical_commit_store_sqlite_schema_mismatch)",
                    CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION, found
                )));
            }
            return Ok(());
        }

        let existing_keys = self
            .backend
            .list_keys(CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE)
            .map_err(map_sqlite_commit_store_error)?;
        if !existing_keys.is_empty() {
            return Err(BlockPipelineError::CommitStore(
                "canonical commit sqlite schema row missing with existing commit entries (canonical_commit_store_sqlite_schema_missing)"
                    .to_owned(),
            ));
        }

        self.backend
            .put(
                CANONICAL_COMMIT_SQLITE_META_NAMESPACE,
                CANONICAL_COMMIT_SQLITE_SCHEMA_KEY,
                CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION
                    .to_string()
                    .as_bytes(),
            )
            .map_err(map_sqlite_commit_store_error)
    }
}

impl CanonicalCommitStore for SqliteCanonicalCommitStore {
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError> {
        let existing = self.list_canonical_commits()?;
        if let Some(last) = existing.last() {
            if record.block_height <= last.block_height {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit block height regression: previous {}, found {} (canonical_commit_store_block_height_regression)",
                    last.block_height, record.block_height
                )));
            }
        }

        let key = sqlite_canonical_commit_store_key(record.block_height);
        let payload = serialize_canonical_commit_record(&record)?;
        self.backend
            .put(
                CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE,
                key.as_str(),
                payload.as_bytes(),
            )
            .map_err(map_sqlite_commit_store_error)?;
        Ok(())
    }

    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        let mut records: Vec<CanonicalCommitRecord> = Vec::new();
        let keys = self
            .backend
            .list_keys(CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE)
            .map_err(map_sqlite_commit_store_error)?;

        for key in keys {
            let key_height = parse_sqlite_canonical_commit_store_key(&key)?;
            let payload_bytes = self
                .backend
                .get(CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE, key.as_str())
                .map_err(map_sqlite_commit_store_error)?
                .ok_or_else(|| {
                    BlockPipelineError::CommitStore(format!(
                        "canonical commit sqlite row missing for key {key} (canonical_commit_store_sqlite_missing_entry)"
                    ))
                })?;
            let payload = String::from_utf8(payload_bytes).map_err(|_| {
                BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite payload is not utf-8 for key {key} (canonical_commit_store_sqlite_payload_not_utf8)"
                ))
            })?;
            if payload.trim().is_empty() {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite payload is empty for key {key} (canonical_commit_store_sqlite_payload_empty)"
                )));
            }
            let record = parse_canonical_commit_record(&payload)?;
            if record.block_height != key_height {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite key height mismatch: key {}, payload {} (canonical_commit_store_sqlite_key_height_mismatch)",
                    key_height, record.block_height
                )));
            }
            if let Some(previous) = records.last() {
                if record.block_height <= previous.block_height {
                    return Err(BlockPipelineError::CommitStore(format!(
                        "canonical commit block height regression in persisted lineage: previous {}, found {} (canonical_commit_store_block_height_regression)",
                        previous.block_height, record.block_height
                    )));
                }
            }
            records.push(record);
        }
        Ok(records)
    }
}

fn sqlite_canonical_commit_store_key(block_height: u64) -> String {
    format!("height:{block_height:020}")
}

fn parse_sqlite_canonical_commit_store_key(key: &str) -> Result<u64, BlockPipelineError> {
    let Some(height_raw) = key.strip_prefix("height:") else {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite key is malformed: {key} (canonical_commit_store_sqlite_key_malformed)"
        )));
    };
    if height_raw.len() != 20
        || !height_raw
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite key is malformed: {key} (canonical_commit_store_sqlite_key_malformed)"
        )));
    }
    height_raw.parse::<u64>().map_err(|_| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite key is malformed: {key} (canonical_commit_store_sqlite_key_malformed)"
        ))
    })
}

fn map_sqlite_commit_store_error(error: SqliteStoreBackendError) -> BlockPipelineError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => BlockPipelineError::CommitStore(
            "canonical commit sqlite backend schema missing (canonical_commit_store_sqlite_backend_schema_missing)"
                .to_owned(),
        ),
        SqliteStoreBackendError::SchemaVersionInvalid(value) => BlockPipelineError::CommitStore(
            format!(
                "canonical commit sqlite backend schema invalid: {value} (canonical_commit_store_sqlite_backend_schema_invalid)"
            ),
        ),
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            BlockPipelineError::CommitStore(format!(
                "canonical commit sqlite backend schema mismatch: expected {expected}, found {found} (canonical_commit_store_sqlite_backend_schema_mismatch)"
            ))
        }
        SqliteStoreBackendError::InvalidPath => BlockPipelineError::CommitStore(
            "canonical commit sqlite path is invalid (canonical_commit_store_path_invalid)"
                .to_owned(),
        ),
        other => BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite backend operation failed: {other} (canonical_commit_store_io)"
        )),
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

const CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION: &str = "kamn.runtime.canonical-replay-evidence.v1";

/// Validates canonical lineage continuity across restart/replay checkpoints.
pub fn build_canonical_replay_evidence_bundle(
    pre_restart: &[CanonicalCommitRecord],
    post_restart: &[CanonicalCommitRecord],
) -> Result<CanonicalReplayEvidenceBundle, BlockPipelineError> {
    let Some(restart_boundary) = pre_restart.last() else {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_pre_restart_lineage_empty".to_owned(),
            detail: "pre-restart canonical lineage cannot be empty".to_owned(),
        });
    };

    for (index, expected) in pre_restart.iter().enumerate() {
        let Some(found) = post_restart.get(index) else {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_checkpoint_missing".to_owned(),
                detail: format!(
                    "post-restart lineage missing checkpoint at index {index} (expected height {})",
                    expected.block_height
                ),
            });
        };

        if found.block_height != expected.block_height {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_block_height_mismatch".to_owned(),
                detail: format!(
                    "canonical replay block height mismatch at index {index}: expected {}, found {}",
                    expected.block_height, found.block_height
                ),
            });
        }
        if found.producer_role != expected.producer_role {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_producer_role_mismatch".to_owned(),
                detail: format!(
                    "canonical replay producer role mismatch at index {index}: expected {}, found {}",
                    expected.producer_role.as_str(),
                    found.producer_role.as_str()
                ),
            });
        }
        if found.payload_digest != expected.payload_digest {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_payload_digest_mismatch".to_owned(),
                detail: format!(
                    "canonical replay payload digest mismatch at index {index}: expected {}, found {}",
                    expected.payload_digest, found.payload_digest
                ),
            });
        }
        if found.transaction_ids != expected.transaction_ids {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_transaction_ids_mismatch".to_owned(),
                detail: format!(
                    "canonical replay transaction ids mismatch at index {index}: expected {:?}, found {:?}",
                    expected.transaction_ids, found.transaction_ids
                ),
            });
        }
    }

    let replay_checkpoint = post_restart
        .get(pre_restart.len().saturating_sub(1))
        .ok_or_else(|| BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_checkpoint_missing".to_owned(),
            detail: "post-restart lineage missing replay checkpoint".to_owned(),
        })?;

    Ok(CanonicalReplayEvidenceBundle {
        schema_version: CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION.to_owned(),
        restart_boundary_block_height: restart_boundary.block_height,
        replay_checkpoint_block_height: replay_checkpoint.block_height,
        pre_restart_commit_count: pre_restart.len(),
        post_restart_commit_count: post_restart.len(),
        continuity_status: "verified".to_owned(),
    })
}

fn serialize_canonical_commit_record(
    record: &CanonicalCommitRecord,
) -> Result<String, BlockPipelineError> {
    if record.block_height == 0 {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit block height must be positive (canonical_commit_store_block_height_invalid)"
                .to_owned(),
        ));
    }
    if record.transaction_ids.is_empty() {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit transaction ids cannot be empty (canonical_commit_store_transaction_ids_invalid)"
                .to_owned(),
        ));
    }
    validate_canonical_commit_store_field("payload_digest", record.payload_digest.as_str())?;
    let mut encoded_ids = Vec::with_capacity(record.transaction_ids.len());
    for tx_id in &record.transaction_ids {
        validate_canonical_commit_store_field("transaction_id", tx_id.as_str())?;
        if tx_id.contains(',') {
            return Err(BlockPipelineError::CommitStore(
                "canonical commit transaction id cannot contain ',' (canonical_commit_store_transaction_ids_invalid)"
                    .to_owned(),
            ));
        }
        encoded_ids.push(tx_id.as_str());
    }

    Ok(format!(
        "{}|{}|{}|{}\n",
        record.block_height,
        record.producer_role.as_str(),
        record.payload_digest,
        encoded_ids.join(",")
    ))
}

fn parse_canonical_commit_record(line: &str) -> Result<CanonicalCommitRecord, BlockPipelineError> {
    let mut segments = line.split('|');
    let block_height_raw = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    let producer_role_raw = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    let payload_digest = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    let transaction_ids_raw = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    if segments.next().is_some() {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        )));
    }

    let block_height = block_height_raw.parse::<u64>().map_err(|_| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit block height is invalid: {block_height_raw} (canonical_commit_store_block_height_invalid)"
        ))
    })?;
    if block_height == 0 {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit block height must be positive (canonical_commit_store_block_height_invalid)"
                .to_owned(),
        ));
    }
    let producer_role = match producer_role_raw {
        "processor" => NodeRole::Processor,
        "listener" => NodeRole::Listener,
        "approver" => NodeRole::Approver,
        other => {
            return Err(BlockPipelineError::CommitStore(format!(
                "canonical commit producer role is invalid: {other} (canonical_commit_store_producer_role_invalid)"
            )));
        }
    };
    validate_canonical_commit_store_field("payload_digest", payload_digest)?;

    let mut seen_ids = BTreeSet::new();
    let mut transaction_ids = Vec::new();
    for tx_id in transaction_ids_raw.split(',').map(|value| value.trim()) {
        if tx_id.is_empty() {
            continue;
        }
        validate_canonical_commit_store_field("transaction_id", tx_id)?;
        if !seen_ids.insert(tx_id.to_owned()) {
            return Err(BlockPipelineError::CommitStore(format!(
                "canonical commit transaction id is duplicated: {tx_id} (canonical_commit_store_transaction_ids_invalid)"
            )));
        }
        transaction_ids.push(tx_id.to_owned());
    }
    if transaction_ids.is_empty() {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit transaction ids cannot be empty (canonical_commit_store_transaction_ids_invalid)"
                .to_owned(),
        ));
    }

    Ok(CanonicalCommitRecord {
        block_height,
        producer_role,
        payload_digest: payload_digest.to_owned(),
        transaction_ids,
    })
}

fn validate_canonical_commit_store_field(
    field: &str,
    value: &str,
) -> Result<(), BlockPipelineError> {
    if value.trim().is_empty() {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit field is empty: {field} (canonical_commit_store_field_empty)"
        )));
    }
    if value.contains('|') {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit field contains reserved separator '|': {field} (canonical_commit_store_field_separator_invalid)"
        )));
    }
    if value.contains('\n') || value.contains('\r') {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit field contains line break: {field} (canonical_commit_store_field_line_break)"
        )));
    }
    Ok(())
}

fn classify_gossip_topic(topic: &str) -> Result<GossipIngressTopicKind, GossipIngressError> {
    match topic.trim() {
        TOPIC_MESSAGES_LEGACY | TOPIC_MESSAGES_V1 => Ok(GossipIngressTopicKind::Transaction),
        TOPIC_BLOCKS_LEGACY | TOPIC_BLOCKS_V1 => Ok(GossipIngressTopicKind::Block),
        unsupported => Err(GossipIngressError::new(
            "p2p_ingress_topic_unsupported",
            format!("unsupported gossip topic for block pipeline ingress: {unsupported}"),
        )),
    }
}

fn parse_payload_fields(payload: &str) -> Result<BTreeMap<String, String>, GossipIngressError> {
    if payload.trim().is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_empty",
            "gossip ingress payload cannot be empty",
        ));
    }

    let mut fields = BTreeMap::new();
    for raw_line in payload.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = line.split_once('=').ok_or_else(|| {
            GossipIngressError::new(
                "p2p_ingress_payload_line_malformed",
                format!("malformed key/value line: {line}"),
            )
        })?;
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(GossipIngressError::new(
                "p2p_ingress_payload_line_malformed",
                format!("payload key cannot be empty: {line}"),
            ));
        }
        if fields.contains_key(key) {
            return Err(GossipIngressError::new(
                "p2p_ingress_payload_duplicate_field",
                format!("duplicate payload field: {key}"),
            ));
        }
        fields.insert(key.to_owned(), raw_value.trim().to_owned());
    }

    if fields.is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_empty",
            "gossip ingress payload has no key/value fields",
        ));
    }

    Ok(fields)
}

fn required_payload_field<'a>(
    fields: &'a BTreeMap<String, String>,
    field: &'static str,
) -> Result<&'a str, GossipIngressError> {
    let value = fields.get(field).ok_or_else(|| {
        GossipIngressError::new(
            "p2p_ingress_payload_missing_field",
            format!("missing required payload field: {field}"),
        )
    })?;
    if value.trim().is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_missing_field",
            format!("required payload field is empty: {field}"),
        ));
    }
    Ok(value.as_str())
}

fn decode_transaction_record(
    fields: &BTreeMap<String, String>,
) -> Result<GossipIngressRecord, GossipIngressError> {
    let id = required_payload_field(fields, "id")?;
    let sender = required_payload_field(fields, "sender")?;
    let nonce_raw = required_payload_field(fields, "nonce")?;
    let state_hash = required_payload_field(fields, "state_hash")?;
    let payload = required_payload_field(fields, "payload")?;
    let signature = required_payload_field(fields, "signature")?;

    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        GossipIngressError::new(
            "p2p_ingress_payload_nonce_invalid",
            format!("nonce field must be positive integer, found: {nonce_raw}"),
        )
    })?;
    if nonce == 0 {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_nonce_invalid",
            "nonce field must be positive integer, found: 0",
        ));
    }

    let tx = BaselineTransaction {
        id: id.to_owned(),
        sender: sender.to_owned(),
        nonce,
        payload: payload.to_owned(),
        state_hash: state_hash.to_owned(),
        signature: signature.to_owned(),
    };
    if tx.signature != tx.expected_signature() {
        return Err(GossipIngressError::new(
            "p2p_ingress_tx_signature_invalid",
            format!(
                "transaction signature failed baseline profile validation for {}",
                tx.id
            ),
        ));
    }

    Ok(GossipIngressRecord::Transaction(tx))
}

fn decode_block_candidate_record(
    fields: &BTreeMap<String, String>,
) -> Result<GossipIngressRecord, GossipIngressError> {
    let block_height_raw = required_payload_field(fields, "block_height")?;
    let block_height = block_height_raw.parse::<u64>().map_err(|_| {
        GossipIngressError::new(
            "p2p_ingress_block_height_invalid",
            format!("block_height field must be positive integer, found: {block_height_raw}"),
        )
    })?;
    if block_height == 0 {
        return Err(GossipIngressError::new(
            "p2p_ingress_block_height_invalid",
            "block_height field must be positive integer, found: 0",
        ));
    }

    let producer_role_raw = required_payload_field(fields, "producer_role")?;
    let producer_role = match producer_role_raw {
        "processor" => NodeRole::Processor,
        "listener" => NodeRole::Listener,
        "approver" => NodeRole::Approver,
        other => {
            return Err(GossipIngressError::new(
                "p2p_ingress_block_role_invalid",
                format!("unsupported producer_role field value: {other}"),
            ));
        }
    };

    let payload_digest = required_payload_field(fields, "payload_digest")?;
    let transaction_ids_raw = required_payload_field(fields, "transaction_ids")?;

    let mut seen_ids = BTreeSet::new();
    let mut transaction_ids = Vec::new();
    for tx_id in transaction_ids_raw.split(',').map(|value| value.trim()) {
        if tx_id.is_empty() {
            continue;
        }
        if !seen_ids.insert(tx_id.to_owned()) {
            return Err(GossipIngressError::new(
                "p2p_ingress_block_transaction_ids_invalid",
                format!("duplicate transaction id in block candidate payload: {tx_id}"),
            ));
        }
        transaction_ids.push(tx_id.to_owned());
    }
    if transaction_ids.is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_block_transaction_ids_invalid",
            "transaction_ids field must contain at least one identifier",
        ));
    }

    Ok(GossipIngressRecord::BlockCandidate(CanonicalCommitRecord {
        block_height,
        producer_role,
        payload_digest: payload_digest.to_owned(),
        transaction_ids,
    }))
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
