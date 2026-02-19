use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

const RUNTIME_LISTENER_QUORUM_INVALID_LISTENER_DID_REASON_CODE: &str =
    "runtime_listener_quorum_invalid_listener_did";
const RUNTIME_APPROVER_QUORUM_INVALID_APPROVER_DID_REASON_CODE: &str =
    "runtime_approver_quorum_invalid_approver_did";

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
        parse_listener_did(
            listener_did,
            "listener_did",
            RUNTIME_LISTENER_QUORUM_INVALID_LISTENER_DID_REASON_CODE,
        )?;
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
    InvalidListenerDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
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
            Self::InvalidListenerDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
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
            parse_listener_did(
                attestation.listener_did(),
                "attestations[].listener_did",
                RUNTIME_LISTENER_QUORUM_INVALID_LISTENER_DID_REASON_CODE,
            )?;
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
        parse_approver_did(
            approver_did,
            "approver_did",
            RUNTIME_APPROVER_QUORUM_INVALID_APPROVER_DID_REASON_CODE,
        )?;
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
    InvalidApproverDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
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
            Self::InvalidApproverDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
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
            parse_approver_did(
                attestation.approver_did(),
                "attestations[].approver_did",
                RUNTIME_APPROVER_QUORUM_INVALID_APPROVER_DID_REASON_CODE,
            )?;
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

fn parse_listener_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, ListenerQuorumError> {
    AgentDid::parse(value).map_err(|error| ListenerQuorumError::InvalidListenerDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

fn parse_approver_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, ApproverQuorumError> {
    AgentDid::parse(value).map_err(|error| ApproverQuorumError::InvalidApproverDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

/// Handles authorize daemon outbound action.
pub fn authorize_daemon_outbound_action(
    evaluator: &ApproverQuorumEvaluator,
    input: ApproverQuorumInput,
) -> Result<ApproverQuorumDecision, ApproverQuorumError> {
    evaluator.authorize(input)
}
