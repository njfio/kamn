use crate::config::{NodeConfig, NodeRole};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerLifecycleState {
    Disconnected,
    Connecting,
    Active,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerLifecycleEvent {
    StartConnect,
    HandshakeSucceeded,
    HeartbeatMissed,
    HeartbeatRestored,
    Disconnect,
    Rejoin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeLifecycleError {
    InvalidPeerId,
    InvalidTransition {
        from: PeerLifecycleState,
        event: PeerLifecycleEvent,
    },
}

impl Display for RuntimeLifecycleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeerId => write!(f, "runtime peer id cannot be empty"),
            Self::InvalidTransition { from, event } => {
                write!(
                    f,
                    "invalid peer lifecycle transition from {from:?} via {event:?}"
                )
            }
        }
    }
}

impl Error for RuntimeLifecycleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycle {
    peer_id: String,
    state: PeerLifecycleState,
}

impl PeerLifecycle {
    pub fn new(peer_id: &str) -> Result<Self, RuntimeLifecycleError> {
        if peer_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::InvalidPeerId);
        }
        Ok(Self {
            peer_id: peer_id.to_owned(),
            state: PeerLifecycleState::Disconnected,
        })
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn state(&self) -> PeerLifecycleState {
        self.state
    }

    pub fn transition(
        &mut self,
        event: PeerLifecycleEvent,
    ) -> Result<PeerLifecycleState, RuntimeLifecycleError> {
        let Some(next_state) = next_peer_state(self.state, event) else {
            return Err(RuntimeLifecycleError::InvalidTransition {
                from: self.state,
                event,
            });
        };
        self.state = next_state;
        Ok(next_state)
    }
}

fn next_peer_state(
    from: PeerLifecycleState,
    event: PeerLifecycleEvent,
) -> Option<PeerLifecycleState> {
    match (from, event) {
        (PeerLifecycleState::Disconnected, PeerLifecycleEvent::StartConnect)
        | (PeerLifecycleState::Disconnected, PeerLifecycleEvent::Rejoin) => {
            Some(PeerLifecycleState::Connecting)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::HandshakeSucceeded) => {
            Some(PeerLifecycleState::Active)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Active, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Degraded, PeerLifecycleEvent::Disconnect) => {
            Some(PeerLifecycleState::Disconnected)
        }
        (PeerLifecycleState::Active, PeerLifecycleEvent::HeartbeatMissed) => {
            Some(PeerLifecycleState::Degraded)
        }
        (PeerLifecycleState::Degraded, PeerLifecycleEvent::HeartbeatRestored) => {
            Some(PeerLifecycleState::Active)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeQueueError {
    InvalidCapacity {
        capacity: usize,
    },
    Overflow {
        capacity: usize,
        attempted_len: usize,
    },
}

impl Display for RuntimeQueueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCapacity { capacity } => {
                write!(
                    f,
                    "runtime queue capacity must be at least 1 (got {capacity})"
                )
            }
            Self::Overflow {
                capacity,
                attempted_len,
            } => write!(
                f,
                "runtime queue overflow: capacity {capacity}, attempted length {attempted_len}"
            ),
        }
    }
}

impl Error for RuntimeQueueError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedRuntimeQueue<T> {
    capacity: usize,
    entries: VecDeque<T>,
}

impl<T> BoundedRuntimeQueue<T> {
    pub fn new(capacity: usize) -> Result<Self, RuntimeQueueError> {
        if capacity == 0 {
            return Err(RuntimeQueueError::InvalidCapacity { capacity });
        }
        Ok(Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn enqueue(&mut self, item: T) -> Result<(), RuntimeQueueError> {
        if self.entries.len() >= self.capacity {
            return Err(RuntimeQueueError::Overflow {
                capacity: self.capacity,
                attempted_len: self.entries.len() + 1,
            });
        }
        self.entries.push_back(item);
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.entries.pop_front()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalCandidate {
    id: String,
    sender_did: String,
    nonce: u64,
    state_hash: String,
}

impl ProposalCandidate {
    pub fn new(
        id: &str,
        sender_did: &str,
        nonce: u64,
        state_hash: &str,
    ) -> Result<Self, ProposalPlannerError> {
        if id.trim().is_empty() {
            return Err(ProposalPlannerError::InvalidCandidateId);
        }
        if sender_did.trim().is_empty() {
            return Err(ProposalPlannerError::InvalidSenderDid);
        }
        if state_hash.trim().is_empty() {
            return Err(ProposalPlannerError::InvalidStateHash);
        }
        if nonce == 0 {
            return Err(ProposalPlannerError::InvalidNonce);
        }
        Ok(Self {
            id: id.to_owned(),
            sender_did: sender_did.to_owned(),
            nonce,
            state_hash: state_hash.to_owned(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn sender_did(&self) -> &str {
        &self.sender_did
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalPlan {
    ordered_candidates: Vec<ProposalCandidate>,
}

impl ProposalPlan {
    pub fn ordered_candidates(&self) -> &[ProposalCandidate] {
        &self.ordered_candidates
    }

    pub fn ordered_candidate_ids(&self) -> Vec<String> {
        self.ordered_candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalPlannerError {
    InvalidCandidateId,
    InvalidSenderDid,
    InvalidStateHash,
    InvalidNonce,
    DuplicateCandidateId(String),
    StaleStateHash { expected: String, found: String },
}

impl Display for ProposalPlannerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCandidateId => write!(f, "proposal candidate id cannot be empty"),
            Self::InvalidSenderDid => write!(f, "proposal candidate sender DID cannot be empty"),
            Self::InvalidStateHash => write!(f, "proposal candidate state hash cannot be empty"),
            Self::InvalidNonce => write!(f, "proposal candidate nonce must be positive"),
            Self::DuplicateCandidateId(id) => {
                write!(f, "duplicate proposal candidate id: {id}")
            }
            Self::StaleStateHash { expected, found } => {
                write!(
                    f,
                    "proposal candidate state hash mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for ProposalPlannerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicProposalPlanner {
    expected_state_hash: String,
}

impl DeterministicProposalPlanner {
    pub fn new(expected_state_hash: &str) -> Self {
        Self {
            expected_state_hash: expected_state_hash.to_owned(),
        }
    }

    pub fn plan(
        &self,
        mut candidates: Vec<ProposalCandidate>,
    ) -> Result<ProposalPlan, ProposalPlannerError> {
        let mut seen_ids = HashSet::new();
        for candidate in &candidates {
            if !seen_ids.insert(candidate.id.as_str()) {
                return Err(ProposalPlannerError::DuplicateCandidateId(
                    candidate.id.clone(),
                ));
            }
            if candidate.state_hash != self.expected_state_hash {
                return Err(ProposalPlannerError::StaleStateHash {
                    expected: self.expected_state_hash.clone(),
                    found: candidate.state_hash.clone(),
                });
            }
        }

        candidates.sort_by(|left, right| {
            left.nonce
                .cmp(&right.nonce)
                .then_with(|| left.sender_did.cmp(&right.sender_did))
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(ProposalPlan {
            ordered_candidates: candidates,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejoinAttempt {
    node_id: String,
    state_version: u64,
    state_hash: String,
    resume_token: String,
}

impl RejoinAttempt {
    pub fn new(
        node_id: &str,
        state_version: u64,
        state_hash: &str,
        resume_token: &str,
    ) -> Result<Self, RecoveryGuardError> {
        if node_id.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidNodeId);
        }
        if state_version == 0 {
            return Err(RecoveryGuardError::InvalidStateVersion);
        }
        if state_hash.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidStateHash);
        }
        if resume_token.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidResumeToken);
        }
        Ok(Self {
            node_id: node_id.to_owned(),
            state_version,
            state_hash: state_hash.to_owned(),
            resume_token: resume_token.to_owned(),
        })
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    pub fn resume_token(&self) -> &str {
        &self.resume_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    state_version: u64,
    state_hash: String,
    cursor: u64,
}

impl RuntimeSnapshot {
    pub fn new(state_version: u64, state_hash: &str) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(state_version, state_hash, state_version)
    }

    pub fn with_cursor(
        state_version: u64,
        state_hash: &str,
        cursor: u64,
    ) -> Result<Self, SnapshotRestoreError> {
        if state_version == 0 {
            return Err(SnapshotRestoreError::InvalidStateVersion);
        }
        if !is_valid_snapshot_hash(state_hash) {
            return Err(SnapshotRestoreError::InvalidStateHash);
        }
        if cursor == 0 {
            return Err(SnapshotRestoreError::InvalidCursor);
        }
        Ok(Self {
            state_version,
            state_hash: state_hash.to_owned(),
            cursor,
        })
    }

    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    pub fn cursor(&self) -> u64 {
        self.cursor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotRestoreError {
    InvalidStateVersion,
    InvalidStateHash,
    InvalidCursor,
    StateVersionMismatch { expected: u64, found: u64 },
    StateHashMismatch { expected: String, found: String },
    CursorMismatch { expected: u64, found: u64 },
}

impl Display for SnapshotRestoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStateVersion => write!(f, "snapshot state version must be positive"),
            Self::InvalidStateHash => write!(f, "snapshot state hash cannot be empty"),
            Self::InvalidCursor => write!(f, "snapshot cursor must be positive"),
            Self::StateVersionMismatch { expected, found } => {
                write!(
                    f,
                    "snapshot state version mismatch: expected {expected}, found {found}"
                )
            }
            Self::StateHashMismatch { expected, found } => {
                write!(
                    f,
                    "snapshot state hash mismatch: expected {expected}, found {found}"
                )
            }
            Self::CursorMismatch { expected, found } => {
                write!(
                    f,
                    "snapshot cursor mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for SnapshotRestoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRestoreGuard {
    expected_state_version: u64,
    expected_state_hash: String,
    expected_cursor: Option<u64>,
}

impl SnapshotRestoreGuard {
    pub fn new(
        expected_state_version: u64,
        expected_state_hash: &str,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(expected_state_version, expected_state_hash, None)
    }

    pub fn with_expected_cursor(
        expected_state_version: u64,
        expected_state_hash: &str,
        expected_cursor: u64,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(
            expected_state_version,
            expected_state_hash,
            Some(expected_cursor),
        )
    }

    fn with_cursor(
        expected_state_version: u64,
        expected_state_hash: &str,
        expected_cursor: Option<u64>,
    ) -> Result<Self, SnapshotRestoreError> {
        if expected_state_version == 0 {
            return Err(SnapshotRestoreError::InvalidStateVersion);
        }
        if !is_valid_snapshot_hash(expected_state_hash) {
            return Err(SnapshotRestoreError::InvalidStateHash);
        }
        if matches!(expected_cursor, Some(0)) {
            return Err(SnapshotRestoreError::InvalidCursor);
        }
        Ok(Self {
            expected_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
            expected_cursor,
        })
    }

    pub fn validate(&self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotRestoreError> {
        if snapshot.state_version() != self.expected_state_version {
            return Err(SnapshotRestoreError::StateVersionMismatch {
                expected: self.expected_state_version,
                found: snapshot.state_version(),
            });
        }
        if snapshot.state_hash() != self.expected_state_hash {
            return Err(SnapshotRestoreError::StateHashMismatch {
                expected: self.expected_state_hash.clone(),
                found: snapshot.state_hash().to_owned(),
            });
        }
        if let Some(expected_cursor) = self.expected_cursor {
            if snapshot.cursor() != expected_cursor {
                return Err(SnapshotRestoreError::CursorMismatch {
                    expected: expected_cursor,
                    found: snapshot.cursor(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotStoreError {
    Io(String),
    InvalidPayload(String),
    StateVersionRegression {
        previous: u64,
        found: u64,
    },
    CursorRegression {
        previous: u64,
        found: u64,
    },
    StaleStateHash {
        state_hash: String,
        previous_version: u64,
        found_version: u64,
    },
}

impl Display for SnapshotStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "snapshot store I/O error: {message}"),
            Self::InvalidPayload(payload) => {
                write!(f, "snapshot store invalid payload: {payload}")
            }
            Self::StateVersionRegression { previous, found } => {
                write!(
                    f,
                    "snapshot state version regression: previous {previous}, found {found}"
                )
            }
            Self::CursorRegression { previous, found } => {
                write!(
                    f,
                    "snapshot cursor regression: previous {previous}, found {found}"
                )
            }
            Self::StaleStateHash {
                state_hash,
                previous_version,
                found_version,
            } => {
                write!(
                    f,
                    "snapshot stale state hash detected for {state_hash}: versions {previous_version}->{found_version}"
                )
            }
        }
    }
}

impl Error for SnapshotStoreError {}

pub trait RuntimeSnapshotStore {
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError>;
    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError>;
    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryRuntimeSnapshotStore {
    entries: Vec<RuntimeSnapshot>,
}

impl RuntimeSnapshotStore for InMemoryRuntimeSnapshotStore {
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError> {
        validate_snapshot_continuity(self.entries.last(), &snapshot)?;
        self.entries.push(snapshot);
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError> {
        Ok(self.entries.last().cloned())
    }

    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError> {
        Ok(self.entries.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRuntimeSnapshotStore {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecoveryResult {
    pub latest: Option<RuntimeSnapshot>,
    pub recovered_entries: usize,
    pub dropped_corrupt_entries: usize,
}

impl FileRuntimeSnapshotStore {
    pub fn new(path: PathBuf) -> Result<Self, SnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(SnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        Ok(Self { path })
    }

    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<SnapshotRecoveryResult, SnapshotStoreError> {
        if !self.path.exists() {
            return Ok(SnapshotRecoveryResult {
                latest: None,
                recovered_entries: 0,
                dropped_corrupt_entries: 0,
            });
        }

        let payload = fs::read_to_string(&self.path)
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))?;
        let mut snapshots = Vec::new();
        let mut dropped_corrupt_entries = 0;
        let mut corruption_detected = false;

        for line in payload.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if corruption_detected {
                dropped_corrupt_entries += 1;
                continue;
            }

            match parse_snapshot_line(trimmed) {
                Ok(snapshot) => {
                    if validate_snapshot_continuity(snapshots.last(), &snapshot).is_err() {
                        corruption_detected = true;
                        dropped_corrupt_entries += 1;
                        continue;
                    }
                    snapshots.push(snapshot);
                }
                Err(_) => {
                    corruption_detected = true;
                    dropped_corrupt_entries += 1;
                }
            }
        }

        if corruption_detected {
            self.persist_snapshots(&snapshots)?;
        }

        Ok(SnapshotRecoveryResult {
            latest: snapshots.last().cloned(),
            recovered_entries: snapshots.len(),
            dropped_corrupt_entries,
        })
    }

    fn persist_snapshots(&self, snapshots: &[RuntimeSnapshot]) -> Result<(), SnapshotStoreError> {
        let mut serialized = String::new();
        for snapshot in snapshots {
            serialized.push_str(&format!(
                "{}|{}|{}\n",
                snapshot.state_version(),
                snapshot.state_hash(),
                snapshot.cursor()
            ));
        }
        fs::write(&self.path, serialized).map_err(|error| SnapshotStoreError::Io(error.to_string()))
    }
}

impl RuntimeSnapshotStore for FileRuntimeSnapshotStore {
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError> {
        if let Some(previous) = self.read_latest()? {
            validate_snapshot_continuity(Some(&previous), &snapshot)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))?;
        let serialized = format!(
            "{}|{}|{}\n",
            snapshot.state_version(),
            snapshot.state_hash(),
            snapshot.cursor()
        );
        file.write_all(serialized.as_bytes())
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))
    }

    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError> {
        Ok(self.list()?.pop())
    }

    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let payload = fs::read_to_string(&self.path)
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))?;
        let mut snapshots = Vec::new();

        for line in payload.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let snapshot = parse_snapshot_line(trimmed)?;
            validate_snapshot_continuity(snapshots.last(), &snapshot)?;
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }
}

fn parse_snapshot_line(line: &str) -> Result<RuntimeSnapshot, SnapshotStoreError> {
    let mut segments = line.split('|');
    let Some(state_version_raw) = segments.next() else {
        return Err(SnapshotStoreError::InvalidPayload(line.to_owned()));
    };
    let Some(state_hash_raw) = segments.next() else {
        return Err(SnapshotStoreError::InvalidPayload(line.to_owned()));
    };
    let cursor_raw = segments.next();
    if segments.next().is_some() {
        return Err(SnapshotStoreError::InvalidPayload(line.to_owned()));
    }

    let state_version = state_version_raw
        .parse::<u64>()
        .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))?;
    if let Some(cursor_raw) = cursor_raw {
        let cursor = cursor_raw
            .parse::<u64>()
            .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))?;
        RuntimeSnapshot::with_cursor(state_version, state_hash_raw, cursor)
            .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))
    } else {
        RuntimeSnapshot::new(state_version, state_hash_raw)
            .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))
    }
}

fn is_valid_snapshot_hash(state_hash: &str) -> bool {
    !state_hash.trim().is_empty()
        && !state_hash.contains('|')
        && !state_hash.contains('\n')
        && !state_hash.contains('\r')
}

fn validate_snapshot_continuity(
    previous: Option<&RuntimeSnapshot>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), SnapshotStoreError> {
    if let Some(previous) = previous {
        if snapshot.state_version() <= previous.state_version() {
            return Err(SnapshotStoreError::StateVersionRegression {
                previous: previous.state_version(),
                found: snapshot.state_version(),
            });
        }
        if snapshot.cursor() <= previous.cursor() {
            return Err(SnapshotStoreError::CursorRegression {
                previous: previous.cursor(),
                found: snapshot.cursor(),
            });
        }
        if snapshot.state_hash() == previous.state_hash() {
            return Err(SnapshotStoreError::StaleStateHash {
                state_hash: snapshot.state_hash().to_owned(),
                previous_version: previous.state_version(),
                found_version: snapshot.state_version(),
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructLockLease {
    owner_id: String,
    fencing_token: u64,
}

impl ConstructLockLease {
    fn new(owner_id: String, fencing_token: u64) -> Self {
        Self {
            owner_id,
            fencing_token,
        }
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn fencing_token(&self) -> u64 {
        self.fencing_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructLockError {
    InvalidLeaseTtl,
    InvalidOwnerId,
    NoActiveLease,
    NoLeaseForExecution,
    LeaseAlreadyHeld { owner: String },
    LeaseOwnerMismatch { expected: String, found: String },
    StaleFencingToken { expected: u64, found: u64 },
}

impl Display for ConstructLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLeaseTtl => write!(f, "construct lock lease ttl must be positive"),
            Self::InvalidOwnerId => write!(f, "construct lock owner id cannot be empty"),
            Self::NoActiveLease => write!(f, "construct lock has no active lease"),
            Self::NoLeaseForExecution => {
                write!(
                    f,
                    "daemon execution requires an active construct lock lease"
                )
            }
            Self::LeaseAlreadyHeld { owner } => {
                write!(f, "construct lock lease already held by {owner}")
            }
            Self::LeaseOwnerMismatch { expected, found } => {
                write!(
                    f,
                    "construct lock owner mismatch: expected {expected}, found {found}"
                )
            }
            Self::StaleFencingToken { expected, found } => {
                write!(
                    f,
                    "construct lock stale fencing token: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for ConstructLockError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructLockGuard {
    lease_ttl_ticks: u64,
    current_lease: Option<ConstructLockLease>,
}

impl ConstructLockGuard {
    pub fn new(lease_ttl_ticks: u64) -> Result<Self, ConstructLockError> {
        if lease_ttl_ticks == 0 {
            return Err(ConstructLockError::InvalidLeaseTtl);
        }
        Ok(Self {
            lease_ttl_ticks,
            current_lease: None,
        })
    }

    pub fn lease_ttl_ticks(&self) -> u64 {
        self.lease_ttl_ticks
    }

    pub fn acquire_for(
        &mut self,
        owner_id: &str,
    ) -> Result<ConstructLockLease, ConstructLockError> {
        if owner_id.trim().is_empty() {
            return Err(ConstructLockError::InvalidOwnerId);
        }

        if let Some(lease) = &self.current_lease {
            if lease.owner_id() != owner_id {
                return Err(ConstructLockError::LeaseAlreadyHeld {
                    owner: lease.owner_id().to_owned(),
                });
            }
            return Ok(lease.clone());
        }

        let lease = ConstructLockLease::new(owner_id.to_owned(), 1);
        self.current_lease = Some(lease.clone());
        Ok(lease)
    }

    pub fn renew(
        &mut self,
        owner_id: &str,
        fencing_token: u64,
    ) -> Result<ConstructLockLease, ConstructLockError> {
        if owner_id.trim().is_empty() {
            return Err(ConstructLockError::InvalidOwnerId);
        }
        let current_lease = self
            .current_lease
            .as_ref()
            .ok_or(ConstructLockError::NoActiveLease)?;

        if current_lease.owner_id() != owner_id {
            return Err(ConstructLockError::LeaseOwnerMismatch {
                expected: current_lease.owner_id().to_owned(),
                found: owner_id.to_owned(),
            });
        }

        if current_lease.fencing_token() != fencing_token {
            return Err(ConstructLockError::StaleFencingToken {
                expected: current_lease.fencing_token(),
                found: fencing_token,
            });
        }

        let renewed = ConstructLockLease::new(
            current_lease.owner_id().to_owned(),
            current_lease.fencing_token() + 1,
        );
        self.current_lease = Some(renewed.clone());
        Ok(renewed)
    }

    pub fn release(
        &mut self,
        owner_id: &str,
        fencing_token: u64,
    ) -> Result<(), ConstructLockError> {
        if owner_id.trim().is_empty() {
            return Err(ConstructLockError::InvalidOwnerId);
        }
        let current_lease = self
            .current_lease
            .as_ref()
            .ok_or(ConstructLockError::NoActiveLease)?;

        if current_lease.owner_id() != owner_id {
            return Err(ConstructLockError::LeaseOwnerMismatch {
                expected: current_lease.owner_id().to_owned(),
                found: owner_id.to_owned(),
            });
        }

        if current_lease.fencing_token() != fencing_token {
            return Err(ConstructLockError::StaleFencingToken {
                expected: current_lease.fencing_token(),
                found: fencing_token,
            });
        }

        self.current_lease = None;
        Ok(())
    }

    pub fn transfer(
        &mut self,
        owner_id: &str,
        next_owner_id: &str,
        fencing_token: u64,
    ) -> Result<ConstructLockLease, ConstructLockError> {
        if owner_id.trim().is_empty() || next_owner_id.trim().is_empty() {
            return Err(ConstructLockError::InvalidOwnerId);
        }
        let current_lease = self
            .current_lease
            .as_ref()
            .ok_or(ConstructLockError::NoActiveLease)?;

        if current_lease.owner_id() != owner_id {
            return Err(ConstructLockError::LeaseOwnerMismatch {
                expected: current_lease.owner_id().to_owned(),
                found: owner_id.to_owned(),
            });
        }

        if current_lease.fencing_token() != fencing_token {
            return Err(ConstructLockError::StaleFencingToken {
                expected: current_lease.fencing_token(),
                found: fencing_token,
            });
        }

        if current_lease.owner_id() == next_owner_id {
            return Err(ConstructLockError::LeaseAlreadyHeld {
                owner: current_lease.owner_id().to_owned(),
            });
        }

        let transferred =
            ConstructLockLease::new(next_owner_id.to_owned(), current_lease.fencing_token() + 1);
        self.current_lease = Some(transferred.clone());
        Ok(transferred)
    }

    pub fn validate_execution_lease(
        &self,
        owner_id: &str,
        fencing_token: u64,
    ) -> Result<(), ConstructLockError> {
        if owner_id.trim().is_empty() {
            return Err(ConstructLockError::InvalidOwnerId);
        }
        let current_lease = self
            .current_lease
            .as_ref()
            .ok_or(ConstructLockError::NoLeaseForExecution)?;

        if current_lease.owner_id() != owner_id {
            return Err(ConstructLockError::LeaseOwnerMismatch {
                expected: current_lease.owner_id().to_owned(),
                found: owner_id.to_owned(),
            });
        }

        if current_lease.fencing_token() != fencing_token {
            return Err(ConstructLockError::StaleFencingToken {
                expected: current_lease.fencing_token(),
                found: fencing_token,
            });
        }

        Ok(())
    }
}

pub fn execute_processor_daemon_tick(
    lock_guard: &ConstructLockGuard,
    owner_id: &str,
    fencing_token: u64,
    executed_ticks: u64,
) -> Result<u64, ConstructLockError> {
    lock_guard.validate_execution_lease(owner_id, fencing_token)?;
    Ok(executed_ticks + 1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerAttestation {
    listener_did: String,
    attestation_id: String,
}

impl ListenerAttestation {
    pub fn new(listener_did: &str, attestation_id: &str) -> Result<Self, ListenerQuorumError> {
        if !is_valid_listener_did(listener_did) {
            return Err(ListenerQuorumError::InvalidListenerDid);
        }
        if attestation_id.trim().is_empty() {
            return Err(ListenerQuorumError::InvalidAttestationId);
        }
        Ok(Self {
            listener_did: listener_did.to_owned(),
            attestation_id: attestation_id.to_owned(),
        })
    }

    pub fn listener_did(&self) -> &str {
        &self.listener_did
    }

    pub fn attestation_id(&self) -> &str {
        &self.attestation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerQuorumInput {
    event_id: String,
    event_sequence: u64,
    attestations: Vec<ListenerAttestation>,
}

impl ListenerQuorumInput {
    pub fn new(
        event_id: &str,
        event_sequence: u64,
        attestations: Vec<ListenerAttestation>,
    ) -> Result<Self, ListenerQuorumError> {
        if event_id.trim().is_empty() {
            return Err(ListenerQuorumError::InvalidEventId);
        }
        if event_sequence == 0 {
            return Err(ListenerQuorumError::InvalidEventSequence);
        }
        Ok(Self {
            event_id: event_id.to_owned(),
            event_sequence,
            attestations,
        })
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn event_sequence(&self) -> u64 {
        self.event_sequence
    }

    pub fn attestations(&self) -> &[ListenerAttestation] {
        &self.attestations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerQuorumDecision {
    pub event_id: String,
    pub event_sequence: u64,
    pub required_confirmations: usize,
    pub confirmed_listeners: Vec<String>,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenerQuorumError {
    InvalidRequiredConfirmations {
        required: usize,
    },
    InvalidEventId,
    InvalidEventSequence,
    InvalidListenerDid,
    InvalidAttestationId,
    DuplicateListenerAttestation {
        listener_did: String,
    },
    ReplayedEventSequence {
        event_id: String,
        previous_sequence: u64,
        received_sequence: u64,
    },
    InsufficientConfirmations {
        required: usize,
        received: usize,
    },
}

impl Display for ListenerQuorumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequiredConfirmations { required } => {
                write!(f, "invalid listener quorum requirement: {required}")
            }
            Self::InvalidEventId => write!(f, "listener quorum event id cannot be empty"),
            Self::InvalidEventSequence => {
                write!(f, "listener quorum event sequence must be positive")
            }
            Self::InvalidListenerDid => write!(f, "listener attestation did is invalid"),
            Self::InvalidAttestationId => write!(f, "listener attestation id cannot be empty"),
            Self::DuplicateListenerAttestation { listener_did } => {
                write!(
                    f,
                    "duplicate listener attestation replay detected for {listener_did}"
                )
            }
            Self::ReplayedEventSequence {
                event_id,
                previous_sequence,
                received_sequence,
            } => {
                write!(
                    f,
                    "listener event sequence replay detected for {event_id}: previous {previous_sequence}, received {received_sequence}"
                )
            }
            Self::InsufficientConfirmations { required, received } => {
                write!(
                    f,
                    "listener quorum insufficient confirmations: required {required}, received {received}"
                )
            }
        }
    }
}

impl Error for ListenerQuorumError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerQuorumEvaluator {
    required_confirmations: usize,
    latest_sequence_by_event: BTreeMap<String, u64>,
}

impl ListenerQuorumEvaluator {
    pub fn new(required_confirmations: usize) -> Result<Self, ListenerQuorumError> {
        if required_confirmations == 0 {
            return Err(ListenerQuorumError::InvalidRequiredConfirmations {
                required: required_confirmations,
            });
        }
        Ok(Self {
            required_confirmations,
            latest_sequence_by_event: BTreeMap::new(),
        })
    }

    pub fn evaluate(
        &mut self,
        input: ListenerQuorumInput,
    ) -> Result<ListenerQuorumDecision, ListenerQuorumError> {
        if let Some(previous_sequence) = self.latest_sequence_by_event.get(input.event_id()) {
            if input.event_sequence() <= *previous_sequence {
                return Err(ListenerQuorumError::ReplayedEventSequence {
                    event_id: input.event_id().to_owned(),
                    previous_sequence: *previous_sequence,
                    received_sequence: input.event_sequence(),
                });
            }
        }

        let mut confirmed = BTreeSet::new();
        for attestation in input.attestations() {
            if !is_valid_listener_did(attestation.listener_did()) {
                return Err(ListenerQuorumError::InvalidListenerDid);
            }
            if !confirmed.insert(attestation.listener_did().to_owned()) {
                return Err(ListenerQuorumError::DuplicateListenerAttestation {
                    listener_did: attestation.listener_did().to_owned(),
                });
            }
        }

        let confirmed_listeners = confirmed.into_iter().collect::<Vec<_>>();
        if confirmed_listeners.len() < self.required_confirmations {
            return Err(ListenerQuorumError::InsufficientConfirmations {
                required: self.required_confirmations,
                received: confirmed_listeners.len(),
            });
        }

        self.latest_sequence_by_event
            .insert(input.event_id().to_owned(), input.event_sequence());

        Ok(ListenerQuorumDecision {
            event_id: input.event_id().to_owned(),
            event_sequence: input.event_sequence(),
            required_confirmations: self.required_confirmations,
            confirmed_listeners,
            accepted: true,
        })
    }
}

pub fn evaluate_daemon_listener_quorum(
    evaluator: &mut ListenerQuorumEvaluator,
    input: ListenerQuorumInput,
) -> Result<ListenerQuorumDecision, ListenerQuorumError> {
    evaluator.evaluate(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproverAttestation {
    approver_did: String,
    payload_digest: String,
    attestation_id: String,
}

impl ApproverAttestation {
    pub fn new(
        approver_did: &str,
        payload_digest: &str,
        attestation_id: &str,
    ) -> Result<Self, ApproverQuorumError> {
        if !is_valid_kamn_did(approver_did) {
            return Err(ApproverQuorumError::InvalidApproverDid);
        }
        if payload_digest.trim().is_empty() {
            return Err(ApproverQuorumError::InvalidPayloadDigest);
        }
        if attestation_id.trim().is_empty() {
            return Err(ApproverQuorumError::InvalidAttestationId);
        }
        Ok(Self {
            approver_did: approver_did.to_owned(),
            payload_digest: payload_digest.to_owned(),
            attestation_id: attestation_id.to_owned(),
        })
    }

    pub fn approver_did(&self) -> &str {
        &self.approver_did
    }

    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproverQuorumInput {
    action_id: String,
    payload_digest: String,
    attestations: Vec<ApproverAttestation>,
}

impl ApproverQuorumInput {
    pub fn new(
        action_id: &str,
        payload_digest: &str,
        attestations: Vec<ApproverAttestation>,
    ) -> Result<Self, ApproverQuorumError> {
        if action_id.trim().is_empty() {
            return Err(ApproverQuorumError::InvalidActionId);
        }
        if payload_digest.trim().is_empty() {
            return Err(ApproverQuorumError::InvalidPayloadDigest);
        }
        Ok(Self {
            action_id: action_id.to_owned(),
            payload_digest: payload_digest.to_owned(),
            attestations,
        })
    }

    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }

    pub fn attestations(&self) -> &[ApproverAttestation] {
        &self.attestations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproverQuorumDecision {
    pub action_id: String,
    pub required_approvals: usize,
    pub approved_by: Vec<String>,
    pub authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApproverQuorumError {
    InvalidRequiredApprovals { required: usize },
    InvalidActionId,
    InvalidPayloadDigest,
    InvalidApproverDid,
    InvalidAttestationId,
    DuplicateApproverAttestation { approver_did: String },
    PayloadDigestMismatch { expected: String, found: String },
    InsufficientApprovals { required: usize, received: usize },
}

impl Display for ApproverQuorumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequiredApprovals { required } => {
                write!(f, "invalid approver quorum requirement: {required}")
            }
            Self::InvalidActionId => write!(f, "approver quorum action id cannot be empty"),
            Self::InvalidPayloadDigest => {
                write!(f, "approver quorum payload digest cannot be empty")
            }
            Self::InvalidApproverDid => write!(f, "approver attestation did is invalid"),
            Self::InvalidAttestationId => write!(f, "approver attestation id cannot be empty"),
            Self::DuplicateApproverAttestation { approver_did } => {
                write!(
                    f,
                    "duplicate approver attestation replay detected for {approver_did}"
                )
            }
            Self::PayloadDigestMismatch { expected, found } => {
                write!(
                    f,
                    "approver payload digest mismatch: expected {expected}, found {found}"
                )
            }
            Self::InsufficientApprovals { required, received } => {
                write!(
                    f,
                    "approver quorum insufficient approvals: required {required}, received {received}"
                )
            }
        }
    }
}

impl Error for ApproverQuorumError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproverQuorumEvaluator {
    required_approvals: usize,
}

impl ApproverQuorumEvaluator {
    pub fn new(required_approvals: usize) -> Result<Self, ApproverQuorumError> {
        if required_approvals == 0 {
            return Err(ApproverQuorumError::InvalidRequiredApprovals {
                required: required_approvals,
            });
        }
        Ok(Self { required_approvals })
    }

    pub fn authorize(
        &self,
        input: ApproverQuorumInput,
    ) -> Result<ApproverQuorumDecision, ApproverQuorumError> {
        let mut approved = BTreeSet::new();

        for attestation in input.attestations() {
            if !is_valid_kamn_did(attestation.approver_did()) {
                return Err(ApproverQuorumError::InvalidApproverDid);
            }
            if attestation.payload_digest() != input.payload_digest() {
                return Err(ApproverQuorumError::PayloadDigestMismatch {
                    expected: input.payload_digest().to_owned(),
                    found: attestation.payload_digest().to_owned(),
                });
            }
            if !approved.insert(attestation.approver_did().to_owned()) {
                return Err(ApproverQuorumError::DuplicateApproverAttestation {
                    approver_did: attestation.approver_did().to_owned(),
                });
            }
        }

        let approved_by = approved.into_iter().collect::<Vec<_>>();
        if approved_by.len() < self.required_approvals {
            return Err(ApproverQuorumError::InsufficientApprovals {
                required: self.required_approvals,
                received: approved_by.len(),
            });
        }

        Ok(ApproverQuorumDecision {
            action_id: input.action_id().to_owned(),
            required_approvals: self.required_approvals,
            approved_by,
            authorized: true,
        })
    }
}

pub fn authorize_daemon_outbound_action(
    evaluator: &ApproverQuorumEvaluator,
    input: ApproverQuorumInput,
) -> Result<ApproverQuorumDecision, ApproverQuorumError> {
    evaluator.authorize(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateDivergenceStatus {
    InSync,
    Diverged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateDivergenceSeverity {
    Info,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDivergenceEvidence {
    pub peer_id: String,
    pub expected_state_version: u64,
    pub observed_state_version: u64,
    pub expected_state_hash: String,
    pub observed_state_hash: String,
    pub observed_at_tick: u64,
}

impl StateDivergenceEvidence {
    pub fn new(
        peer_id: &str,
        expected_state_version: u64,
        observed_state_version: u64,
        expected_state_hash: &str,
        observed_state_hash: &str,
        observed_at_tick: u64,
    ) -> Result<Self, StateDivergenceError> {
        if !is_valid_kamn_did(peer_id) {
            return Err(StateDivergenceError::InvalidPeerDid);
        }
        if expected_state_version == 0 {
            return Err(StateDivergenceError::InvalidStateVersion {
                field: "expected_state_version",
                value: expected_state_version,
            });
        }
        if observed_state_version == 0 {
            return Err(StateDivergenceError::InvalidStateVersion {
                field: "observed_state_version",
                value: observed_state_version,
            });
        }
        if expected_state_hash.trim().is_empty() {
            return Err(StateDivergenceError::IncompleteEvidenceField {
                field: "expected_state_hash",
            });
        }
        if observed_state_hash.trim().is_empty() {
            return Err(StateDivergenceError::IncompleteEvidenceField {
                field: "observed_state_hash",
            });
        }
        if observed_at_tick == 0 {
            return Err(StateDivergenceError::InvalidObservedTick {
                tick: observed_at_tick,
            });
        }

        Ok(Self {
            peer_id: peer_id.to_owned(),
            expected_state_version,
            observed_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
            observed_state_hash: observed_state_hash.to_owned(),
            observed_at_tick,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDivergenceWatchInput {
    evidence: StateDivergenceEvidence,
}

impl StateDivergenceWatchInput {
    pub fn new(
        peer_id: &str,
        expected_state_version: u64,
        observed_state_version: u64,
        expected_state_hash: &str,
        observed_state_hash: &str,
        observed_at_tick: u64,
    ) -> Result<Self, StateDivergenceError> {
        let evidence = StateDivergenceEvidence::new(
            peer_id,
            expected_state_version,
            observed_state_version,
            expected_state_hash,
            observed_state_hash,
            observed_at_tick,
        )?;
        Ok(Self { evidence })
    }

    pub fn evidence(&self) -> &StateDivergenceEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDivergenceReport {
    pub status: StateDivergenceStatus,
    pub severity: StateDivergenceSeverity,
    pub incident_fingerprint: String,
    pub evidence: StateDivergenceEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateDivergenceError {
    InvalidPeerDid,
    InvalidStateVersion { field: &'static str, value: u64 },
    IncompleteEvidenceField { field: &'static str },
    InvalidObservedTick { tick: u64 },
}

impl Display for StateDivergenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeerDid => write!(f, "state divergence peer did is invalid"),
            Self::InvalidStateVersion { field, value } => {
                write!(
                    f,
                    "state divergence {field} must be positive, found {value}"
                )
            }
            Self::IncompleteEvidenceField { field } => {
                write!(
                    f,
                    "state divergence evidence field cannot be empty: {field}"
                )
            }
            Self::InvalidObservedTick { tick } => {
                write!(
                    f,
                    "state divergence observed tick must be positive, found {tick}"
                )
            }
        }
    }
}

impl Error for StateDivergenceError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateDivergenceEvaluator;

impl StateDivergenceEvaluator {
    pub fn evaluate(
        &self,
        input: StateDivergenceWatchInput,
    ) -> Result<StateDivergenceReport, StateDivergenceError> {
        let evidence = input.evidence;
        let diverged = evidence.expected_state_version != evidence.observed_state_version
            || evidence.expected_state_hash != evidence.observed_state_hash;

        let status = if diverged {
            StateDivergenceStatus::Diverged
        } else {
            StateDivergenceStatus::InSync
        };
        let severity = if diverged {
            StateDivergenceSeverity::Critical
        } else {
            StateDivergenceSeverity::Info
        };
        let incident_fingerprint = format!(
            "state-divergence:{}:{}:{}:{}:{}",
            evidence.peer_id,
            evidence.expected_state_version,
            evidence.observed_state_version,
            evidence.expected_state_hash,
            evidence.observed_state_hash
        );

        Ok(StateDivergenceReport {
            status,
            severity,
            incident_fingerprint,
            evidence,
        })
    }
}

pub fn evaluate_daemon_state_divergence(
    evaluator: &StateDivergenceEvaluator,
    input: StateDivergenceWatchInput,
) -> Result<StateDivergenceReport, StateDivergenceError> {
    evaluator.evaluate(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogAnomalyKind {
    Nominal,
    LivenessDegradation,
    CensorshipSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogAnomalySeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchdogAnomalyWatchInput {
    sample_id: String,
    expected_deliveries: u32,
    delivered_deliveries: u32,
    active_peers: u32,
    healthy_peers: u32,
    sample_window_secs: u64,
    targeted_peer_count: u32,
}

impl WatchdogAnomalyWatchInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_id: &str,
        expected_deliveries: u32,
        delivered_deliveries: u32,
        active_peers: u32,
        healthy_peers: u32,
        sample_window_secs: u64,
        targeted_peer_count: u32,
    ) -> Result<Self, WatchdogAnomalyError> {
        if sample_id.trim().is_empty() {
            return Err(WatchdogAnomalyError::InvalidSampleId);
        }
        if expected_deliveries == 0 {
            return Err(WatchdogAnomalyError::InvalidExpectedDeliveries {
                expected_deliveries,
            });
        }
        if delivered_deliveries > expected_deliveries {
            return Err(WatchdogAnomalyError::InvalidSampleCounts {
                expected_deliveries,
                delivered_deliveries,
            });
        }
        if active_peers == 0 || healthy_peers > active_peers {
            return Err(WatchdogAnomalyError::InvalidPeerCounts {
                active_peers,
                healthy_peers,
            });
        }
        if sample_window_secs == 0 {
            return Err(WatchdogAnomalyError::InvalidSampleWindow { sample_window_secs });
        }

        Ok(Self {
            sample_id: sample_id.to_owned(),
            expected_deliveries,
            delivered_deliveries,
            active_peers,
            healthy_peers,
            sample_window_secs,
            targeted_peer_count,
        })
    }

    fn delivery_ratio_per_mille(&self) -> u16 {
        ((self.delivered_deliveries as u64) * 1000 / self.expected_deliveries as u64) as u16
    }

    fn liveness_ratio_per_mille(&self) -> u16 {
        ((self.healthy_peers as u64) * 1000 / self.active_peers as u64) as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchdogAnomalyReport {
    pub sample_id: String,
    pub kind: WatchdogAnomalyKind,
    pub severity: WatchdogAnomalySeverity,
    pub delivery_ratio_per_mille: u16,
    pub liveness_ratio_per_mille: u16,
    pub targeted_peer_count: u32,
    pub sample_window_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchdogAnomalyError {
    InvalidSampleId,
    InvalidExpectedDeliveries {
        expected_deliveries: u32,
    },
    InvalidSampleCounts {
        expected_deliveries: u32,
        delivered_deliveries: u32,
    },
    InvalidPeerCounts {
        active_peers: u32,
        healthy_peers: u32,
    },
    InvalidSampleWindow {
        sample_window_secs: u64,
    },
}

impl Display for WatchdogAnomalyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleId => write!(f, "watchdog anomaly sample id cannot be empty"),
            Self::InvalidExpectedDeliveries {
                expected_deliveries,
            } => write!(
                f,
                "watchdog anomaly expected deliveries must be positive, found {expected_deliveries}"
            ),
            Self::InvalidSampleCounts {
                expected_deliveries,
                delivered_deliveries,
            } => write!(
                f,
                "watchdog anomaly delivered deliveries {delivered_deliveries} exceed expected {expected_deliveries}"
            ),
            Self::InvalidPeerCounts {
                active_peers,
                healthy_peers,
            } => write!(
                f,
                "watchdog anomaly peer counts are invalid: active {active_peers}, healthy {healthy_peers}"
            ),
            Self::InvalidSampleWindow { sample_window_secs } => write!(
                f,
                "watchdog anomaly sample window must be positive, found {sample_window_secs}"
            ),
        }
    }
}

impl Error for WatchdogAnomalyError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WatchdogAnomalyEvaluator;

impl WatchdogAnomalyEvaluator {
    pub fn evaluate(
        &self,
        input: WatchdogAnomalyWatchInput,
    ) -> Result<WatchdogAnomalyReport, WatchdogAnomalyError> {
        let delivery_ratio_per_mille = input.delivery_ratio_per_mille();
        let liveness_ratio_per_mille = input.liveness_ratio_per_mille();

        let (kind, severity) = if input.targeted_peer_count >= 2 && delivery_ratio_per_mille <= 500
        {
            (
                WatchdogAnomalyKind::CensorshipSignal,
                WatchdogAnomalySeverity::Critical,
            )
        } else if input.targeted_peer_count >= 2 && delivery_ratio_per_mille <= 850 {
            (
                WatchdogAnomalyKind::CensorshipSignal,
                WatchdogAnomalySeverity::Warning,
            )
        } else if liveness_ratio_per_mille <= 500 {
            (
                WatchdogAnomalyKind::LivenessDegradation,
                WatchdogAnomalySeverity::Critical,
            )
        } else if liveness_ratio_per_mille < 1000 {
            (
                WatchdogAnomalyKind::LivenessDegradation,
                WatchdogAnomalySeverity::Warning,
            )
        } else {
            (WatchdogAnomalyKind::Nominal, WatchdogAnomalySeverity::Info)
        };

        Ok(WatchdogAnomalyReport {
            sample_id: input.sample_id,
            kind,
            severity,
            delivery_ratio_per_mille,
            liveness_ratio_per_mille,
            targeted_peer_count: input.targeted_peer_count,
            sample_window_secs: input.sample_window_secs,
        })
    }
}

pub fn evaluate_daemon_watchdog_anomaly(
    evaluator: &WatchdogAnomalyEvaluator,
    input: WatchdogAnomalyWatchInput,
) -> Result<WatchdogAnomalyReport, WatchdogAnomalyError> {
    evaluator.evaluate(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFaultSimulationInput {
    sample_id: String,
    peer_id: String,
    expected_deliveries: u32,
    delivered_deliveries: u32,
    active_peers: u32,
    healthy_peers: u32,
    sample_window_secs: u64,
    targeted_peer_count: u32,
    queue_capacity: usize,
    queued_events: usize,
}

impl NetworkFaultSimulationInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_id: &str,
        peer_id: &str,
        expected_deliveries: u32,
        delivered_deliveries: u32,
        active_peers: u32,
        healthy_peers: u32,
        sample_window_secs: u64,
        targeted_peer_count: u32,
        queue_capacity: usize,
        queued_events: usize,
    ) -> Result<Self, NetworkFaultSimulationError> {
        if sample_id.trim().is_empty() {
            return Err(NetworkFaultSimulationError::InvalidSampleId);
        }
        if peer_id.trim().is_empty() {
            return Err(NetworkFaultSimulationError::InvalidPeerId);
        }
        if queue_capacity == 0 {
            return Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 });
        }
        WatchdogAnomalyWatchInput::new(
            sample_id,
            expected_deliveries,
            delivered_deliveries,
            active_peers,
            healthy_peers,
            sample_window_secs,
            targeted_peer_count,
        )
        .map_err(NetworkFaultSimulationError::Watchdog)?;

        Ok(Self {
            sample_id: sample_id.to_owned(),
            peer_id: peer_id.to_owned(),
            expected_deliveries,
            delivered_deliveries,
            active_peers,
            healthy_peers,
            sample_window_secs,
            targeted_peer_count,
            queue_capacity,
            queued_events,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFaultSimulationReport {
    pub sample_id: String,
    pub final_lifecycle_state: PeerLifecycleState,
    pub queue_capacity: usize,
    pub queued_events: usize,
    pub queue_overflow_attempts: usize,
    pub watchdog_kind: WatchdogAnomalyKind,
    pub watchdog_severity: WatchdogAnomalySeverity,
    pub watchdog_delivery_ratio_per_mille: u16,
    pub watchdog_liveness_ratio_per_mille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkFaultSimulationError {
    InvalidSampleId,
    InvalidPeerId,
    InvalidQueueCapacity { capacity: usize },
    Lifecycle(RuntimeLifecycleError),
    Watchdog(WatchdogAnomalyError),
}

impl Display for NetworkFaultSimulationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleId => {
                write!(f, "network fault simulation sample id cannot be empty")
            }
            Self::InvalidPeerId => write!(f, "network fault simulation peer id cannot be empty"),
            Self::InvalidQueueCapacity { capacity } => write!(
                f,
                "network fault simulation queue capacity must be positive, found {capacity}"
            ),
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::Watchdog(error) => write!(f, "{error}"),
        }
    }
}

impl Error for NetworkFaultSimulationError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeterministicNetworkFaultSimulator {
    anomaly_evaluator: WatchdogAnomalyEvaluator,
}

impl DeterministicNetworkFaultSimulator {
    pub fn simulate(
        &self,
        input: NetworkFaultSimulationInput,
    ) -> Result<NetworkFaultSimulationReport, NetworkFaultSimulationError> {
        let mut lifecycle =
            PeerLifecycle::new(&input.peer_id).map_err(NetworkFaultSimulationError::Lifecycle)?;
        lifecycle
            .transition(PeerLifecycleEvent::StartConnect)
            .map_err(NetworkFaultSimulationError::Lifecycle)?;
        lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .map_err(NetworkFaultSimulationError::Lifecycle)?;
        if input.healthy_peers < input.active_peers {
            lifecycle
                .transition(PeerLifecycleEvent::HeartbeatMissed)
                .map_err(NetworkFaultSimulationError::Lifecycle)?;
        }

        let mut queue = BoundedRuntimeQueue::new(input.queue_capacity).map_err(|_| {
            NetworkFaultSimulationError::InvalidQueueCapacity {
                capacity: input.queue_capacity,
            }
        })?;
        let mut queue_overflow_attempts = 0usize;
        for event_index in 0..input.queued_events {
            if queue.enqueue(format!("fault-event-{event_index}")).is_err() {
                queue_overflow_attempts += 1;
            }
        }

        let watchdog_input = WatchdogAnomalyWatchInput::new(
            &input.sample_id,
            input.expected_deliveries,
            input.delivered_deliveries,
            input.active_peers,
            input.healthy_peers,
            input.sample_window_secs,
            input.targeted_peer_count,
        )
        .map_err(NetworkFaultSimulationError::Watchdog)?;
        let watchdog_report = self
            .anomaly_evaluator
            .evaluate(watchdog_input)
            .map_err(NetworkFaultSimulationError::Watchdog)?;

        Ok(NetworkFaultSimulationReport {
            sample_id: input.sample_id,
            final_lifecycle_state: lifecycle.state(),
            queue_capacity: input.queue_capacity,
            queued_events: input.queued_events,
            queue_overflow_attempts,
            watchdog_kind: watchdog_report.kind,
            watchdog_severity: watchdog_report.severity,
            watchdog_delivery_ratio_per_mille: watchdog_report.delivery_ratio_per_mille,
            watchdog_liveness_ratio_per_mille: watchdog_report.liveness_ratio_per_mille,
        })
    }
}

pub fn simulate_daemon_network_fault(
    simulator: &DeterministicNetworkFaultSimulator,
    input: NetworkFaultSimulationInput,
) -> Result<NetworkFaultSimulationReport, NetworkFaultSimulationError> {
    simulator.simulate(input)
}

fn is_valid_listener_did(value: &str) -> bool {
    is_valid_kamn_did(value)
}

fn is_valid_kamn_did(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.starts_with("kamn:did:")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStatus {
    RejoinAccepted,
    CatchUpRequired { from_version: u64, to_version: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryGuardError {
    InvalidNodeId,
    InvalidStateVersion,
    InvalidStateHash,
    InvalidResumeToken,
    ReplayResumeToken(String),
    StateVersionMismatch { expected: u64, found: u64 },
    StateHashMismatch { expected: String, found: String },
}

impl Display for RecoveryGuardError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNodeId => write!(f, "rejoin node id cannot be empty"),
            Self::InvalidStateVersion => write!(f, "rejoin state version must be positive"),
            Self::InvalidStateHash => write!(f, "rejoin state hash cannot be empty"),
            Self::InvalidResumeToken => write!(f, "rejoin resume token cannot be empty"),
            Self::ReplayResumeToken(token) => {
                write!(f, "rejoin resume token replayed: {token}")
            }
            Self::StateVersionMismatch { expected, found } => {
                write!(
                    f,
                    "rejoin state version mismatch: expected {expected}, found {found}"
                )
            }
            Self::StateHashMismatch { expected, found } => {
                write!(
                    f,
                    "rejoin state hash mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for RecoveryGuardError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRejoinGuard {
    expected_state_version: u64,
    expected_state_hash: String,
    consumed_resume_tokens: HashSet<String>,
}

impl RecoveryRejoinGuard {
    pub fn new(
        expected_state_version: u64,
        expected_state_hash: &str,
    ) -> Result<Self, RecoveryGuardError> {
        if expected_state_version == 0 {
            return Err(RecoveryGuardError::InvalidStateVersion);
        }
        if expected_state_hash.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidStateHash);
        }
        Ok(Self {
            expected_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
            consumed_resume_tokens: HashSet::new(),
        })
    }

    pub fn evaluate(
        &mut self,
        attempt: RejoinAttempt,
    ) -> Result<RecoveryStatus, RecoveryGuardError> {
        if self.consumed_resume_tokens.contains(attempt.resume_token()) {
            return Err(RecoveryGuardError::ReplayResumeToken(
                attempt.resume_token.clone(),
            ));
        }

        if attempt.state_version < self.expected_state_version {
            return Ok(RecoveryStatus::CatchUpRequired {
                from_version: attempt.state_version,
                to_version: self.expected_state_version,
            });
        }

        if attempt.state_version > self.expected_state_version {
            return Err(RecoveryGuardError::StateVersionMismatch {
                expected: self.expected_state_version,
                found: attempt.state_version,
            });
        }

        if attempt.state_hash != self.expected_state_hash {
            return Err(RecoveryGuardError::StateHashMismatch {
                expected: self.expected_state_hash.clone(),
                found: attempt.state_hash,
            });
        }

        self.consumed_resume_tokens.insert(attempt.resume_token);
        Ok(RecoveryStatus::RejoinAccepted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWiring {
    pub common_components: Vec<&'static str>,
    pub role_components: Vec<&'static str>,
}

impl RuntimeWiring {
    pub fn all_components(&self) -> Vec<&'static str> {
        let mut components = self.common_components.clone();
        components.extend(self.role_components.iter().copied());
        components
    }
}

pub fn build_runtime_wiring(config: &NodeConfig) -> RuntimeWiring {
    let common_components = vec!["state-store", "message-router", "audit-log", "api-surface"];

    let role_components = match config.role {
        NodeRole::Processor => vec!["mempool", "executor", "block-producer"],
        NodeRole::Listener => vec!["external-listener", "event-normalizer"],
        NodeRole::Approver => vec!["quorum-approver", "outbound-authorizer"],
    };

    RuntimeWiring {
        common_components,
        role_components,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        authorize_daemon_outbound_action, build_runtime_wiring, evaluate_daemon_state_divergence,
        evaluate_daemon_watchdog_anomaly, execute_processor_daemon_tick, ApproverAttestation,
        ApproverQuorumError, ApproverQuorumEvaluator, ApproverQuorumInput, BoundedRuntimeQueue,
        ConstructLockError, ConstructLockGuard, DeterministicNetworkFaultSimulator,
        DeterministicProposalPlanner, FileRuntimeSnapshotStore, InMemoryRuntimeSnapshotStore,
        ListenerAttestation, ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput,
        NetworkFaultSimulationError, NetworkFaultSimulationInput, PeerLifecycle,
        PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, ProposalPlannerError,
        RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
        RuntimeLifecycleError, RuntimeQueueError, RuntimeSnapshot, RuntimeSnapshotStore,
        SnapshotRestoreError, SnapshotRestoreGuard, SnapshotStoreError, StateDivergenceError,
        StateDivergenceEvaluator, StateDivergenceSeverity, StateDivergenceStatus,
        StateDivergenceWatchInput, WatchdogAnomalyError, WatchdogAnomalyEvaluator,
        WatchdogAnomalyKind, WatchdogAnomalySeverity, WatchdogAnomalyWatchInput,
    };
    use crate::config::{NodeConfig, NodeRole, SyncMode};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    fn sample_config(role: NodeRole) -> NodeConfig {
        NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        }
    }

    #[test]
    fn processor_wiring_contains_block_producer() {
        let wiring = build_runtime_wiring(&sample_config(NodeRole::Processor));
        assert!(wiring.role_components.contains(&"block-producer"));
    }

    #[test]
    fn listener_wiring_contains_external_listener() {
        let wiring = build_runtime_wiring(&sample_config(NodeRole::Listener));
        assert!(wiring.role_components.contains(&"external-listener"));
    }

    #[test]
    fn approver_wiring_contains_quorum_approver() {
        let wiring = build_runtime_wiring(&sample_config(NodeRole::Approver));
        assert!(wiring.role_components.contains(&"quorum-approver"));
    }

    #[test]
    fn functional_peer_lifecycle_allows_connect_heartbeat_recover_disconnect_flow() {
        let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
        assert_eq!(lifecycle.peer_id(), "peer-1");
        assert_eq!(lifecycle.state(), PeerLifecycleState::Disconnected);
        assert!(lifecycle
            .transition(PeerLifecycleEvent::StartConnect)
            .is_ok());
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HeartbeatMissed)
            .is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Degraded);
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HeartbeatRestored)
            .is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
        assert!(lifecycle.transition(PeerLifecycleEvent::Disconnect).is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Disconnected);
    }

    #[test]
    fn integration_bounded_runtime_queue_preserves_fifo_until_capacity() {
        let mut queue = BoundedRuntimeQueue::new(2).expect("queue should build");
        assert_eq!(queue.capacity(), 2);
        assert!(queue.is_empty());
        assert!(queue.enqueue("evt-1".to_owned()).is_ok());
        assert!(queue.enqueue("evt-2".to_owned()).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.dequeue(), Some("evt-1".to_owned()));
        assert_eq!(queue.dequeue(), Some("evt-2".to_owned()));
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn unit_rejects_invalid_peer_lifecycle_transition() {
        let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
        let error = lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .expect_err("handshake cannot complete before connect");
        assert_eq!(
            error,
            RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Disconnected,
                event: PeerLifecycleEvent::HandshakeSucceeded
            }
        );
    }

    #[test]
    fn regression_rejoin_without_disconnect_is_rejected() {
        // Regression: #324
        let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
        assert!(lifecycle
            .transition(PeerLifecycleEvent::StartConnect)
            .is_ok());
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .is_ok());
        let error = lifecycle
            .transition(PeerLifecycleEvent::Rejoin)
            .expect_err("rejoin should require disconnected state");
        assert_eq!(
            error,
            RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Active,
                event: PeerLifecycleEvent::Rejoin
            }
        );
    }

    #[test]
    fn regression_queue_overflow_rejects_new_event() {
        // Regression: #324
        let mut queue = BoundedRuntimeQueue::new(1).expect("queue should build");
        assert!(queue.enqueue("evt-1".to_owned()).is_ok());
        let error = queue
            .enqueue("evt-2".to_owned())
            .expect_err("second enqueue must overflow");
        assert_eq!(
            error,
            RuntimeQueueError::Overflow {
                capacity: 1,
                attempted_len: 2
            }
        );
    }

    #[test]
    fn unit_rejects_empty_peer_id() {
        assert_eq!(
            PeerLifecycle::new(""),
            Err(RuntimeLifecycleError::InvalidPeerId)
        );
    }

    #[test]
    fn unit_rejects_zero_queue_capacity() {
        assert_eq!(
            BoundedRuntimeQueue::<String>::new(0),
            Err(RuntimeQueueError::InvalidCapacity { capacity: 0 })
        );
    }

    #[test]
    fn functional_planner_orders_candidates_deterministically() {
        let candidates = vec![
            ProposalCandidate::new("tx-3", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
            ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            ProposalCandidate::new("tx-2", "did:kamn:agent:bbb", 1, "state-1").expect("valid"),
        ];

        let planner = DeterministicProposalPlanner::new("state-1");
        let plan = planner.plan(candidates).expect("plan should build");
        assert_eq!(
            plan.ordered_candidate_ids(),
            vec!["tx-1".to_owned(), "tx-2".to_owned(), "tx-3".to_owned()]
        );
    }

    #[test]
    fn integration_queue_drains_into_planner_without_order_loss() {
        let mut queue = BoundedRuntimeQueue::new(3).expect("queue should build");
        assert!(queue
            .enqueue(
                ProposalCandidate::new("tx-3", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
            )
            .is_ok());
        assert!(queue
            .enqueue(
                ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            )
            .is_ok());
        assert!(queue
            .enqueue(
                ProposalCandidate::new("tx-2", "did:kamn:agent:bbb", 1, "state-1").expect("valid"),
            )
            .is_ok());

        let mut drained = Vec::new();
        while let Some(candidate) = queue.dequeue() {
            drained.push(candidate);
        }

        let planner = DeterministicProposalPlanner::new("state-1");
        let plan = planner.plan(drained).expect("plan should build");
        assert_eq!(
            plan.ordered_candidate_ids(),
            vec!["tx-1".to_owned(), "tx-2".to_owned(), "tx-3".to_owned()]
        );
    }

    #[test]
    fn unit_rejects_empty_candidate_id() {
        let candidate = ProposalCandidate::new("", "did:kamn:agent:aaa", 1, "state-1");
        assert_eq!(candidate, Err(ProposalPlannerError::InvalidCandidateId));
    }

    #[test]
    fn regression_duplicate_candidate_id_is_rejected() {
        // Regression: #323
        let candidates = vec![
            ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            ProposalCandidate::new("tx-1", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
        ];
        let planner = DeterministicProposalPlanner::new("state-1");
        let error = planner
            .plan(candidates)
            .expect_err("duplicate candidate id must fail");
        assert_eq!(
            error,
            ProposalPlannerError::DuplicateCandidateId("tx-1".to_owned())
        );
    }

    #[test]
    fn regression_stale_state_hash_is_rejected() {
        // Regression: #323
        let candidates = vec![
            ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-2").expect("valid"),
        ];
        let planner = DeterministicProposalPlanner::new("state-1");
        let error = planner
            .plan(candidates)
            .expect_err("candidate state mismatch must fail");
        assert_eq!(
            error,
            ProposalPlannerError::StaleStateHash {
                expected: "state-1".to_owned(),
                found: "state-2".to_owned()
            }
        );
    }

    #[test]
    fn functional_rejoin_guard_accepts_matching_snapshot() {
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let attempt = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
        let status = guard.evaluate(attempt).expect("rejoin should be accepted");
        assert_eq!(status, RecoveryStatus::RejoinAccepted);
    }

    #[test]
    fn integration_rejoin_guard_emits_catch_up_required_for_lagging_node() {
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let attempt = RejoinAttempt::new("node-a", 40, "state-40", "resume-1").expect("valid");
        let status = guard
            .evaluate(attempt)
            .expect("lagging node should receive catch-up guidance");
        assert_eq!(
            status,
            RecoveryStatus::CatchUpRequired {
                from_version: 40,
                to_version: 42
            }
        );
    }

    #[test]
    fn unit_rejoin_guard_rejects_empty_resume_token() {
        let attempt = RejoinAttempt::new("node-a", 42, "state-42", "");
        assert_eq!(attempt, Err(RecoveryGuardError::InvalidResumeToken));
    }

    #[test]
    fn regression_rejoin_replay_token_is_rejected() {
        // Regression: #322
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let first = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
        assert_eq!(guard.evaluate(first), Ok(RecoveryStatus::RejoinAccepted));

        let replay = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
        let error = guard
            .evaluate(replay)
            .expect_err("replay token should be rejected");
        assert_eq!(
            error,
            RecoveryGuardError::ReplayResumeToken("resume-1".to_owned())
        );
    }

    #[test]
    fn regression_rejoin_state_hash_mismatch_is_rejected() {
        // Regression: #322
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let attempt = RejoinAttempt::new("node-a", 42, "state-41", "resume-1").expect("valid");
        let error = guard
            .evaluate(attempt)
            .expect_err("hash mismatch should be rejected");
        assert_eq!(
            error,
            RecoveryGuardError::StateHashMismatch {
                expected: "state-42".to_owned(),
                found: "state-41".to_owned()
            }
        );
    }

    #[test]
    fn functional_snapshot_restore_guard_accepts_matching_snapshot() {
        let guard =
            SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
        let snapshot = RuntimeSnapshot::new(42, "state-42").expect("snapshot should be valid");
        assert!(guard.validate(snapshot).is_ok());
    }

    #[test]
    fn unit_snapshot_restore_guard_rejects_invalid_state_hash() {
        let snapshot = RuntimeSnapshot::new(42, "");
        assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidStateHash));
    }

    #[test]
    fn regression_snapshot_restore_version_mismatch_is_rejected() {
        // Regression: #361
        let guard =
            SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
        let snapshot = RuntimeSnapshot::new(41, "state-42").expect("snapshot should be valid");
        let error = guard
            .validate(snapshot)
            .expect_err("version mismatch should be rejected");
        assert_eq!(
            error,
            SnapshotRestoreError::StateVersionMismatch {
                expected: 42,
                found: 41
            }
        );
    }

    #[test]
    fn regression_snapshot_restore_hash_mismatch_is_rejected() {
        // Regression: #361
        let guard =
            SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
        let snapshot = RuntimeSnapshot::new(42, "state-41").expect("snapshot should be valid");
        let error = guard
            .validate(snapshot)
            .expect_err("hash mismatch should be rejected");
        assert_eq!(
            error,
            SnapshotRestoreError::StateHashMismatch {
                expected: "state-42".to_owned(),
                found: "state-41".to_owned()
            }
        );
    }

    #[test]
    fn functional_snapshot_restore_guard_with_expected_cursor_accepts_matching_snapshot() {
        let guard = SnapshotRestoreGuard::with_expected_cursor(42, "state-42", 100)
            .expect("restore guard should construct");
        let snapshot =
            RuntimeSnapshot::with_cursor(42, "state-42", 100).expect("snapshot should be valid");
        assert!(guard.validate(snapshot).is_ok());
    }

    #[test]
    fn regression_snapshot_restore_cursor_mismatch_is_rejected() {
        // Regression: #617
        let guard = SnapshotRestoreGuard::with_expected_cursor(42, "state-42", 100)
            .expect("restore guard should construct");
        let snapshot =
            RuntimeSnapshot::with_cursor(42, "state-42", 99).expect("snapshot should be valid");
        let error = guard
            .validate(snapshot)
            .expect_err("cursor mismatch should be rejected");
        assert_eq!(
            error,
            SnapshotRestoreError::CursorMismatch {
                expected: 100,
                found: 99
            }
        );
    }

    #[test]
    fn functional_construct_lock_allows_acquire_then_renew_flow() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let renewed = lock
            .renew("processor-a", lease.fencing_token())
            .expect("lease renewal should succeed");
        assert!(renewed.fencing_token() > lease.fencing_token());
    }

    #[test]
    fn unit_construct_lock_rejects_empty_owner_id() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let error = lock
            .acquire_for("")
            .expect_err("empty owner id must be rejected");
        assert_eq!(error, ConstructLockError::InvalidOwnerId);
    }

    #[test]
    fn regression_split_brain_lock_acquisition_is_rejected() {
        // Regression: #362
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        assert!(lock.acquire_for("processor-a").is_ok());
        let error = lock
            .acquire_for("processor-b")
            .expect_err("second owner acquisition must be rejected");
        assert_eq!(
            error,
            ConstructLockError::LeaseAlreadyHeld {
                owner: "processor-a".to_owned()
            }
        );
    }

    #[test]
    fn regression_stale_lease_renewal_is_rejected() {
        // Regression: #362
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let error = lock
            .renew("processor-a", lease.fencing_token().saturating_sub(1))
            .expect_err("stale fencing token must be rejected");
        assert_eq!(
            error,
            ConstructLockError::StaleFencingToken {
                expected: lease.fencing_token(),
                found: lease.fencing_token().saturating_sub(1)
            }
        );
    }

    #[test]
    fn functional_construct_lock_supports_transfer_then_release_flow() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let transferred = lock
            .transfer("processor-a", "processor-b", lease.fencing_token())
            .expect("lease transfer should succeed");
        assert_eq!(transferred.owner_id(), "processor-b");
        assert!(transferred.fencing_token() > lease.fencing_token());
        assert!(lock
            .validate_execution_lease("processor-b", transferred.fencing_token())
            .is_ok());
        assert!(lock
            .release("processor-b", transferred.fencing_token())
            .is_ok());
    }

    #[test]
    fn unit_construct_lock_rejects_release_for_non_owner() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let error = lock
            .release("processor-b", lease.fencing_token())
            .expect_err("non-owner release must be rejected");
        assert_eq!(
            error,
            ConstructLockError::LeaseOwnerMismatch {
                expected: "processor-a".to_owned(),
                found: "processor-b".to_owned()
            }
        );
    }

    #[test]
    fn integration_daemon_tick_requires_matching_active_lease() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        assert_eq!(
            execute_processor_daemon_tick(&lock, "processor-a", lease.fencing_token(), 0),
            Ok(1)
        );
    }

    #[test]
    fn regression_unauthorized_transfer_is_rejected() {
        // Regression: #388
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let error = lock
            .transfer("processor-b", "processor-c", lease.fencing_token())
            .expect_err("unauthorized transfer must be rejected");
        assert_eq!(
            error,
            ConstructLockError::LeaseOwnerMismatch {
                expected: "processor-a".to_owned(),
                found: "processor-b".to_owned()
            }
        );
    }

    #[test]
    fn regression_daemon_tick_without_lease_is_rejected() {
        // Regression: #388
        let lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let error = execute_processor_daemon_tick(&lock, "processor-a", 1, 0)
            .expect_err("daemon execution without active lease must be rejected");
        assert_eq!(error, ConstructLockError::NoLeaseForExecution);
    }

    #[test]
    fn functional_listener_quorum_accepts_canonical_sufficient_attestations() {
        let mut evaluator =
            ListenerQuorumEvaluator::new(2).expect("listener quorum evaluator should build");
        let input = ListenerQuorumInput::new(
            "bridge-event-1",
            1,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-b", "att-2")
                    .expect("valid attestation"),
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");

        let decision = evaluator
            .evaluate(input)
            .expect("quorum should accept canonical listener attestations");
        assert!(decision.accepted);
        assert_eq!(decision.required_confirmations, 2);
        assert_eq!(decision.confirmed_listeners.len(), 2);
        assert_eq!(
            decision.confirmed_listeners,
            vec![
                "kamn:did:agent:listener-a".to_owned(),
                "kamn:did:agent:listener-b".to_owned()
            ]
        );
    }

    #[test]
    fn unit_listener_quorum_rejects_zero_required_confirmations() {
        let error =
            ListenerQuorumEvaluator::new(0).expect_err("zero quorum threshold must be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::InvalidRequiredConfirmations { required: 0 }
        );
    }

    #[test]
    fn integration_daemon_listener_quorum_rejects_replayed_event_sequence() {
        let mut evaluator =
            ListenerQuorumEvaluator::new(1).expect("listener quorum evaluator should build");
        let first = ListenerQuorumInput::new(
            "bridge-event-1",
            3,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        assert!(super::evaluate_daemon_listener_quorum(&mut evaluator, first).is_ok());

        let replay = ListenerQuorumInput::new(
            "bridge-event-1",
            3,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        let error = super::evaluate_daemon_listener_quorum(&mut evaluator, replay)
            .expect_err("replayed sequence should be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::ReplayedEventSequence {
                event_id: "bridge-event-1".to_owned(),
                previous_sequence: 3,
                received_sequence: 3
            }
        );
    }

    #[test]
    fn regression_duplicate_listener_attestation_replay_is_rejected() {
        // Regression: #371
        let mut evaluator =
            ListenerQuorumEvaluator::new(2).expect("listener quorum evaluator should build");
        let input = ListenerQuorumInput::new(
            "bridge-event-dup",
            1,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        let error = evaluator
            .evaluate(input)
            .expect_err("duplicate listener attestations must be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::DuplicateListenerAttestation {
                listener_did: "kamn:did:agent:listener-a".to_owned()
            }
        );
    }

    #[test]
    fn regression_replayed_listener_event_sequence_is_rejected() {
        // Regression: #371
        let mut evaluator =
            ListenerQuorumEvaluator::new(1).expect("listener quorum evaluator should build");
        let first = ListenerQuorumInput::new(
            "bridge-event-regression",
            7,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        assert!(evaluator.evaluate(first).is_ok());

        let replay = ListenerQuorumInput::new(
            "bridge-event-regression",
            6,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        let error = evaluator
            .evaluate(replay)
            .expect_err("stale/replayed sequence must be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::ReplayedEventSequence {
                event_id: "bridge-event-regression".to_owned(),
                previous_sequence: 7,
                received_sequence: 6
            }
        );
    }

    #[test]
    fn functional_approver_quorum_authorizes_outbound_with_threshold_attestations() {
        let evaluator =
            ApproverQuorumEvaluator::new(2).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-1",
            "payload-hash-1",
            vec![
                ApproverAttestation::new("kamn:did:agent:approver-a", "payload-hash-1", "att-1")
                    .expect("valid attestation"),
                ApproverAttestation::new("kamn:did:agent:approver-b", "payload-hash-1", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid outbound authorization input");

        let decision = evaluator
            .authorize(input)
            .expect("outbound action should be authorized");
        assert!(decision.authorized);
        assert_eq!(decision.required_approvals, 2);
        assert_eq!(
            decision.approved_by,
            vec![
                "kamn:did:agent:approver-a".to_owned(),
                "kamn:did:agent:approver-b".to_owned()
            ]
        );
    }

    #[test]
    fn unit_approver_quorum_rejects_zero_required_approvals() {
        let error =
            ApproverQuorumEvaluator::new(0).expect_err("zero required approvals must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::InvalidRequiredApprovals { required: 0 }
        );
    }

    #[test]
    fn integration_daemon_outbound_approver_quorum_rejects_under_threshold() {
        let evaluator =
            ApproverQuorumEvaluator::new(2).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-under-threshold",
            "payload-hash-2",
            vec![
                ApproverAttestation::new("kamn:did:agent:approver-a", "payload-hash-2", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid outbound authorization input");
        let error = authorize_daemon_outbound_action(&evaluator, input)
            .expect_err("under-threshold approvals must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::InsufficientApprovals {
                required: 2,
                received: 1
            }
        );
    }

    #[test]
    fn regression_malformed_approver_payload_is_rejected() {
        // Regression: #372
        let evaluator =
            ApproverQuorumEvaluator::new(1).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-malformed",
            "payload-hash-expected",
            vec![ApproverAttestation::new(
                "kamn:did:agent:approver-a",
                "payload-hash-tampered",
                "att-1",
            )
            .expect("valid attestation")],
        )
        .expect("valid outbound authorization input");
        let error = evaluator
            .authorize(input)
            .expect_err("payload mismatch must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::PayloadDigestMismatch {
                expected: "payload-hash-expected".to_owned(),
                found: "payload-hash-tampered".to_owned()
            }
        );
    }

    #[test]
    fn regression_outbound_under_quorum_is_rejected() {
        // Regression: #372
        let evaluator =
            ApproverQuorumEvaluator::new(3).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-regression",
            "payload-hash-regression",
            vec![
                ApproverAttestation::new(
                    "kamn:did:agent:approver-a",
                    "payload-hash-regression",
                    "att-1",
                )
                .expect("valid attestation"),
                ApproverAttestation::new(
                    "kamn:did:agent:approver-b",
                    "payload-hash-regression",
                    "att-2",
                )
                .expect("valid attestation"),
            ],
        )
        .expect("valid outbound authorization input");
        let error = evaluator
            .authorize(input)
            .expect_err("under-threshold approvals must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::InsufficientApprovals {
                required: 3,
                received: 2
            }
        );
    }

    #[test]
    fn functional_divergence_watchdog_flags_hash_mismatch_as_critical() {
        let evaluator = StateDivergenceEvaluator;
        let input = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            42,
            42,
            "state-hash-expected",
            "state-hash-observed",
            110,
        )
        .expect("valid divergence input");

        let report = evaluator
            .evaluate(input)
            .expect("hash mismatch should emit divergence report");
        assert_eq!(report.status, StateDivergenceStatus::Diverged);
        assert_eq!(report.severity, StateDivergenceSeverity::Critical);
    }

    #[test]
    fn unit_divergence_watchdog_rejects_incomplete_evidence_payload() {
        let error = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            42,
            42,
            "state-hash-expected",
            "",
            110,
        )
        .expect_err("empty observed hash must be rejected");
        assert_eq!(
            error,
            StateDivergenceError::IncompleteEvidenceField {
                field: "observed_state_hash"
            }
        );
    }

    #[test]
    fn integration_daemon_divergence_report_includes_deterministic_evidence_fields() {
        let evaluator = StateDivergenceEvaluator;
        let input = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            42,
            42,
            "state-hash-expected",
            "state-hash-observed",
            110,
        )
        .expect("valid divergence input");

        let report = evaluate_daemon_state_divergence(&evaluator, input)
            .expect("daemon divergence evaluation should succeed");
        assert_eq!(report.evidence.peer_id, "kamn:did:agent:validator-a");
        assert_eq!(report.evidence.expected_state_version, 42);
        assert_eq!(report.evidence.observed_state_version, 42);
        assert_eq!(report.evidence.expected_state_hash, "state-hash-expected");
        assert_eq!(report.evidence.observed_state_hash, "state-hash-observed");
        assert_eq!(report.evidence.observed_at_tick, 110);
        assert_eq!(
            report.incident_fingerprint,
            "state-divergence:kamn:did:agent:validator-a:42:42:state-hash-expected:state-hash-observed"
        );
    }

    #[test]
    fn regression_state_divergence_false_negative_is_rejected() {
        // Regression: #381
        let evaluator = StateDivergenceEvaluator;
        let input = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            99,
            99,
            "state-hash-expected",
            "state-hash-mismatched",
            220,
        )
        .expect("valid divergence input");

        let report = evaluate_daemon_state_divergence(&evaluator, input)
            .expect("mismatch must produce divergence report");
        assert_eq!(report.status, StateDivergenceStatus::Diverged);
        assert_ne!(
            report.evidence.expected_state_hash,
            report.evidence.observed_state_hash
        );
    }

    #[test]
    fn functional_watchdog_anomaly_classifies_liveness_degradation_as_warning() {
        let evaluator = WatchdogAnomalyEvaluator;
        let input = WatchdogAnomalyWatchInput::new("sample-liveness", 100, 96, 7, 5, 30, 1)
            .expect("valid anomaly sample");
        let report = evaluator
            .evaluate(input)
            .expect("anomaly classification should succeed");
        assert_eq!(report.kind, WatchdogAnomalyKind::LivenessDegradation);
        assert_eq!(report.severity, WatchdogAnomalySeverity::Warning);
    }

    #[test]
    fn unit_watchdog_anomaly_rejects_invalid_delivery_sample() {
        let error = WatchdogAnomalyWatchInput::new("sample-invalid", 10, 12, 5, 5, 30, 2)
            .expect_err("delivered count above expected must be rejected");
        assert_eq!(
            error,
            WatchdogAnomalyError::InvalidSampleCounts {
                expected_deliveries: 10,
                delivered_deliveries: 12
            }
        );
    }

    #[test]
    fn integration_daemon_watchdog_anomaly_report_includes_summary_fields() {
        let evaluator = WatchdogAnomalyEvaluator;
        let input = WatchdogAnomalyWatchInput::new("sample-censorship", 100, 45, 8, 8, 60, 3)
            .expect("valid anomaly sample");
        let report = evaluate_daemon_watchdog_anomaly(&evaluator, input)
            .expect("daemon anomaly evaluation should succeed");
        assert_eq!(report.sample_id, "sample-censorship");
        assert_eq!(report.kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.severity, WatchdogAnomalySeverity::Critical);
        assert_eq!(report.delivery_ratio_per_mille, 450);
        assert_eq!(report.targeted_peer_count, 3);
        assert_eq!(report.sample_window_secs, 60);
    }

    #[test]
    fn regression_censorship_edge_signal_remains_detected_as_critical() {
        // Regression: #382
        let evaluator = WatchdogAnomalyEvaluator;
        let input = WatchdogAnomalyWatchInput::new("sample-regression", 200, 98, 12, 12, 60, 2)
            .expect("valid anomaly sample");
        let report = evaluate_daemon_watchdog_anomaly(&evaluator, input)
            .expect("edge censorship signal should be classified");
        assert_eq!(report.kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.severity, WatchdogAnomalySeverity::Critical);
    }

    #[test]
    fn unit_network_fault_simulation_rejects_zero_queue_capacity() {
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-invalid",
            "peer-sim-a",
            100,
            99,
            6,
            6,
            30,
            1,
            0,
            2,
        );
        assert_eq!(
            input,
            Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 })
        );
    }

    #[test]
    fn functional_network_fault_simulation_classifies_targeted_packet_loss_as_critical() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-censorship",
            "peer-sim-a",
            100,
            45,
            8,
            8,
            60,
            3,
            8,
            8,
        )
        .expect("valid simulation input");
        let report = simulator
            .simulate(input)
            .expect("simulation should classify targeted packet loss");

        assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Critical);
        assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Active);
        assert_eq!(report.queue_overflow_attempts, 0);
    }

    #[test]
    fn integration_daemon_network_fault_simulation_reports_overflow_and_degradation() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-overflow",
            "peer-sim-b",
            120,
            110,
            6,
            4,
            30,
            1,
            2,
            5,
        )
        .expect("valid simulation input");
        let report = super::simulate_daemon_network_fault(&simulator, input)
            .expect("simulation should pass");

        assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Degraded);
        assert_eq!(report.queue_overflow_attempts, 3);
        assert_eq!(
            report.watchdog_kind,
            WatchdogAnomalyKind::LivenessDegradation
        );
    }

    #[test]
    fn regression_network_fault_simulation_keeps_censorship_critical_boundary() {
        // Regression: #618
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-regression",
            "peer-sim-c",
            200,
            100,
            12,
            12,
            60,
            2,
            4,
            4,
        )
        .expect("valid simulation input");
        let report = simulator
            .simulate(input)
            .expect("simulation should classify censorship boundary");

        assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Critical);
    }

    #[test]
    fn performance_network_fault_simulation_pr_lane_stays_within_budget() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let start = Instant::now();
        for sample_index in 0..256 {
            let input = NetworkFaultSimulationInput::new(
                &format!("fault-sample-perf-{sample_index}"),
                "peer-sim-perf",
                100,
                98,
                16,
                16,
                30,
                1,
                8,
                8,
            )
            .expect("valid simulation input");
            assert!(simulator.simulate(input).is_ok());
        }
        let elapsed_millis = start.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "network fault simulation PR lane exceeded budget: {elapsed_millis}ms"
        );
    }

    #[test]
    #[ignore = "scheduled chaos lane"]
    fn performance_network_fault_simulation_chaos_lane_stress() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        for sample_index in 0..5000 {
            let targeted_peer_count = if sample_index % 4 == 0 { 3 } else { 1 };
            let delivered = if targeted_peer_count == 3 { 45 } else { 99 };
            let healthy_peers = if sample_index % 3 == 0 { 8 } else { 10 };
            let input = NetworkFaultSimulationInput::new(
                &format!("fault-sample-chaos-{sample_index}"),
                "peer-sim-chaos",
                100,
                delivered,
                10,
                healthy_peers,
                30,
                targeted_peer_count,
                16,
                24,
            )
            .expect("valid simulation input");
            assert!(simulator.simulate(input).is_ok());
        }
    }

    #[test]
    fn functional_in_memory_snapshot_store_round_trips_snapshots() {
        let mut store = InMemoryRuntimeSnapshotStore::default();
        assert!(store.list().expect("list should succeed").is_empty());

        let snapshot_1 = RuntimeSnapshot::new(41, "state-41").expect("valid snapshot");
        let snapshot_2 = RuntimeSnapshot::new(42, "state-42").expect("valid snapshot");
        assert!(store.write(snapshot_1).is_ok());
        assert!(store.write(snapshot_2.clone()).is_ok());

        let latest = store.read_latest().expect("read_latest should succeed");
        assert_eq!(latest, Some(snapshot_2));
        assert_eq!(store.list().expect("list should succeed").len(), 2);
    }

    #[test]
    fn integration_file_snapshot_store_round_trips_snapshots() {
        let path = temp_snapshot_store_path("roundtrip");
        let _ = fs::remove_file(&path);

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let snapshot_1 = RuntimeSnapshot::new(41, "state-41").expect("valid snapshot");
        let snapshot_2 = RuntimeSnapshot::new(42, "state-42").expect("valid snapshot");
        assert!(store.write(snapshot_1).is_ok());
        assert!(store.write(snapshot_2.clone()).is_ok());

        let latest = store.read_latest().expect("read_latest should succeed");
        assert_eq!(latest, Some(snapshot_2));
        assert_eq!(store.list().expect("list should succeed").len(), 2);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_rejects_malformed_payload() {
        // Regression: #387
        let path = temp_snapshot_store_path("malformed");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "not-a-valid-snapshot-line\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let error = store
            .list()
            .expect_err("malformed payload must be rejected");
        assert_eq!(
            error,
            SnapshotStoreError::InvalidPayload("not-a-valid-snapshot-line".to_owned())
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn unit_file_snapshot_store_recovery_handles_missing_snapshot_file() {
        let path = temp_snapshot_store_path("recover-missing");
        let _ = fs::remove_file(&path);

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert!(result.latest.is_none());
        assert_eq!(result.recovered_entries, 0);
        assert_eq!(result.dropped_corrupt_entries, 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_snapshot_store_recovery_recovers_latest_after_trailing_corruption() {
        let path = temp_snapshot_store_path("recover-trailing-corruption");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41\n42|state-42\n43|\n").is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::InvalidPayload("43|".to_owned()))
        );

        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(
            result.latest,
            Some(RuntimeSnapshot::new(42, "state-42").expect("valid snapshot"))
        );
        assert_eq!(result.recovered_entries, 2);
        assert_eq!(result.dropped_corrupt_entries, 1);
        assert_eq!(store.list().expect("list should succeed").len(), 2);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn integration_file_snapshot_store_recovery_allows_append_after_restart() {
        let path = temp_snapshot_store_path("recover-restart");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41\n42|state-42\n43|\n").is_ok());

        let mut first_store =
            FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let first_recovery = first_store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(first_recovery.recovered_entries, 2);

        let mut restarted_store =
            FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let next_snapshot = RuntimeSnapshot::new(43, "state-43").expect("valid snapshot");
        assert!(restarted_store.write(next_snapshot.clone()).is_ok());
        assert_eq!(
            restarted_store.read_latest().expect("read should pass"),
            Some(next_snapshot)
        );
        assert_eq!(restarted_store.list().expect("list should pass").len(), 3);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_recovery_truncates_corrupt_suffix() {
        // Regression: #617
        let path = temp_snapshot_store_path("recover-corrupt-suffix");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41\n42|state-42\nbroken\n43|state-43\n").is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(
            result.latest,
            Some(RuntimeSnapshot::new(42, "state-42").expect("valid snapshot"))
        );
        assert_eq!(result.recovered_entries, 2);
        assert_eq!(result.dropped_corrupt_entries, 2);
        assert_eq!(
            fs::read_to_string(&path).expect("snapshot file should be readable"),
            "41|state-41|41\n42|state-42|42\n"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn unit_runtime_snapshot_with_cursor_rejects_zero_cursor() {
        let snapshot = RuntimeSnapshot::with_cursor(42, "state-42", 0);
        assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidCursor));
    }

    #[test]
    fn unit_runtime_snapshot_rejects_hash_with_metadata_delimiter() {
        let snapshot = RuntimeSnapshot::new(42, "state-42|100");
        assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidStateHash));
    }

    #[test]
    fn unit_in_memory_snapshot_store_rejects_state_version_regression() {
        let mut store = InMemoryRuntimeSnapshotStore::default();
        let baseline = RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot");
        assert!(store.write(baseline).is_ok());
        let stale = RuntimeSnapshot::with_cursor(40, "state-40", 101).expect("valid snapshot");
        assert_eq!(
            store.write(stale),
            Err(SnapshotStoreError::StateVersionRegression {
                previous: 41,
                found: 40
            })
        );
    }

    #[test]
    fn unit_in_memory_snapshot_store_rejects_cursor_regression() {
        let mut store = InMemoryRuntimeSnapshotStore::default();
        let baseline = RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot");
        assert!(store.write(baseline).is_ok());
        let stale = RuntimeSnapshot::with_cursor(42, "state-42", 99).expect("valid snapshot");
        assert_eq!(
            store.write(stale),
            Err(SnapshotStoreError::CursorRegression {
                previous: 100,
                found: 99
            })
        );
    }

    #[test]
    fn regression_file_snapshot_store_rejects_version_regression_metadata() {
        // Regression: #617
        let path = temp_snapshot_store_path("version-regression");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n40|state-40|101\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::StateVersionRegression {
                previous: 41,
                found: 40
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_rejects_cursor_regression_metadata() {
        // Regression: #617
        let path = temp_snapshot_store_path("cursor-regression");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n42|state-42|99\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::CursorRegression {
                previous: 100,
                found: 99
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_rejects_stale_hash_metadata() {
        // Regression: #617
        let path = temp_snapshot_store_path("hash-regression");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n42|state-41|101\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::StaleStateHash {
                state_hash: "state-41".to_owned(),
                previous_version: 41,
                found_version: 42
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_snapshot_store_recovery_truncates_stale_metadata_suffix() {
        let path = temp_snapshot_store_path("recover-stale-metadata");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n42|state-42|99\n43|state-43|102\n").is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(
            result.latest,
            Some(RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot"))
        );
        assert_eq!(result.recovered_entries, 1);
        assert_eq!(result.dropped_corrupt_entries, 2);
        assert_eq!(
            fs::read_to_string(&path).expect("snapshot file should be readable"),
            "41|state-41|100\n"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn performance_file_snapshot_store_recovery_scan_stays_within_ci_budget() {
        let path = temp_snapshot_store_path("recover-performance");
        let _ = fs::remove_file(&path);
        let mut payload = String::new();
        for state_version in 1..=256 {
            payload.push_str(&format!("{state_version}|state-{state_version}\n"));
        }
        payload.push_str("broken\n");
        assert!(fs::write(&path, payload).is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let start = Instant::now();
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        let elapsed_millis = start.elapsed().as_millis();
        assert_eq!(result.recovered_entries, 256);
        assert_eq!(result.dropped_corrupt_entries, 1);
        assert!(
            elapsed_millis < 250,
            "snapshot recovery exceeded CI budget: {elapsed_millis}ms"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    #[ignore = "scheduled snapshot deep lane"]
    fn performance_file_snapshot_store_recovery_deep_lane_large_payload() {
        let path = temp_snapshot_store_path("recover-deep-lane");
        let _ = fs::remove_file(&path);
        let mut payload = String::new();
        for state_version in 1..=8192 {
            payload.push_str(&format!(
                "{state_version}|state-{state_version}|{state_version}\n"
            ));
        }
        payload.push_str("8193|state-8193|0\n");
        assert!(fs::write(&path, payload).is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let start = Instant::now();
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        let elapsed_millis = start.elapsed().as_millis();
        assert_eq!(result.recovered_entries, 8192);
        assert_eq!(result.dropped_corrupt_entries, 1);
        assert!(
            elapsed_millis < 2000,
            "snapshot deep-lane recovery exceeded budget: {elapsed_millis}ms"
        );

        let _ = fs::remove_file(path);
    }

    fn temp_snapshot_store_path(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-runtime-snapshot-{tag}-{nonce}.log"))
    }
}
