use crate::{PeerLifecycleState, RuntimeBackpressureDecision};

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

/// Runtime backpressure projection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9RuntimeBackpressureProjectionRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Recipient agent DID.
    pub recipient_agent_did: String,
    /// Queue capacity used to evaluate runtime backpressure thresholds.
    pub queue_capacity: usize,
    /// Recipient peer lifecycle state used by runtime backpressure policy.
    pub lifecycle_state: PeerLifecycleState,
    /// Slow-producer threshold per mille.
    pub slow_threshold_per_mille: u16,
    /// Reject-new-enqueue threshold per mille.
    pub reject_threshold_per_mille: u16,
    /// Whether disconnected peers with pending queue entries should be purged.
    pub purge_disconnected_with_pending_queue: bool,
}

/// Channel dispatch authorization request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9ChannelDispatchAuthorizationRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Channel identifier used for membership validation.
    pub channel_id: String,
    /// Sender agent DID.
    pub sender_agent_did: String,
    /// Recipient agent DID.
    pub recipient_agent_did: String,
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

/// Runtime backpressure projection output for one recipient queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9RuntimeBackpressureProjection {
    /// Recipient agent DID.
    pub recipient_agent_did: String,
    /// Pending queue depth used for runtime backpressure input.
    pub pending_queue_depth: usize,
    /// Deferred queue depth retained in M9 for audit/diagnostic context.
    pub deferred_count: usize,
    /// Runtime backpressure decision.
    pub runtime_decision: RuntimeBackpressureDecision,
    /// Stable reason code from runtime backpressure decision.
    pub reason_code: &'static str,
}

/// Queue snapshot projection for deterministic ordering validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9RecipientQueueSnapshot {
    /// Recipient agent DID.
    pub recipient_agent_did: String,
    /// Pending queue depth.
    pub pending_queue_depth: usize,
    /// Deferred queue depth.
    pub deferred_count: usize,
    /// Pending message identifiers in insertion order.
    pub pending_message_ids: Vec<String>,
    /// Deferred message identifiers in insertion order.
    pub deferred_message_ids: Vec<String>,
    /// Timestamp when queue first reached full capacity.
    pub first_full_at_epoch_seconds: Option<u64>,
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
