//! M9 realtime delivery contracts for presence and deterministic backpressure.
//!
//! This module models PRD M9 behavior as deterministic Rust contracts:
//! owner-scoped dispatch acknowledgements, scoped presence visibility, and
//! queue-cap backpressure escalation markers.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Maximum pending messages per recipient queue before full backpressure mode.
pub const DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES: usize = 1_000;
/// Full-queue duration threshold (seconds) after which warning events are emitted.
pub const DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS: u64 = 300;
/// Full-queue duration threshold (seconds) after which escrow timeout extension is recommended.
pub const DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS: u64 = 3_600;
/// Stable reason marker for delivered acknowledgements.
pub const DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE: &str = "m9_realtime_ack_delivered";
/// Stable reason marker for queued acknowledgements (queue not yet full).
pub const DATA_LAYER_M9_ACK_QUEUED_REASON_CODE: &str = "m9_realtime_ack_queued";
/// Stable reason marker for queued acknowledgements when queue is full.
pub const DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE: &str =
    "m9_realtime_ack_queued_queue_full";
/// Stable reason marker for owner-scope authorization denials.
pub const DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE: &str = "m9_realtime_owner_scope_denied";
/// Stable reason marker for scoped-presence visibility denials.
pub const DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE: &str =
    "m9_realtime_presence_visibility_denied";

/// Dispatch acknowledgement status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM9DispatchAckStatus {
    /// Recipient was connected with no backlog, so immediate delivery ACK is emitted.
    Delivered,
    /// Delivery was deferred and sender receives queued ACK.
    Queued,
}

/// Dispatch request envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9DispatchRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Sender agent DID.
    pub sender_agent_did: String,
    /// Recipient agent DID.
    pub recipient_agent_did: String,
    /// Stable message identifier.
    pub message_id: String,
    /// Dispatch timestamp in epoch seconds.
    pub dispatched_at_epoch_seconds: u64,
}

/// Dispatch outcome projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9DispatchOutcome {
    /// Message identifier.
    pub message_id: String,
    /// Delivery acknowledgement status.
    pub ack_status: DataLayerM9DispatchAckStatus,
    /// Pending queue depth for the recipient after processing this dispatch.
    pub pending_queue_depth: usize,
    /// Deferred message count beyond full queue cap.
    pub deferred_count: usize,
    /// Stable decision reason marker.
    pub reason_code: &'static str,
    /// True when persistent backpressure warning threshold is crossed.
    pub backpressure_warning_event: bool,
    /// True when sustained backpressure threshold is crossed and escrow extension is recommended.
    pub escrow_timeout_extension_recommended: bool,
}

/// Presence-connect request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9PresenceConnectRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Connected agent DID.
    pub agent_did: String,
    /// Connection start timestamp.
    pub connected_since_epoch_seconds: u64,
    /// Last heartbeat timestamp.
    pub last_heartbeat_epoch_seconds: u64,
    /// Gateway node identifier holding the active connection.
    pub gateway_node: String,
    /// Currently active capabilities.
    pub capabilities_active: Vec<String>,
}

/// Presence query envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9PresenceQuery {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Requesting agent DID.
    pub requester_agent_did: String,
    /// Target agent DID.
    pub target_agent_did: String,
}

/// Relationship-link request used for scoped presence visibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9PresenceRelationshipRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Requesting agent DID.
    pub requester_agent_did: String,
    /// Counterparty agent DID.
    pub counterparty_agent_did: String,
}

/// Presence projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9PresenceRecord {
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID.
    pub agent_did: String,
    /// Connection start timestamp.
    pub connected_since_epoch_seconds: u64,
    /// Last heartbeat timestamp.
    pub last_heartbeat_epoch_seconds: u64,
    /// Gateway node identifier.
    pub gateway_node: String,
    /// Sorted active capabilities.
    pub capabilities_active: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DataLayerM9RecipientQueueState {
    pending_message_ids: Vec<String>,
    deferred_message_ids: Vec<String>,
    first_full_at_epoch_seconds: Option<u64>,
}

/// M9 realtime delivery registry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM9RealtimeDeliveryRegistry {
    presence_by_agent: BTreeMap<String, DataLayerM9PresenceRecord>,
    queue_by_recipient: BTreeMap<String, DataLayerM9RecipientQueueState>,
    interaction_pairs: BTreeSet<(String, String)>,
    shared_escrow_pairs: BTreeSet<(String, String)>,
}

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Creates an empty realtime delivery registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or refreshes active presence for one agent.
    pub fn connect_presence(
        &mut self,
        request: DataLayerM9PresenceConnectRequest,
    ) -> Result<DataLayerM9PresenceRecord, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_kamn_did(request.agent_did.as_str())?;
        validate_non_empty(request.gateway_node.as_str(), "gateway_node")?;
        if request.connected_since_epoch_seconds == 0 {
            return Err(DataLayerM9RealtimeDeliveryError::EmptyField(
                "connected_since_epoch_seconds",
            ));
        }
        if request.last_heartbeat_epoch_seconds < request.connected_since_epoch_seconds {
            return Err(DataLayerM9RealtimeDeliveryError::InvalidTimestampOrder {
                connected_since_epoch_seconds: request.connected_since_epoch_seconds,
                last_heartbeat_epoch_seconds: request.last_heartbeat_epoch_seconds,
            });
        }

        let mut capabilities_active = request.capabilities_active;
        capabilities_active.sort();
        capabilities_active.dedup();
        if capabilities_active
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(DataLayerM9RealtimeDeliveryError::EmptyField(
                "capabilities_active",
            ));
        }

        let record = DataLayerM9PresenceRecord {
            owner_did: request.owner_did,
            agent_did: request.agent_did.clone(),
            connected_since_epoch_seconds: request.connected_since_epoch_seconds,
            last_heartbeat_epoch_seconds: request.last_heartbeat_epoch_seconds,
            gateway_node: request.gateway_node,
            capabilities_active,
        };
        self.presence_by_agent
            .insert(request.agent_did, record.clone());
        Ok(record)
    }

    /// Registers prior-interaction linkage for scoped presence visibility.
    pub fn record_interaction_link(
        &mut self,
        request: DataLayerM9PresenceRelationshipRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_kamn_did(request.requester_agent_did.as_str())?;
        validate_kamn_did(request.counterparty_agent_did.as_str())?;
        if request.requester_agent_did == request.counterparty_agent_did {
            return Err(DataLayerM9RealtimeDeliveryError::SameAgentRelationship);
        }
        self.interaction_pairs.insert(normalize_pair(
            request.requester_agent_did.as_str(),
            request.counterparty_agent_did.as_str(),
        ));
        Ok(())
    }

    /// Registers shared-escrow linkage for scoped presence visibility.
    pub fn record_shared_escrow_link(
        &mut self,
        request: DataLayerM9PresenceRelationshipRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_kamn_did(request.requester_agent_did.as_str())?;
        validate_kamn_did(request.counterparty_agent_did.as_str())?;
        if request.requester_agent_did == request.counterparty_agent_did {
            return Err(DataLayerM9RealtimeDeliveryError::SameAgentRelationship);
        }
        self.shared_escrow_pairs.insert(normalize_pair(
            request.requester_agent_did.as_str(),
            request.counterparty_agent_did.as_str(),
        ));
        Ok(())
    }

    /// Queries target presence with scoped visibility controls.
    pub fn query_presence(
        &self,
        query: DataLayerM9PresenceQuery,
    ) -> Result<Option<DataLayerM9PresenceRecord>, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(query.requester_owner_did.as_str(), query.owner_did.as_str())?;
        validate_kamn_did(query.requester_agent_did.as_str())?;
        validate_kamn_did(query.target_agent_did.as_str())?;

        let has_visibility = if query.requester_agent_did == query.target_agent_did {
            true
        } else {
            let pair = normalize_pair(
                query.requester_agent_did.as_str(),
                query.target_agent_did.as_str(),
            );
            self.interaction_pairs.contains(&pair) || self.shared_escrow_pairs.contains(&pair)
        };

        if !has_visibility {
            return Err(DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied {
                reason_code: DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
            });
        }

        Ok(self.presence_by_agent.get(&query.target_agent_did).cloned())
    }

    /// Dispatches one message and computes deterministic ACK outcome.
    pub fn dispatch_message(
        &mut self,
        request: DataLayerM9DispatchRequest,
    ) -> Result<DataLayerM9DispatchOutcome, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_kamn_did(request.sender_agent_did.as_str())?;
        validate_kamn_did(request.recipient_agent_did.as_str())?;
        validate_non_empty(request.message_id.as_str(), "message_id")?;
        if request.dispatched_at_epoch_seconds == 0 {
            return Err(DataLayerM9RealtimeDeliveryError::EmptyField(
                "dispatched_at_epoch_seconds",
            ));
        }

        let queue_state = self
            .queue_by_recipient
            .entry(request.recipient_agent_did.clone())
            .or_default();
        if queue_state
            .pending_message_ids
            .contains(&request.message_id)
            || queue_state
                .deferred_message_ids
                .contains(&request.message_id)
        {
            return Err(DataLayerM9RealtimeDeliveryError::DuplicateMessageId(
                request.message_id,
            ));
        }

        let recipient_connected = self
            .presence_by_agent
            .contains_key(&request.recipient_agent_did);
        if recipient_connected && queue_state.pending_message_ids.is_empty() {
            return Ok(DataLayerM9DispatchOutcome {
                message_id: request.message_id,
                ack_status: DataLayerM9DispatchAckStatus::Delivered,
                pending_queue_depth: 0,
                deferred_count: queue_state.deferred_message_ids.len(),
                reason_code: DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE,
                backpressure_warning_event: false,
                escrow_timeout_extension_recommended: false,
            });
        }

        if queue_state.pending_message_ids.len() < DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
            queue_state
                .pending_message_ids
                .push(request.message_id.clone());
            if queue_state.pending_message_ids.len() == DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES
                && queue_state.first_full_at_epoch_seconds.is_none()
            {
                queue_state.first_full_at_epoch_seconds = Some(request.dispatched_at_epoch_seconds);
            }
            let (warning, extension) = queue_escalation(
                queue_state.first_full_at_epoch_seconds,
                request.dispatched_at_epoch_seconds,
            );
            return Ok(DataLayerM9DispatchOutcome {
                message_id: request.message_id,
                ack_status: DataLayerM9DispatchAckStatus::Queued,
                pending_queue_depth: queue_state.pending_message_ids.len(),
                deferred_count: queue_state.deferred_message_ids.len(),
                reason_code: DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
                backpressure_warning_event: warning,
                escrow_timeout_extension_recommended: extension,
            });
        }

        if queue_state.first_full_at_epoch_seconds.is_none() {
            queue_state.first_full_at_epoch_seconds = Some(request.dispatched_at_epoch_seconds);
        }
        queue_state
            .deferred_message_ids
            .push(request.message_id.clone());
        let (warning, extension) = queue_escalation(
            queue_state.first_full_at_epoch_seconds,
            request.dispatched_at_epoch_seconds,
        );
        Ok(DataLayerM9DispatchOutcome {
            message_id: request.message_id,
            ack_status: DataLayerM9DispatchAckStatus::Queued,
            pending_queue_depth: DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES,
            deferred_count: queue_state.deferred_message_ids.len(),
            reason_code: DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE,
            backpressure_warning_event: warning,
            escrow_timeout_extension_recommended: extension,
        })
    }
}

/// Error taxonomy for M9 realtime delivery contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM9RealtimeDeliveryError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Owner-scope authorization failed.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Presence visibility policy denied the query.
    PresenceVisibilityDenied {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Connected-since and heartbeat timestamps were ordered incorrectly.
    InvalidTimestampOrder {
        /// Connection start timestamp.
        connected_since_epoch_seconds: u64,
        /// Last heartbeat timestamp.
        last_heartbeat_epoch_seconds: u64,
    },
    /// Request attempted to link one agent to itself.
    SameAgentRelationship,
    /// Duplicate message id in recipient queue/deferred state.
    DuplicateMessageId(String),
}

impl fmt::Display for DataLayerM9RealtimeDeliveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::PresenceVisibilityDenied { reason_code } => {
                write!(f, "presence visibility denied: {reason_code}")
            }
            Self::InvalidTimestampOrder {
                connected_since_epoch_seconds,
                last_heartbeat_epoch_seconds,
            } => write!(
                f,
                "invalid timestamp order: connected_since={connected_since_epoch_seconds}, last_heartbeat={last_heartbeat_epoch_seconds}"
            ),
            Self::SameAgentRelationship => {
                write!(f, "relationship requester and counterparty must differ")
            }
            Self::DuplicateMessageId(value) => write!(f, "duplicate message id: {value}"),
        }
    }
}

impl std::error::Error for DataLayerM9RealtimeDeliveryError {}

fn queue_escalation(first_full_at: Option<u64>, now_epoch_seconds: u64) -> (bool, bool) {
    let Some(first_full_at_epoch_seconds) = first_full_at else {
        return (false, false);
    };
    let full_duration_seconds = now_epoch_seconds.saturating_sub(first_full_at_epoch_seconds);
    let warning = full_duration_seconds > DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS;
    let extension =
        full_duration_seconds > DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS;
    (warning, extension)
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    if value.trim().is_empty() {
        return Err(DataLayerM9RealtimeDeliveryError::EmptyField(field));
    }
    Ok(())
}

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM9RealtimeDeliveryError::InvalidDid(
            value.to_owned(),
        ));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM9RealtimeDeliveryError::InvalidDid(
            value.to_owned(),
        ));
    }
    Ok(())
}

fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    validate_kamn_did(requester_owner_did)?;
    validate_kamn_did(owner_did)?;
    if requester_owner_did != owner_did {
        return Err(DataLayerM9RealtimeDeliveryError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
        });
    }
    Ok(())
}

fn normalize_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_owned(), right.to_owned())
    } else {
        (right.to_owned(), left.to_owned())
    }
}
