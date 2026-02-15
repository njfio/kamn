use super::{
    is_valid_kamn_did, DeterministicBackpressureController, RuntimeBackpressureAction,
    RuntimeBackpressureDecision, RuntimeBackpressureError, RuntimeBackpressureInput,
};
use crate::config::{NodeConfig, NodeRole};
use crate::signature_profile::baseline_signature_for_fields;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Peer lifecycle state.
pub enum PeerLifecycleState {
    /// Disconnected.
    Disconnected,
    /// Connecting.
    Connecting,
    /// Active.
    Active,
    /// Degraded.
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Peer lifecycle event.
pub enum PeerLifecycleEvent {
    /// Start connect.
    StartConnect,
    /// Handshake succeeded.
    HandshakeSucceeded,
    /// Heartbeat missed.
    HeartbeatMissed,
    /// Heartbeat restored.
    HeartbeatRestored,
    /// Disconnect.
    Disconnect,
    /// Rejoin.
    Rejoin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime lifecycle error.
pub enum RuntimeLifecycleError {
    /// Invalid peer id.
    InvalidPeerId,
    /// Invalid transition.
    InvalidTransition {
        /// From.
        from: PeerLifecycleState,
        /// Event.
        event: PeerLifecycleEvent,
    },
}

impl RuntimeLifecycleError {
    /// Handles reason code.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidPeerId => "runtime_peer_id_invalid",
            Self::InvalidTransition { .. } => "runtime_peer_transition_invalid",
        }
    }
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
/// Peer lifecycle.
pub struct PeerLifecycle {
    peer_id: String,
    state: PeerLifecycleState,
}

impl PeerLifecycle {
    /// Handles new.
    pub fn new(peer_id: &str) -> Result<Self, RuntimeLifecycleError> {
        if peer_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::InvalidPeerId);
        }
        Ok(Self {
            peer_id: peer_id.to_owned(),
            state: PeerLifecycleState::Disconnected,
        })
    }

    /// Handles peer id.
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Handles state.
    pub fn state(&self) -> PeerLifecycleState {
        self.state
    }

    /// Handles transition.
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
/// Runtime queue error.
pub enum RuntimeQueueError {
    /// Invalid capacity.
    InvalidCapacity {
        /// Capacity.
        capacity: usize,
    },
    /// Overflow.
    Overflow {
        /// Capacity.
        capacity: usize,
        /// Attempted len.
        attempted_len: usize,
    },
    /// Backpressure input validation failed.
    BackpressureInput(RuntimeBackpressureError),
    /// Backpressure rejected enqueue for saturation.
    BackpressureRejected {
        /// Deterministic reason code for operational telemetry.
        reason_code: &'static str,
        /// Queue utilization at decision time.
        queue_utilization_per_mille: u16,
    },
    /// Backpressure purged stale disconnected queue.
    BackpressurePurgedStalePeerQueue {
        /// Deterministic reason code for operational telemetry.
        reason_code: &'static str,
        /// Number of queued items purged.
        purged_entries: usize,
    },
}

impl RuntimeQueueError {
    /// Returns deterministic queue/backpressure reason codes.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidCapacity { .. } => "runtime_queue_invalid_capacity",
            Self::Overflow { .. } => "runtime_queue_overflow",
            Self::BackpressureInput(error) => error.reason_code(),
            Self::BackpressureRejected { reason_code, .. } => reason_code,
            Self::BackpressurePurgedStalePeerQueue { reason_code, .. } => reason_code,
        }
    }
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
            Self::BackpressureInput(error) => write!(f, "{error}"),
            Self::BackpressureRejected {
                queue_utilization_per_mille,
                ..
            } => write!(
                f,
                "runtime queue enqueue rejected by backpressure at {queue_utilization_per_mille} per mille utilization"
            ),
            Self::BackpressurePurgedStalePeerQueue { purged_entries, .. } => write!(
                f,
                "runtime queue stale peer purge triggered by backpressure; purged {purged_entries} queued entries"
            ),
        }
    }
}

impl Error for RuntimeQueueError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Bounded runtime queue.
pub struct BoundedRuntimeQueue<T> {
    capacity: usize,
    entries: VecDeque<T>,
}

impl<T> BoundedRuntimeQueue<T> {
    /// Handles new.
    pub fn new(capacity: usize) -> Result<Self, RuntimeQueueError> {
        if capacity == 0 {
            return Err(RuntimeQueueError::InvalidCapacity { capacity });
        }
        Ok(Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        })
    }

    /// Handles capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Handles len.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Handles is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Handles enqueue.
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

    /// Evaluates deterministic backpressure and applies action before queue mutation.
    pub fn enqueue_with_backpressure(
        &mut self,
        item: T,
        controller: &DeterministicBackpressureController,
        peer_id: &str,
        lifecycle_state: PeerLifecycleState,
    ) -> Result<RuntimeBackpressureDecision, RuntimeQueueError> {
        let input = RuntimeBackpressureInput::new(
            peer_id,
            self.entries.len(),
            self.capacity,
            lifecycle_state,
        )
        .map_err(RuntimeQueueError::BackpressureInput)?;
        let decision = controller
            .evaluate(input)
            .map_err(RuntimeQueueError::BackpressureInput)?;

        match decision.action {
            RuntimeBackpressureAction::Accept | RuntimeBackpressureAction::SlowProducer => {
                self.enqueue(item)?;
                Ok(decision)
            }
            RuntimeBackpressureAction::RejectNewEnqueue => {
                Err(RuntimeQueueError::BackpressureRejected {
                    reason_code: decision.reason_code(),
                    queue_utilization_per_mille: decision.queue_utilization_per_mille,
                })
            }
            RuntimeBackpressureAction::PurgeStalePeerQueue => {
                let purged_entries = self.entries.len();
                self.entries.clear();
                Err(RuntimeQueueError::BackpressurePurgedStalePeerQueue {
                    reason_code: decision.reason_code(),
                    purged_entries,
                })
            }
        }
    }

    /// Handles dequeue.
    pub fn dequeue(&mut self) -> Option<T> {
        self.entries.pop_front()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Authenticated peer frame error.
pub enum AuthenticatedPeerFrameError {
    /// Invalid frame id.
    InvalidFrameId,
    /// Invalid sender did.
    InvalidSenderDid,
    /// Invalid recipient did.
    InvalidRecipientDid,
    /// Invalid local peer did.
    InvalidLocalPeerDid,
    /// Empty allowed senders.
    EmptyAllowedSenders,
    /// Invalid nonce.
    InvalidNonce,
    /// Empty payload.
    EmptyPayload,
    /// Empty signature.
    EmptySignature,
    /// Invalid wire field delimiter.
    InvalidWireFieldDelimiter {
        /// Field.
        field: &'static str,
    },
    /// Invalid wire format.
    InvalidWireFormat(String),
    /// Signature mismatch.
    SignatureMismatch {
        /// Expected.
        expected: String,
        /// Found.
        found: String,
    },
    /// Unauthorized sender.
    UnauthorizedSender(String),
    /// Wrong recipient.
    WrongRecipient {
        /// Expected.
        expected: String,
        /// Found.
        found: String,
    },
    /// Replay nonce.
    ReplayNonce {
        /// Sender did.
        sender_did: String,
        /// Last nonce.
        last_nonce: u64,
        /// Found.
        found: u64,
    },
}

impl Display for AuthenticatedPeerFrameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFrameId => write!(f, "peer frame id cannot be empty"),
            Self::InvalidSenderDid => write!(f, "peer frame sender DID is invalid"),
            Self::InvalidRecipientDid => write!(f, "peer frame recipient DID is invalid"),
            Self::InvalidLocalPeerDid => write!(f, "local peer DID is invalid"),
            Self::EmptyAllowedSenders => write!(f, "allowed sender DID set cannot be empty"),
            Self::InvalidNonce => write!(f, "peer frame nonce must be positive"),
            Self::EmptyPayload => write!(f, "peer frame payload cannot be empty"),
            Self::EmptySignature => write!(f, "peer frame signature cannot be empty"),
            Self::InvalidWireFieldDelimiter { field } => write!(
                f,
                "peer frame field contains unsupported wire delimiters: {field}"
            ),
            Self::InvalidWireFormat(payload) => {
                write!(f, "peer frame wire payload is invalid: {payload}")
            }
            Self::SignatureMismatch { expected, found } => {
                write!(
                    f,
                    "peer frame signature mismatch: expected {expected}, found {found}"
                )
            }
            Self::UnauthorizedSender(value) => {
                write!(f, "peer frame sender is unauthorized: {value}")
            }
            Self::WrongRecipient { expected, found } => write!(
                f,
                "peer frame recipient mismatch: expected {expected}, found {found}"
            ),
            Self::ReplayNonce {
                sender_did,
                last_nonce,
                found,
            } => write!(
                f,
                "peer frame nonce replay for {sender_did}: last {last_nonce}, found {found}"
            ),
        }
    }
}

impl Error for AuthenticatedPeerFrameError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Authenticated peer frame.
pub struct AuthenticatedPeerFrame {
    frame_id: String,
    sender_peer_did: String,
    recipient_peer_did: String,
    nonce: u64,
    payload: String,
    signature: String,
}

impl AuthenticatedPeerFrame {
    /// Handles new.
    pub fn new(
        frame_id: &str,
        sender_peer_did: &str,
        recipient_peer_did: &str,
        nonce: u64,
        payload: &str,
        signature: &str,
    ) -> Result<Self, AuthenticatedPeerFrameError> {
        if frame_id.trim().is_empty() {
            return Err(AuthenticatedPeerFrameError::InvalidFrameId);
        }
        if !is_valid_kamn_did(sender_peer_did) {
            return Err(AuthenticatedPeerFrameError::InvalidSenderDid);
        }
        if !is_valid_kamn_did(recipient_peer_did) {
            return Err(AuthenticatedPeerFrameError::InvalidRecipientDid);
        }
        if nonce == 0 {
            return Err(AuthenticatedPeerFrameError::InvalidNonce);
        }
        if payload.trim().is_empty() {
            return Err(AuthenticatedPeerFrameError::EmptyPayload);
        }
        if signature.trim().is_empty() {
            return Err(AuthenticatedPeerFrameError::EmptySignature);
        }
        ensure_peer_frame_wire_field(frame_id, "frame_id")?;
        ensure_peer_frame_wire_field(sender_peer_did, "sender_peer_did")?;
        ensure_peer_frame_wire_field(recipient_peer_did, "recipient_peer_did")?;
        ensure_peer_frame_wire_field(payload, "payload")?;
        ensure_peer_frame_wire_field(signature, "signature")?;

        Ok(Self {
            frame_id: frame_id.to_owned(),
            sender_peer_did: sender_peer_did.to_owned(),
            recipient_peer_did: recipient_peer_did.to_owned(),
            nonce,
            payload: payload.to_owned(),
            signature: signature.to_owned(),
        })
    }

    /// Handles signed.
    pub fn signed(
        frame_id: &str,
        sender_peer_did: &str,
        recipient_peer_did: &str,
        nonce: u64,
        payload: &str,
    ) -> Result<Self, AuthenticatedPeerFrameError> {
        let signature =
            expected_peer_frame_signature(sender_peer_did, recipient_peer_did, nonce, payload);
        Self::new(
            frame_id,
            sender_peer_did,
            recipient_peer_did,
            nonce,
            payload,
            &signature,
        )
    }

    /// Handles frame id.
    pub fn frame_id(&self) -> &str {
        &self.frame_id
    }

    /// Handles sender peer did.
    pub fn sender_peer_did(&self) -> &str {
        &self.sender_peer_did
    }

    /// Handles recipient peer did.
    pub fn recipient_peer_did(&self) -> &str {
        &self.recipient_peer_did
    }

    /// Handles nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Handles payload.
    pub fn payload(&self) -> &str {
        &self.payload
    }

    /// Handles signature.
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Handles verify signature.
    pub fn verify_signature(&self) -> Result<(), AuthenticatedPeerFrameError> {
        let expected = expected_peer_frame_signature(
            &self.sender_peer_did,
            &self.recipient_peer_did,
            self.nonce,
            &self.payload,
        );
        if self.signature != expected {
            return Err(AuthenticatedPeerFrameError::SignatureMismatch {
                expected,
                found: self.signature.clone(),
            });
        }
        Ok(())
    }

    /// Handles to wire.
    pub fn to_wire(&self) -> Result<String, AuthenticatedPeerFrameError> {
        ensure_peer_frame_wire_field(&self.frame_id, "frame_id")?;
        ensure_peer_frame_wire_field(&self.sender_peer_did, "sender_peer_did")?;
        ensure_peer_frame_wire_field(&self.recipient_peer_did, "recipient_peer_did")?;
        ensure_peer_frame_wire_field(&self.payload, "payload")?;
        ensure_peer_frame_wire_field(&self.signature, "signature")?;
        Ok(format!(
            "frame|{}|{}|{}|{}|{}|{}",
            self.frame_id,
            self.sender_peer_did,
            self.recipient_peer_did,
            self.nonce,
            self.payload,
            self.signature
        ))
    }

    /// Handles from wire.
    pub fn from_wire(raw: &str) -> Result<Self, AuthenticatedPeerFrameError> {
        let mut segments = raw.split('|');
        let Some(prefix) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        let Some(frame_id) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        let Some(sender_peer_did) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        let Some(recipient_peer_did) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        let Some(nonce_raw) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        let Some(payload) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        let Some(signature) = segments.next() else {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        };
        if prefix != "frame" || segments.next().is_some() {
            return Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                raw.to_owned(),
            ));
        }

        let nonce = nonce_raw
            .parse::<u64>()
            .map_err(|_| AuthenticatedPeerFrameError::InvalidWireFormat(raw.to_owned()))?;

        Self::new(
            frame_id,
            sender_peer_did,
            recipient_peer_did,
            nonce,
            payload,
            signature,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Peer frame authenticator.
pub struct PeerFrameAuthenticator {
    local_peer_did: String,
    allowed_sender_dids: BTreeSet<String>,
    last_nonce_by_sender: BTreeMap<String, u64>,
}

impl PeerFrameAuthenticator {
    /// Handles new.
    pub fn new(
        local_peer_did: &str,
        allowed_sender_dids: Vec<String>,
    ) -> Result<Self, AuthenticatedPeerFrameError> {
        if !is_valid_kamn_did(local_peer_did) {
            return Err(AuthenticatedPeerFrameError::InvalidLocalPeerDid);
        }
        if allowed_sender_dids.is_empty() {
            return Err(AuthenticatedPeerFrameError::EmptyAllowedSenders);
        }

        let mut allowlist = BTreeSet::new();
        for sender_did in allowed_sender_dids {
            if !is_valid_kamn_did(&sender_did) {
                return Err(AuthenticatedPeerFrameError::InvalidSenderDid);
            }
            allowlist.insert(sender_did);
        }

        Ok(Self {
            local_peer_did: local_peer_did.to_owned(),
            allowed_sender_dids: allowlist,
            last_nonce_by_sender: BTreeMap::new(),
        })
    }

    /// Handles validate inbound.
    pub fn validate_inbound(
        &mut self,
        frame: &AuthenticatedPeerFrame,
    ) -> Result<(), AuthenticatedPeerFrameError> {
        frame.verify_signature()?;

        if frame.recipient_peer_did != self.local_peer_did {
            return Err(AuthenticatedPeerFrameError::WrongRecipient {
                expected: self.local_peer_did.clone(),
                found: frame.recipient_peer_did.clone(),
            });
        }

        if !self.allowed_sender_dids.contains(&frame.sender_peer_did) {
            return Err(AuthenticatedPeerFrameError::UnauthorizedSender(
                frame.sender_peer_did.clone(),
            ));
        }

        if let Some(last_nonce) = self.last_nonce_by_sender.get(&frame.sender_peer_did) {
            if frame.nonce <= *last_nonce {
                return Err(AuthenticatedPeerFrameError::ReplayNonce {
                    sender_did: frame.sender_peer_did.clone(),
                    last_nonce: *last_nonce,
                    found: frame.nonce,
                });
            }
        }

        self.last_nonce_by_sender
            .insert(frame.sender_peer_did.clone(), frame.nonce);
        Ok(())
    }
}

fn expected_peer_frame_signature(
    sender_peer_did: &str,
    recipient_peer_did: &str,
    nonce: u64,
    payload: &str,
) -> String {
    baseline_signature_for_fields(sender_peer_did, nonce, recipient_peer_did, payload)
}

fn ensure_peer_frame_wire_field(
    value: &str,
    field: &'static str,
) -> Result<(), AuthenticatedPeerFrameError> {
    if value.contains('|') || value.contains('\n') || value.contains('\r') {
        return Err(AuthenticatedPeerFrameError::InvalidWireFieldDelimiter { field });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proposal candidate.
pub struct ProposalCandidate {
    id: String,
    sender_did: String,
    nonce: u64,
    state_hash: String,
}

impl ProposalCandidate {
    /// Handles new.
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

    /// Handles id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Handles sender did.
    pub fn sender_did(&self) -> &str {
        &self.sender_did
    }

    /// Handles nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Handles state hash.
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proposal plan.
pub struct ProposalPlan {
    ordered_candidates: Vec<ProposalCandidate>,
}

impl ProposalPlan {
    /// Handles ordered candidates.
    pub fn ordered_candidates(&self) -> &[ProposalCandidate] {
        &self.ordered_candidates
    }

    /// Handles ordered candidate ids.
    pub fn ordered_candidate_ids(&self) -> Vec<String> {
        self.ordered_candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proposal planner error.
pub enum ProposalPlannerError {
    /// Invalid candidate id.
    InvalidCandidateId,
    /// Invalid sender did.
    InvalidSenderDid,
    /// Invalid state hash.
    InvalidStateHash,
    /// Invalid nonce.
    InvalidNonce,
    /// Duplicate candidate id.
    DuplicateCandidateId(String),
    /// Stale state hash.
    StaleStateHash {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
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
/// Deterministic proposal planner.
pub struct DeterministicProposalPlanner {
    expected_state_hash: String,
}

impl DeterministicProposalPlanner {
    /// Handles new.
    pub fn new(expected_state_hash: &str) -> Self {
        Self {
            expected_state_hash: expected_state_hash.to_owned(),
        }
    }

    /// Handles plan.
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
/// Runtime wiring.
pub struct RuntimeWiring {
    /// Common components.
    pub common_components: Vec<&'static str>,
    /// Role components.
    pub role_components: Vec<&'static str>,
}

impl RuntimeWiring {
    /// Handles all components.
    pub fn all_components(&self) -> Vec<&'static str> {
        let mut components = self.common_components.clone();
        components.extend(self.role_components.iter().copied());
        components
    }
}

/// Handles build runtime wiring.
pub fn build_runtime_wiring(config: &NodeConfig) -> RuntimeWiring {
    let mut common_components = vec!["state-store", "message-router", "audit-log", "api-surface"];
    if config.enable_gossip {
        common_components.push("p2p-discovery");
        common_components.push("p2p-gossip-transport");
        common_components.push("p2p-libp2p-swarm-stack");
        common_components.push("p2p-libp2p-harness-ready");
    } else {
        common_components.push("gossip-transport-disabled");
    }

    let role_components = match config.role {
        NodeRole::Processor => vec![
            "mempool",
            "executor",
            "block-producer",
            "consensus-validator",
        ],
        NodeRole::Listener => vec!["external-listener", "event-normalizer"],
        NodeRole::Approver => vec!["quorum-approver", "outbound-authorizer"],
    };

    RuntimeWiring {
        common_components,
        role_components,
    }
}
