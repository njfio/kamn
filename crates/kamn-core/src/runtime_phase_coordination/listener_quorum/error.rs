use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum error.
pub enum ListenerQuorumError {
    /// Invalid required confirmations.
    InvalidRequiredConfirmations { required: usize },
    /// Invalid event id.
    InvalidEventId,
    /// Invalid event sequence.
    InvalidEventSequence,
    /// Invalid listener did.
    InvalidListenerDid {
        field: &'static str,
        reason_code: &'static str,
        detail: String,
    },
    /// Invalid attestation id.
    InvalidAttestationId,
    /// Duplicate listener attestation.
    DuplicateListenerAttestation { listener_did: String },
    /// Replayed event sequence.
    ReplayedEventSequence {
        event_id: String,
        previous_sequence: u64,
        received_sequence: u64,
    },
    /// Insufficient confirmations.
    InsufficientConfirmations { required: usize, received: usize },
}

impl Display for ListenerQuorumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequiredConfirmations { required } => write!(f, "invalid listener quorum requirement: {required}"),
            Self::InvalidEventId => write!(f, "listener quorum event id cannot be empty"),
            Self::InvalidEventSequence => write!(f, "listener quorum event sequence must be positive"),
            Self::InvalidListenerDid { field, reason_code, detail } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidAttestationId => write!(f, "listener attestation id cannot be empty"),
            Self::DuplicateListenerAttestation { listener_did } => write!(f, "duplicate listener attestation replay detected for {listener_did}"),
            Self::ReplayedEventSequence { event_id, previous_sequence, received_sequence } => write!(f, "listener event sequence replay detected for {event_id}: previous {previous_sequence}, received {received_sequence}"),
            Self::InsufficientConfirmations { required, received } => write!(f, "listener quorum insufficient confirmations: required {required}, received {received}"),
        }
    }
}

impl Error for ListenerQuorumError {}
