use crate::config::{NodeConfig, NodeRole};
use crate::signature_profile::baseline_signature_for_fields;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[path = "runtime_backpressure.rs"]
mod runtime_backpressure;

pub use runtime_backpressure::{
    DeterministicBackpressureController, RuntimeBackpressureAction, RuntimeBackpressureDecision,
    RuntimeBackpressureError, RuntimeBackpressureInput, RuntimeBackpressurePolicy,
};

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
/// Rejoin attempt.
pub struct RejoinAttempt {
    node_id: String,
    state_version: u64,
    state_hash: String,
    resume_token: String,
}

impl RejoinAttempt {
    /// Handles new.
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

    /// Handles node id.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Handles state version.
    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    /// Handles state hash.
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    /// Handles resume token.
    pub fn resume_token(&self) -> &str {
        &self.resume_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime snapshot.
pub struct RuntimeSnapshot {
    state_version: u64,
    state_hash: String,
    cursor: u64,
}

impl RuntimeSnapshot {
    /// Handles new.
    pub fn new(state_version: u64, state_hash: &str) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(state_version, state_hash, state_version)
    }

    /// Handles with cursor.
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

    /// Handles state version.
    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    /// Handles state hash.
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    /// Handles cursor.
    pub fn cursor(&self) -> u64 {
        self.cursor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot restore error.
pub enum SnapshotRestoreError {
    /// Invalid state version.
    InvalidStateVersion,
    /// Invalid state hash.
    InvalidStateHash,
    /// Invalid cursor.
    InvalidCursor,
    /// State version mismatch.
    StateVersionMismatch {
        /// Expected state version.
        expected: u64,
        /// Observed state version.
        found: u64,
    },
    /// State hash mismatch.
    StateHashMismatch {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
    /// Cursor mismatch.
    CursorMismatch {
        /// Expected cursor value.
        expected: u64,
        /// Observed cursor value.
        found: u64,
    },
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
/// Snapshot restore guard.
pub struct SnapshotRestoreGuard {
    expected_state_version: u64,
    expected_state_hash: String,
    expected_cursor: Option<u64>,
}

impl SnapshotRestoreGuard {
    /// Handles new.
    pub fn new(
        expected_state_version: u64,
        expected_state_hash: &str,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(expected_state_version, expected_state_hash, None)
    }

    /// Handles with expected cursor.
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

    /// Handles validate.
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
/// Snapshot store error.
pub enum SnapshotStoreError {
    /// Io.
    Io(String),
    /// Invalid payload.
    InvalidPayload(String),
    /// State version regression.
    StateVersionRegression {
        /// Previous.
        previous: u64,
        /// Found.
        found: u64,
    },
    /// Cursor regression.
    CursorRegression {
        /// Previous.
        previous: u64,
        /// Found.
        found: u64,
    },
    /// Stale state hash.
    StaleStateHash {
        /// State hash.
        state_hash: String,
        /// Previous version.
        previous_version: u64,
        /// Found version.
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

/// Runtime snapshot store.
pub trait RuntimeSnapshotStore {
    /// Handles write.
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError>;
    /// Handles read latest.
    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError>;
    /// Handles list.
    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// In memory runtime snapshot store.
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
/// File runtime snapshot store.
pub struct FileRuntimeSnapshotStore {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot recovery result.
pub struct SnapshotRecoveryResult {
    /// Latest.
    pub latest: Option<RuntimeSnapshot>,
    /// Recovered entries.
    pub recovered_entries: usize,
    /// Dropped corrupt entries.
    pub dropped_corrupt_entries: usize,
}

impl FileRuntimeSnapshotStore {
    /// Handles new.
    pub fn new(path: PathBuf) -> Result<Self, SnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(SnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        Ok(Self { path })
    }

    /// Handles recover latest and repair.
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
/// Construct lock lease.
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

    /// Handles owner id.
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    /// Handles fencing token.
    pub fn fencing_token(&self) -> u64 {
        self.fencing_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Construct lock error.
pub enum ConstructLockError {
    /// Invalid lease ttl.
    InvalidLeaseTtl,
    /// Invalid owner id.
    InvalidOwnerId,
    /// No active lease.
    NoActiveLease,
    /// No lease for execution.
    NoLeaseForExecution,
    /// Lease already held.
    LeaseAlreadyHeld {
        /// Current lock owner id.
        owner: String,
    },
    /// Lease owner mismatch.
    LeaseOwnerMismatch {
        /// Expected owner id.
        expected: String,
        /// Observed owner id.
        found: String,
    },
    /// Stale fencing token.
    StaleFencingToken {
        /// Expected fencing token.
        expected: u64,
        /// Observed fencing token.
        found: u64,
    },
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
/// Construct lock guard.
pub struct ConstructLockGuard {
    lease_ttl_ticks: u64,
    current_lease: Option<ConstructLockLease>,
}

impl ConstructLockGuard {
    /// Handles new.
    pub fn new(lease_ttl_ticks: u64) -> Result<Self, ConstructLockError> {
        if lease_ttl_ticks == 0 {
            return Err(ConstructLockError::InvalidLeaseTtl);
        }
        Ok(Self {
            lease_ttl_ticks,
            current_lease: None,
        })
    }

    /// Handles lease ttl ticks.
    pub fn lease_ttl_ticks(&self) -> u64 {
        self.lease_ttl_ticks
    }

    /// Handles acquire for.
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

    /// Handles renew.
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

    /// Handles release.
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

    /// Handles transfer.
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

    /// Handles validate execution lease.
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

/// Handles execute processor daemon tick.
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
/// Listener attestation.
pub struct ListenerAttestation {
    listener_did: String,
    attestation_id: String,
}

impl ListenerAttestation {
    /// Handles new.
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

    /// Handles listener did.
    pub fn listener_did(&self) -> &str {
        &self.listener_did
    }

    /// Handles attestation id.
    pub fn attestation_id(&self) -> &str {
        &self.attestation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum input.
pub struct ListenerQuorumInput {
    event_id: String,
    event_sequence: u64,
    attestations: Vec<ListenerAttestation>,
}

impl ListenerQuorumInput {
    /// Handles new.
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

    /// Handles event id.
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    /// Handles event sequence.
    pub fn event_sequence(&self) -> u64 {
        self.event_sequence
    }

    /// Handles attestations.
    pub fn attestations(&self) -> &[ListenerAttestation] {
        &self.attestations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum decision.
pub struct ListenerQuorumDecision {
    /// Event id.
    pub event_id: String,
    /// Event sequence.
    pub event_sequence: u64,
    /// Required confirmations.
    pub required_confirmations: usize,
    /// Confirmed listeners.
    pub confirmed_listeners: Vec<String>,
    /// Accepted.
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum error.
pub enum ListenerQuorumError {
    /// Invalid required confirmations.
    InvalidRequiredConfirmations {
        /// Required.
        required: usize,
    },
    /// Invalid event id.
    InvalidEventId,
    /// Invalid event sequence.
    InvalidEventSequence,
    /// Invalid listener did.
    InvalidListenerDid,
    /// Invalid attestation id.
    InvalidAttestationId,
    /// Duplicate listener attestation.
    DuplicateListenerAttestation {
        /// Listener did.
        listener_did: String,
    },
    /// Replayed event sequence.
    ReplayedEventSequence {
        /// Event id.
        event_id: String,
        /// Previous sequence.
        previous_sequence: u64,
        /// Received sequence.
        received_sequence: u64,
    },
    /// Insufficient confirmations.
    InsufficientConfirmations {
        /// Required.
        required: usize,
        /// Received.
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
/// Listener quorum evaluator.
pub struct ListenerQuorumEvaluator {
    required_confirmations: usize,
    latest_sequence_by_event: BTreeMap<String, u64>,
}

impl ListenerQuorumEvaluator {
    /// Handles new.
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

    /// Handles evaluate.
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

/// Handles evaluate daemon listener quorum.
pub fn evaluate_daemon_listener_quorum(
    evaluator: &mut ListenerQuorumEvaluator,
    input: ListenerQuorumInput,
) -> Result<ListenerQuorumDecision, ListenerQuorumError> {
    evaluator.evaluate(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver attestation.
pub struct ApproverAttestation {
    approver_did: String,
    payload_digest: String,
    attestation_id: String,
}

impl ApproverAttestation {
    /// Handles new.
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

    /// Handles approver did.
    pub fn approver_did(&self) -> &str {
        &self.approver_did
    }

    /// Handles payload digest.
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum input.
pub struct ApproverQuorumInput {
    action_id: String,
    payload_digest: String,
    attestations: Vec<ApproverAttestation>,
}

impl ApproverQuorumInput {
    /// Handles new.
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

    /// Handles action id.
    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    /// Handles payload digest.
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }

    /// Handles attestations.
    pub fn attestations(&self) -> &[ApproverAttestation] {
        &self.attestations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum decision.
pub struct ApproverQuorumDecision {
    /// Action id.
    pub action_id: String,
    /// Required approvals.
    pub required_approvals: usize,
    /// Approved by.
    pub approved_by: Vec<String>,
    /// Authorized.
    pub authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum error.
pub enum ApproverQuorumError {
    /// Invalid required approvals.
    InvalidRequiredApprovals {
        /// Required approval threshold.
        required: usize,
    },
    /// Invalid action id.
    InvalidActionId,
    /// Invalid payload digest.
    InvalidPayloadDigest,
    /// Invalid approver did.
    InvalidApproverDid,
    /// Invalid attestation id.
    InvalidAttestationId,
    /// Duplicate approver attestation.
    DuplicateApproverAttestation {
        /// Approver DID that duplicated an attestation.
        approver_did: String,
    },
    /// Payload digest mismatch.
    PayloadDigestMismatch {
        /// Expected payload digest.
        expected: String,
        /// Observed payload digest.
        found: String,
    },
    /// Insufficient approvals.
    InsufficientApprovals {
        /// Required approval threshold.
        required: usize,
        /// Number of received approvals.
        received: usize,
    },
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
/// Approver quorum evaluator.
pub struct ApproverQuorumEvaluator {
    required_approvals: usize,
}

impl ApproverQuorumEvaluator {
    /// Handles new.
    pub fn new(required_approvals: usize) -> Result<Self, ApproverQuorumError> {
        if required_approvals == 0 {
            return Err(ApproverQuorumError::InvalidRequiredApprovals {
                required: required_approvals,
            });
        }
        Ok(Self { required_approvals })
    }

    /// Handles authorize.
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

/// Handles authorize daemon outbound action.
pub fn authorize_daemon_outbound_action(
    evaluator: &ApproverQuorumEvaluator,
    input: ApproverQuorumInput,
) -> Result<ApproverQuorumDecision, ApproverQuorumError> {
    evaluator.authorize(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State divergence status.
pub enum StateDivergenceStatus {
    /// In sync.
    InSync,
    /// Diverged.
    Diverged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State divergence severity.
pub enum StateDivergenceSeverity {
    /// Info.
    Info,
    /// Critical.
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence evidence.
pub struct StateDivergenceEvidence {
    /// Peer id.
    pub peer_id: String,
    /// Expected state version.
    pub expected_state_version: u64,
    /// Observed state version.
    pub observed_state_version: u64,
    /// Expected state hash.
    pub expected_state_hash: String,
    /// Observed state hash.
    pub observed_state_hash: String,
    /// Observed at tick.
    pub observed_at_tick: u64,
}

impl StateDivergenceEvidence {
    /// Handles new.
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
/// State divergence watch input.
pub struct StateDivergenceWatchInput {
    evidence: StateDivergenceEvidence,
}

impl StateDivergenceWatchInput {
    /// Handles new.
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

    /// Handles evidence.
    pub fn evidence(&self) -> &StateDivergenceEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence report.
pub struct StateDivergenceReport {
    /// Status.
    pub status: StateDivergenceStatus,
    /// Severity.
    pub severity: StateDivergenceSeverity,
    /// Incident fingerprint.
    pub incident_fingerprint: String,
    /// Evidence.
    pub evidence: StateDivergenceEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence error.
pub enum StateDivergenceError {
    /// Invalid peer did.
    InvalidPeerDid,
    /// Invalid state version.
    InvalidStateVersion {
        /// Field name associated with the invalid value.
        field: &'static str,
        /// Provided state version value.
        value: u64,
    },
    /// Incomplete evidence field.
    IncompleteEvidenceField {
        /// Missing or empty evidence field name.
        field: &'static str,
    },
    /// Invalid observed tick.
    InvalidObservedTick {
        /// Observed daemon tick value.
        tick: u64,
    },
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
/// State divergence evaluator.
pub struct StateDivergenceEvaluator;

impl StateDivergenceEvaluator {
    /// Handles evaluate.
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

/// Handles evaluate daemon state divergence.
pub fn evaluate_daemon_state_divergence(
    evaluator: &StateDivergenceEvaluator,
    input: StateDivergenceWatchInput,
) -> Result<StateDivergenceReport, StateDivergenceError> {
    evaluator.evaluate(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Watchdog anomaly kind.
pub enum WatchdogAnomalyKind {
    /// Nominal.
    Nominal,
    /// Liveness degradation.
    LivenessDegradation,
    /// Censorship signal.
    CensorshipSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Watchdog anomaly severity.
pub enum WatchdogAnomalySeverity {
    /// Info.
    Info,
    /// Warning.
    Warning,
    /// Critical.
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Watchdog anomaly watch input.
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
    /// Handles new.
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
/// Watchdog anomaly report.
pub struct WatchdogAnomalyReport {
    /// Sample id.
    pub sample_id: String,
    /// Kind.
    pub kind: WatchdogAnomalyKind,
    /// Severity.
    pub severity: WatchdogAnomalySeverity,
    /// Delivery ratio per mille.
    pub delivery_ratio_per_mille: u16,
    /// Liveness ratio per mille.
    pub liveness_ratio_per_mille: u16,
    /// Targeted peer count.
    pub targeted_peer_count: u32,
    /// Sample window secs.
    pub sample_window_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Watchdog anomaly error.
pub enum WatchdogAnomalyError {
    /// Invalid sample id.
    InvalidSampleId,
    /// Invalid expected deliveries.
    InvalidExpectedDeliveries {
        /// Expected deliveries.
        expected_deliveries: u32,
    },
    /// Invalid sample counts.
    InvalidSampleCounts {
        /// Expected deliveries.
        expected_deliveries: u32,
        /// Delivered deliveries.
        delivered_deliveries: u32,
    },
    /// Invalid peer counts.
    InvalidPeerCounts {
        /// Active peers.
        active_peers: u32,
        /// Healthy peers.
        healthy_peers: u32,
    },
    /// Invalid sample window.
    InvalidSampleWindow {
        /// Sample window secs.
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
/// Watchdog anomaly evaluator.
pub struct WatchdogAnomalyEvaluator;

impl WatchdogAnomalyEvaluator {
    /// Handles evaluate.
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

/// Handles evaluate daemon watchdog anomaly.
pub fn evaluate_daemon_watchdog_anomaly(
    evaluator: &WatchdogAnomalyEvaluator,
    input: WatchdogAnomalyWatchInput,
) -> Result<WatchdogAnomalyReport, WatchdogAnomalyError> {
    evaluator.evaluate(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Network fault simulation input.
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
    /// Handles new.
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
/// Network fault simulation report.
pub struct NetworkFaultSimulationReport {
    /// Sample id.
    pub sample_id: String,
    /// Final lifecycle state.
    pub final_lifecycle_state: PeerLifecycleState,
    /// Queue capacity.
    pub queue_capacity: usize,
    /// Queued events.
    pub queued_events: usize,
    /// Queue overflow attempts.
    pub queue_overflow_attempts: usize,
    /// Backpressure action observed most recently during queue mutation.
    pub backpressure_last_action: RuntimeBackpressureAction,
    /// Backpressure reason marker observed most recently during queue mutation.
    pub backpressure_last_reason_code: &'static str,
    /// Number of queue enqueue attempts rejected by deterministic backpressure.
    pub backpressure_rejected_events: usize,
    /// Number of queued entries purged due to stale disconnected peer policy.
    pub backpressure_purged_events: usize,
    /// Number of queue enqueue attempts accepted in slow-producer mode.
    pub backpressure_slow_events: usize,
    /// Watchdog kind.
    pub watchdog_kind: WatchdogAnomalyKind,
    /// Watchdog severity.
    pub watchdog_severity: WatchdogAnomalySeverity,
    /// Watchdog delivery ratio per mille.
    pub watchdog_delivery_ratio_per_mille: u16,
    /// Watchdog liveness ratio per mille.
    pub watchdog_liveness_ratio_per_mille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Network fault simulation error.
pub enum NetworkFaultSimulationError {
    /// Invalid sample id.
    InvalidSampleId,
    /// Invalid peer id.
    InvalidPeerId,
    /// Invalid queue capacity.
    InvalidQueueCapacity {
        /// Queue capacity value.
        capacity: usize,
    },
    /// Lifecycle.
    Lifecycle(RuntimeLifecycleError),
    /// Backpressure.
    Backpressure(RuntimeBackpressureError),
    /// Watchdog.
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
            Self::Backpressure(error) => write!(f, "{error}"),
            Self::Watchdog(error) => write!(f, "{error}"),
        }
    }
}

impl Error for NetworkFaultSimulationError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Deterministic network fault simulator.
pub struct DeterministicNetworkFaultSimulator {
    anomaly_evaluator: WatchdogAnomalyEvaluator,
}

impl DeterministicNetworkFaultSimulator {
    /// Handles simulate.
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
        if input.active_peers > 0 && input.healthy_peers == 0 {
            lifecycle
                .transition(PeerLifecycleEvent::Disconnect)
                .map_err(NetworkFaultSimulationError::Lifecycle)?;
        }

        let backpressure_policy = RuntimeBackpressurePolicy::new(700, 900, true)
            .map_err(NetworkFaultSimulationError::Backpressure)?;
        let backpressure_controller = DeterministicBackpressureController::new(backpressure_policy);
        let backpressure_peer_id = if is_valid_kamn_did(&input.peer_id) {
            input.peer_id.clone()
        } else {
            format!("kamn:did:agent:{}", input.peer_id)
        };

        let mut queue = BoundedRuntimeQueue::new(input.queue_capacity).map_err(|_| {
            NetworkFaultSimulationError::InvalidQueueCapacity {
                capacity: input.queue_capacity,
            }
        })?;
        let mut queue_overflow_attempts = 0usize;
        let mut backpressure_last_action = RuntimeBackpressureAction::Accept;
        let mut backpressure_last_reason_code = "runtime_backpressure_accept";
        let mut backpressure_rejected_events = 0usize;
        let mut backpressure_purged_events = 0usize;
        let mut backpressure_slow_events = 0usize;
        for event_index in 0..input.queued_events {
            match queue.enqueue_with_backpressure(
                format!("fault-event-{event_index}"),
                &backpressure_controller,
                &backpressure_peer_id,
                lifecycle.state(),
            ) {
                Ok(decision) => {
                    if decision.action == RuntimeBackpressureAction::SlowProducer {
                        backpressure_slow_events += 1;
                    }
                    backpressure_last_action = decision.action;
                    backpressure_last_reason_code = decision.reason_code();
                }
                Err(RuntimeQueueError::BackpressureRejected { reason_code, .. }) => {
                    queue_overflow_attempts += 1;
                    backpressure_rejected_events += 1;
                    backpressure_last_action = RuntimeBackpressureAction::RejectNewEnqueue;
                    backpressure_last_reason_code = reason_code;
                }
                Err(RuntimeQueueError::BackpressurePurgedStalePeerQueue {
                    reason_code,
                    purged_entries,
                }) => {
                    queue_overflow_attempts += 1;
                    backpressure_purged_events += purged_entries;
                    backpressure_last_action = RuntimeBackpressureAction::PurgeStalePeerQueue;
                    backpressure_last_reason_code = reason_code;
                }
                Err(RuntimeQueueError::BackpressureInput(error)) => {
                    return Err(NetworkFaultSimulationError::Backpressure(error));
                }
                Err(RuntimeQueueError::Overflow { .. }) => {
                    queue_overflow_attempts += 1;
                    backpressure_last_action = RuntimeBackpressureAction::RejectNewEnqueue;
                    backpressure_last_reason_code = "runtime_queue_overflow";
                }
                Err(RuntimeQueueError::InvalidCapacity { capacity }) => {
                    return Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity });
                }
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
            backpressure_last_action,
            backpressure_last_reason_code,
            backpressure_rejected_events,
            backpressure_purged_events,
            backpressure_slow_events,
            watchdog_kind: watchdog_report.kind,
            watchdog_severity: watchdog_report.severity,
            watchdog_delivery_ratio_per_mille: watchdog_report.delivery_ratio_per_mille,
            watchdog_liveness_ratio_per_mille: watchdog_report.liveness_ratio_per_mille,
        })
    }
}

/// Handles simulate daemon network fault.
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
/// Recovery status.
pub enum RecoveryStatus {
    /// Rejoin accepted.
    RejoinAccepted,
    /// Catch up required.
    CatchUpRequired {
        /// Current local state version.
        from_version: u64,
        /// Target remote state version.
        to_version: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recovery guard error.
pub enum RecoveryGuardError {
    /// Invalid node id.
    InvalidNodeId,
    /// Invalid state version.
    InvalidStateVersion,
    /// Invalid state hash.
    InvalidStateHash,
    /// Invalid resume token.
    InvalidResumeToken,
    /// Replay resume token.
    ReplayResumeToken(String),
    /// State version mismatch.
    StateVersionMismatch {
        /// Expected state version.
        expected: u64,
        /// Observed state version.
        found: u64,
    },
    /// State hash mismatch.
    StateHashMismatch {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
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
/// Recovery rejoin guard.
pub struct RecoveryRejoinGuard {
    expected_state_version: u64,
    expected_state_hash: String,
    consumed_resume_tokens: HashSet<String>,
}

impl RecoveryRejoinGuard {
    /// Handles new.
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

    /// Handles evaluate.
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
    } else {
        common_components.push("gossip-transport-disabled");
    }

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
        ApproverQuorumError, ApproverQuorumEvaluator, ApproverQuorumInput, AuthenticatedPeerFrame,
        AuthenticatedPeerFrameError, BoundedRuntimeQueue, ConstructLockError, ConstructLockGuard,
        DeterministicBackpressureController, DeterministicNetworkFaultSimulator,
        DeterministicProposalPlanner, FileRuntimeSnapshotStore, InMemoryRuntimeSnapshotStore,
        ListenerAttestation, ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput,
        NetworkFaultSimulationError, NetworkFaultSimulationInput, PeerFrameAuthenticator,
        PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate,
        ProposalPlannerError, RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus,
        RejoinAttempt, RuntimeBackpressureAction, RuntimeBackpressureDecision,
        RuntimeBackpressureError, RuntimeBackpressureInput, RuntimeBackpressurePolicy,
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
    fn unit_runtime_backpressure_policy_rejects_invalid_threshold_order() {
        assert_eq!(
            RuntimeBackpressurePolicy::new(900, 900, true),
            Err(RuntimeBackpressureError::InvalidThresholdOrder {
                slow_threshold_per_mille: 900,
                reject_threshold_per_mille: 900
            })
        );
    }

    #[test]
    fn functional_runtime_backpressure_classifies_queue_saturation() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let input = RuntimeBackpressureInput::new(
            "kamn:did:agent:peer-a",
            8,
            10,
            PeerLifecycleState::Active,
        )
        .expect("valid input");
        let decision = controller.evaluate(input).expect("evaluation should pass");
        assert_eq!(decision.action, RuntimeBackpressureAction::SlowProducer);
        assert_eq!(decision.queue_utilization_per_mille, 800);
    }

    #[test]
    fn integration_runtime_backpressure_purges_stale_disconnected_peer_queue() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let input = RuntimeBackpressureInput::new(
            "kamn:did:agent:peer-b",
            3,
            10,
            PeerLifecycleState::Disconnected,
        )
        .expect("valid input");
        let decision = controller.evaluate(input).expect("evaluation should pass");
        assert_eq!(
            decision.action,
            RuntimeBackpressureAction::PurgeStalePeerQueue
        );
        assert_eq!(
            decision.reason_code(),
            "runtime_backpressure_purge_stale_peer_queue"
        );
    }

    #[test]
    fn functional_runtime_queue_enforces_reject_action_on_enqueue() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let mut queue = BoundedRuntimeQueue::new(10).expect("queue should build");
        for index in 0..9 {
            queue
                .enqueue(format!("evt-{index}"))
                .expect("preload should stay in bounds");
        }

        let error = queue
            .enqueue_with_backpressure(
                "evt-reject".to_owned(),
                &controller,
                "kamn:did:agent:peer-bp",
                PeerLifecycleState::Active,
            )
            .expect_err("enqueue should be rejected at saturation threshold");
        assert_eq!(
            error,
            RuntimeQueueError::BackpressureRejected {
                reason_code: "runtime_backpressure_reject_new_enqueue",
                queue_utilization_per_mille: 900,
            }
        );
        assert_eq!(
            error.reason_code(),
            "runtime_backpressure_reject_new_enqueue"
        );
        assert_eq!(queue.len(), 9);
    }

    #[test]
    fn integration_runtime_queue_enforces_stale_peer_purge_action() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let mut queue = BoundedRuntimeQueue::new(8).expect("queue should build");
        queue
            .enqueue("evt-1".to_owned())
            .expect("preload should succeed");
        queue
            .enqueue("evt-2".to_owned())
            .expect("preload should succeed");

        let error = queue
            .enqueue_with_backpressure(
                "evt-disconnected".to_owned(),
                &controller,
                "kamn:did:agent:peer-stale",
                PeerLifecycleState::Disconnected,
            )
            .expect_err("disconnected stale queue should be purged");
        assert_eq!(
            error,
            RuntimeQueueError::BackpressurePurgedStalePeerQueue {
                reason_code: "runtime_backpressure_purge_stale_peer_queue",
                purged_entries: 2,
            }
        );
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn regression_runtime_queue_backpressure_reason_markers_remain_stable() {
        // Regression: #2691
        let decision = RuntimeBackpressureDecision {
            action: RuntimeBackpressureAction::SlowProducer,
            queue_utilization_per_mille: 750,
            stale_peer_queue: false,
        };
        assert_eq!(decision.reason_code(), "runtime_backpressure_slow_producer");

        let queue_error = RuntimeQueueError::BackpressureRejected {
            reason_code: "runtime_backpressure_reject_new_enqueue",
            queue_utilization_per_mille: 950,
        };
        assert_eq!(
            queue_error.reason_code(),
            "runtime_backpressure_reject_new_enqueue"
        );
    }

    #[test]
    fn performance_runtime_queue_backpressure_enforcement_stays_within_ci_budget() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let started = Instant::now();
        for sample_index in 0..2000 {
            let mut queue = BoundedRuntimeQueue::new(16).expect("queue should build");
            let preload = sample_index % 16;
            for event_index in 0..preload {
                queue
                    .enqueue(format!("evt-preload-{event_index}"))
                    .expect("preload should stay bounded");
            }
            let lifecycle_state = if sample_index % 13 == 0 {
                PeerLifecycleState::Disconnected
            } else {
                PeerLifecycleState::Active
            };
            let _ = queue.enqueue_with_backpressure(
                "evt-runtime".to_owned(),
                &controller,
                "kamn:did:agent:peer-perf-runtime",
                lifecycle_state,
            );
        }

        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "runtime queue backpressure enforcement exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    fn regression_runtime_backpressure_rejects_capacity_overflow_sample() {
        // Regression: #618
        assert_eq!(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-a",
                11,
                10,
                PeerLifecycleState::Active
            ),
            Err(RuntimeBackpressureError::QueueDepthExceedsCapacity {
                depth: 11,
                capacity: 10
            })
        );
    }

    #[test]
    fn performance_runtime_backpressure_evaluation_stays_within_ci_budget() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let started = Instant::now();
        for sample_index in 0..2000 {
            let queue_depth = (sample_index % 10) + 1;
            let state = if sample_index % 7 == 0 {
                PeerLifecycleState::Disconnected
            } else {
                PeerLifecycleState::Active
            };
            let input =
                RuntimeBackpressureInput::new("kamn:did:agent:peer-perf", queue_depth, 10, state)
                    .expect("input should be valid");
            let _ = controller
                .evaluate(input)
                .expect("evaluation should remain bounded");
        }
        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 200,
            "runtime backpressure evaluation exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    fn unit_authenticated_peer_frame_rejects_invalid_wire_format() {
        assert_eq!(
            AuthenticatedPeerFrame::from_wire("frame|broken"),
            Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                "frame|broken".to_owned()
            ))
        );
    }

    #[test]
    fn functional_authenticated_peer_frame_roundtrips_wire_and_signature() {
        let frame = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("signed frame should build");
        let wire = frame.to_wire().expect("wire encode should pass");
        let decoded = AuthenticatedPeerFrame::from_wire(&wire).expect("wire decode should pass");
        decoded
            .verify_signature()
            .expect("signature verification should pass");
        assert_eq!(decoded, frame);
    }

    #[test]
    fn integration_peer_frame_authenticator_accepts_monotonic_nonce_flow() {
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let frame_1 = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("frame 1 should build");
        let frame_2 = AuthenticatedPeerFrame::signed(
            "frame-2",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            2,
            "payload-2",
        )
        .expect("frame 2 should build");

        assert!(authenticator.validate_inbound(&frame_1).is_ok());
        assert!(authenticator.validate_inbound(&frame_2).is_ok());
    }

    #[test]
    fn regression_forged_or_unauthorized_peer_frame_is_rejected() {
        // Regression: #618
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let mut forged = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("frame should build");
        forged.signature.push_str("-tampered");
        assert!(matches!(
            authenticator.validate_inbound(&forged),
            Err(AuthenticatedPeerFrameError::SignatureMismatch { .. })
        ));

        let unauthorized = AuthenticatedPeerFrame::signed(
            "frame-2",
            "kamn:did:agent:peer-z",
            "kamn:did:agent:peer-b",
            1,
            "payload-2",
        )
        .expect("frame should build");
        assert_eq!(
            authenticator.validate_inbound(&unauthorized),
            Err(AuthenticatedPeerFrameError::UnauthorizedSender(
                "kamn:did:agent:peer-z".to_owned()
            ))
        );
    }

    #[test]
    fn regression_replayed_peer_frame_nonce_is_rejected() {
        // Regression: #618
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let frame = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("frame should build");
        authenticator
            .validate_inbound(&frame)
            .expect("first frame should be accepted");
        assert_eq!(
            authenticator.validate_inbound(&frame),
            Err(AuthenticatedPeerFrameError::ReplayNonce {
                sender_did: "kamn:did:agent:peer-a".to_owned(),
                last_nonce: 1,
                found: 1
            })
        );
    }

    #[test]
    fn performance_authenticated_peer_frame_validation_stays_within_ci_budget() {
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let started = Instant::now();
        for nonce in 1..=256 {
            let frame = AuthenticatedPeerFrame::signed(
                &format!("frame-{nonce}"),
                "kamn:did:agent:peer-a",
                "kamn:did:agent:peer-b",
                nonce,
                "payload-bounded",
            )
            .expect("frame should build");
            authenticator
                .validate_inbound(&frame)
                .expect("frame should be accepted");
        }
        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "authenticated peer frame validation exceeded CI budget: {elapsed_millis}ms"
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
        assert_eq!(
            report.backpressure_last_action,
            RuntimeBackpressureAction::SlowProducer
        );
        assert_eq!(
            report.backpressure_last_reason_code,
            "runtime_backpressure_slow_producer"
        );
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
            report.backpressure_last_action,
            RuntimeBackpressureAction::RejectNewEnqueue
        );
        assert_eq!(
            report.backpressure_last_reason_code,
            "runtime_backpressure_reject_new_enqueue"
        );
        assert_eq!(report.backpressure_rejected_events, 3);
        assert_eq!(report.backpressure_purged_events, 0);
        assert_eq!(
            report.watchdog_kind,
            WatchdogAnomalyKind::LivenessDegradation
        );
    }

    #[test]
    fn integration_network_fault_simulation_purges_stale_disconnected_peer_queue() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-stale-peer",
            "peer-sim-stale",
            120,
            118,
            6,
            0,
            30,
            1,
            4,
            4,
        )
        .expect("valid simulation input");
        let report = super::simulate_daemon_network_fault(&simulator, input)
            .expect("simulation should pass");

        assert_eq!(
            report.final_lifecycle_state,
            PeerLifecycleState::Disconnected
        );
        assert_eq!(
            report.backpressure_last_action,
            RuntimeBackpressureAction::PurgeStalePeerQueue
        );
        assert_eq!(
            report.backpressure_last_reason_code,
            "runtime_backpressure_purge_stale_peer_queue"
        );
        assert!(report.backpressure_purged_events > 0);
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
