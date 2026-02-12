//! Message delivery replay, nonce, and snapshot guardrail contracts.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Schema version for serialized delivery guard snapshots.
pub const DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Input contract for delivery guard validation.
pub struct DeliveryGuardInput {
    /// Stable message identifier.
    pub message_id: String,
    /// Sender DID.
    pub sender: String,
    /// Recipient DID.
    pub recipient: String,
    /// Monotonic sender nonce.
    pub nonce: u64,
    /// Message creation timestamp.
    pub created: String,
    /// Message expiration timestamp.
    pub expires: String,
    /// Delivery receive timestamp.
    pub received_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Failure taxonomy for delivery validation rejections.
pub enum DeliveryFailureCode {
    /// Nonce does not match the expected sender sequence.
    NonceOutOfSequence {
        /// Expected nonce value.
        expected: u64,
        /// Observed nonce value.
        found: u64,
    },
    /// Message identifier has already been accepted.
    Replay,
    /// Message was received after expiration.
    Expired,
    /// Created/expires window is invalid.
    InvalidWindow,
}

impl DeliveryFailureCode {
    fn slug(&self) -> &'static str {
        match self {
            Self::NonceOutOfSequence { .. } => "nonce_out_of_sequence",
            Self::Replay => "replay",
            Self::Expired => "expired",
            Self::InvalidWindow => "invalid_window",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Signed notice emitted for rejected deliveries.
pub struct FailedDeliveryNotice {
    /// Message identifier.
    pub message_id: String,
    /// Sender DID.
    pub sender: String,
    /// Recipient DID.
    pub recipient: String,
    /// Failure classification.
    pub code: DeliveryFailureCode,
    /// Human-readable rejection details.
    pub detail: String,
    /// Notice timestamp.
    pub timestamp: String,
    /// Deterministic notice signature payload.
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Delivery validation output.
pub enum DeliveryValidationResult {
    /// Delivery was accepted and nonce state updated.
    Accepted,
    /// Delivery was rejected with failure notice.
    Rejected(FailedDeliveryNotice),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory delivery guard engine.
pub struct MessageDeliveryGuards {
    next_nonce_by_sender: BTreeMap<String, u64>,
    seen_message_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot payload for persisting delivery guard state.
pub struct DeliveryGuardSnapshot {
    /// Snapshot schema version.
    pub schema_version: u16,
    /// Expected next nonce per sender.
    pub next_nonce_by_sender: BTreeMap<String, u64>,
    /// Set of already accepted message ids.
    pub seen_message_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error taxonomy for delivery guard snapshot restoration.
pub enum DeliveryGuardSnapshotError {
    /// Snapshot schema version does not match runtime expectation.
    SchemaVersionMismatch {
        /// Expected schema version.
        expected: u16,
        /// Found schema version.
        found: u16,
    },
    /// Sender id in snapshot is invalid.
    InvalidSender(String),
    /// Nonce in snapshot is invalid.
    InvalidNonce {
        /// Sender id associated with the invalid nonce.
        sender: String,
        /// Invalid nonce value.
        nonce: u64,
    },
    /// Message id in snapshot is invalid.
    InvalidMessageId(String),
}

impl MessageDeliveryGuards {
    /// Creates an empty delivery guard state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the expected nonce for `sender`.
    pub fn expected_nonce(&self, sender: &str) -> u64 {
        self.next_nonce_by_sender.get(sender).copied().unwrap_or(1)
    }

    /// Validates a delivery input and updates guard state on success.
    pub fn validate(&mut self, input: DeliveryGuardInput) -> DeliveryValidationResult {
        if input.expires <= input.created {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::InvalidWindow,
                "expires must be strictly greater than created".to_owned(),
            ));
        }
        if input.received_at > input.expires {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::Expired,
                "message expired before delivery".to_owned(),
            ));
        }
        if self.seen_message_ids.contains(&input.message_id) {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::Replay,
                "message replay detected".to_owned(),
            ));
        }

        let expected = self.expected_nonce(&input.sender);
        if input.nonce != expected {
            return DeliveryValidationResult::Rejected(self.failed_notice(
                &input,
                DeliveryFailureCode::NonceOutOfSequence {
                    expected,
                    found: input.nonce,
                },
                format!(
                    "nonce out of sequence for sender {}, expected {}, found {}",
                    input.sender, expected, input.nonce
                ),
            ));
        }

        self.seen_message_ids.insert(input.message_id.clone());
        self.next_nonce_by_sender
            .insert(input.sender.clone(), input.nonce + 1);
        DeliveryValidationResult::Accepted
    }

    /// Exports current state as a snapshot payload.
    pub fn export_snapshot(&self) -> DeliveryGuardSnapshot {
        DeliveryGuardSnapshot {
            schema_version: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
            next_nonce_by_sender: self.next_nonce_by_sender.clone(),
            seen_message_ids: self.seen_message_ids.clone(),
        }
    }

    /// Restores state from a validated snapshot payload.
    pub fn restore_snapshot(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DeliveryGuardSnapshotError> {
        if snapshot.schema_version != DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION {
            return Err(DeliveryGuardSnapshotError::SchemaVersionMismatch {
                expected: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        for (sender, nonce) in &snapshot.next_nonce_by_sender {
            if sender.trim().is_empty() {
                return Err(DeliveryGuardSnapshotError::InvalidSender(sender.clone()));
            }
            if *nonce == 0 {
                return Err(DeliveryGuardSnapshotError::InvalidNonce {
                    sender: sender.clone(),
                    nonce: *nonce,
                });
            }
        }

        for message_id in &snapshot.seen_message_ids {
            if message_id.trim().is_empty() {
                return Err(DeliveryGuardSnapshotError::InvalidMessageId(
                    message_id.clone(),
                ));
            }
        }

        self.next_nonce_by_sender = snapshot.next_nonce_by_sender;
        self.seen_message_ids = snapshot.seen_message_ids;
        Ok(())
    }

    /// Constructs a guard engine from a snapshot payload.
    pub fn from_snapshot(
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<Self, DeliveryGuardSnapshotError> {
        let mut guards = Self::new();
        guards.restore_snapshot(snapshot)?;
        Ok(guards)
    }

    fn failed_notice(
        &self,
        input: &DeliveryGuardInput,
        code: DeliveryFailureCode,
        detail: String,
    ) -> FailedDeliveryNotice {
        let code_slug = code.slug();
        FailedDeliveryNotice {
            message_id: input.message_id.clone(),
            sender: input.sender.clone(),
            recipient: input.recipient.clone(),
            code,
            detail,
            timestamp: input.received_at.clone(),
            signature: format!(
                "notice:{}:{}:{}:{}:{}",
                input.message_id, code_slug, input.recipient, input.received_at, input.nonce
            ),
        }
    }
}

impl fmt::Display for DeliveryFailureCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonceOutOfSequence { expected, found } => {
                write!(
                    f,
                    "nonce out of sequence (expected {expected}, found {found})"
                )
            }
            Self::Replay => write!(f, "message replay detected"),
            Self::Expired => write!(f, "message expired"),
            Self::InvalidWindow => write!(f, "invalid created/expires window"),
        }
    }
}

impl fmt::Display for DeliveryGuardSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaVersionMismatch { expected, found } => {
                write!(
                    f,
                    "delivery guard snapshot schema version mismatch, expected {expected}, found {found}"
                )
            }
            Self::InvalidSender(sender) => {
                write!(
                    f,
                    "delivery guard snapshot contains empty sender id: {sender}"
                )
            }
            Self::InvalidNonce { sender, nonce } => {
                write!(
                    f,
                    "delivery guard snapshot nonce must be greater than zero for sender {sender}, found {nonce}"
                )
            }
            Self::InvalidMessageId(message_id) => {
                write!(
                    f,
                    "delivery guard snapshot contains invalid message id: {message_id}"
                )
            }
        }
    }
}

impl std::error::Error for DeliveryGuardSnapshotError {}

#[cfg(test)]
mod tests {
    use super::{
        DeliveryFailureCode, DeliveryGuardInput, DeliveryValidationResult, MessageDeliveryGuards,
    };

    fn input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
        DeliveryGuardInput {
            message_id: message_id.to_owned(),
            sender: "kamn:did:agent:sender-1".to_owned(),
            recipient: "kamn:did:agent:recipient-1".to_owned(),
            nonce,
            created: "2026-02-07T20:15:30.123Z".to_owned(),
            expires: "2026-02-07T20:45:30.123Z".to_owned(),
            received_at: received_at.to_owned(),
        }
    }

    #[test]
    fn invalid_window_is_rejected() {
        let mut guards = MessageDeliveryGuards::new();
        let mut candidate = input("urn:uuid:msg-1", 1, "2026-02-07T20:20:30.123Z");
        candidate.expires = candidate.created.clone();

        match guards.validate(candidate) {
            DeliveryValidationResult::Rejected(notice) => {
                assert_eq!(notice.code, DeliveryFailureCode::InvalidWindow);
            }
            DeliveryValidationResult::Accepted => panic!("expected invalid window rejection"),
        }
    }

    #[test]
    fn replay_rejected_after_accept() {
        let mut guards = MessageDeliveryGuards::new();
        assert_eq!(
            guards.validate(input("urn:uuid:msg-2", 1, "2026-02-07T20:20:30.123Z")),
            DeliveryValidationResult::Accepted
        );

        match guards.validate(input("urn:uuid:msg-2", 2, "2026-02-07T20:21:30.123Z")) {
            DeliveryValidationResult::Rejected(notice) => {
                assert_eq!(notice.code, DeliveryFailureCode::Replay);
            }
            DeliveryValidationResult::Accepted => panic!("expected replay rejection"),
        }
    }

    #[test]
    fn snapshot_roundtrip_restores_nonce_and_replay_state() {
        let mut guards = MessageDeliveryGuards::new();
        assert_eq!(
            guards.validate(input("urn:uuid:msg-3", 1, "2026-02-07T20:20:30.123Z")),
            DeliveryValidationResult::Accepted
        );

        let snapshot = guards.export_snapshot();
        let restored = MessageDeliveryGuards::from_snapshot(snapshot).expect("snapshot restore");

        assert_eq!(restored.expected_nonce("kamn:did:agent:sender-1"), 2);
    }

    #[test]
    fn snapshot_rejects_zero_nonce_state() {
        let guards = MessageDeliveryGuards::new();
        let mut snapshot = guards.export_snapshot();
        snapshot
            .next_nonce_by_sender
            .insert("kamn:did:agent:sender-1".to_owned(), 0);

        // Regression: #679
        assert!(MessageDeliveryGuards::from_snapshot(snapshot).is_err());
    }
}
