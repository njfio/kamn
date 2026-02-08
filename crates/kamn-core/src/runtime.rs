use crate::config::{NodeConfig, NodeRole};
use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};

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
}

impl RuntimeSnapshot {
    pub fn new(state_version: u64, state_hash: &str) -> Result<Self, SnapshotRestoreError> {
        if state_version == 0 {
            return Err(SnapshotRestoreError::InvalidStateVersion);
        }
        if state_hash.trim().is_empty() {
            return Err(SnapshotRestoreError::InvalidStateHash);
        }
        Ok(Self {
            state_version,
            state_hash: state_hash.to_owned(),
        })
    }

    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotRestoreError {
    InvalidStateVersion,
    InvalidStateHash,
    StateVersionMismatch { expected: u64, found: u64 },
    StateHashMismatch { expected: String, found: String },
}

impl Display for SnapshotRestoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStateVersion => write!(f, "snapshot state version must be positive"),
            Self::InvalidStateHash => write!(f, "snapshot state hash cannot be empty"),
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
        }
    }
}

impl Error for SnapshotRestoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRestoreGuard {
    expected_state_version: u64,
    expected_state_hash: String,
}

impl SnapshotRestoreGuard {
    pub fn new(
        expected_state_version: u64,
        expected_state_hash: &str,
    ) -> Result<Self, SnapshotRestoreError> {
        if expected_state_version == 0 {
            return Err(SnapshotRestoreError::InvalidStateVersion);
        }
        if expected_state_hash.trim().is_empty() {
            return Err(SnapshotRestoreError::InvalidStateHash);
        }
        Ok(Self {
            expected_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
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
        Ok(())
    }
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
        build_runtime_wiring, BoundedRuntimeQueue, DeterministicProposalPlanner, PeerLifecycle,
        PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, ProposalPlannerError,
        RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
        RuntimeLifecycleError, RuntimeQueueError, RuntimeSnapshot, SnapshotRestoreError,
        SnapshotRestoreGuard,
    };
    use crate::config::{NodeConfig, NodeRole, SyncMode};

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
}
