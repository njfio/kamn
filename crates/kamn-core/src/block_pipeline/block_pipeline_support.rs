use super::*;

pub(super) fn validate_transport_payload_field_value(
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
    pub(super) fn new(reason_code: &'static str, detail: impl Into<String>) -> Self {
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
pub(super) enum GossipIngressTopicKind {
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
    pub(super) fn from_commit_report(report: &BlockPipelineCommitReport) -> Self {
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

const TRANSPORT_CONVERGENCE_EVIDENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.transport-convergence-evidence.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
/// Evidence bundle summarizing one transport convergence fault-drill execution.
pub struct TransportConvergenceEvidenceBundle {
    /// Versioned schema marker for policy and contract-lane checks.
    pub schema_version: String,
    /// Fault-drill case identifier.
    pub case_id: String,
    /// Count of accepted canonical candidates.
    pub accepted_candidate_count: usize,
    /// Count of rejected canonical candidates.
    pub rejected_candidate_count: usize,
    /// Deterministic set of rejection reason codes observed in this case.
    pub rejected_reason_codes: Vec<String>,
    /// Persisted canonical commit count after reconciliation.
    pub persisted_commit_count: usize,
    /// Highest persisted canonical block height after reconciliation.
    pub persisted_highest_block_height: Option<u64>,
    /// Deterministic continuity status marker (`verified` on success).
    pub continuity_status: String,
}

/// Builds deterministic convergence evidence for partition/rejoin and publish-drop drills.
pub fn build_transport_convergence_evidence_bundle(
    case_id: &str,
    outcomes: &[CanonicalCandidateOutcome],
    persisted_commits: &[CanonicalCommitRecord],
) -> Result<TransportConvergenceEvidenceBundle, BlockPipelineError> {
    if case_id.trim().is_empty() {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "transport_convergence_case_id_missing".to_owned(),
            detail: "transport convergence case id cannot be empty".to_owned(),
        });
    }

    let mut accepted_candidate_count = 0usize;
    let mut rejected_candidate_count = 0usize;
    let mut rejected_reason_codes = BTreeSet::new();
    for outcome in outcomes {
        match &outcome.decision {
            CanonicalCandidateDecision::Accepted => {
                accepted_candidate_count += 1;
            }
            CanonicalCandidateDecision::Rejected { reason_code } => {
                rejected_candidate_count += 1;
                rejected_reason_codes.insert(reason_code.clone());
            }
        }
    }

    let mut highest = None;
    for record in persisted_commits {
        if let Some(previous) = highest {
            if record.block_height <= previous {
                return Err(BlockPipelineError::ReplayDrift {
                    reason_code: "transport_convergence_commit_height_regression".to_owned(),
                    detail: format!(
                        "persisted convergence commit height regression: previous {previous}, found {}",
                        record.block_height
                    ),
                });
            }
        }
        highest = Some(record.block_height);
    }

    Ok(TransportConvergenceEvidenceBundle {
        schema_version: TRANSPORT_CONVERGENCE_EVIDENCE_SCHEMA_VERSION.to_owned(),
        case_id: case_id.to_owned(),
        accepted_candidate_count,
        rejected_candidate_count,
        rejected_reason_codes: rejected_reason_codes.into_iter().collect(),
        persisted_commit_count: persisted_commits.len(),
        persisted_highest_block_height: highest,
        continuity_status: "verified".to_owned(),
    })
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
