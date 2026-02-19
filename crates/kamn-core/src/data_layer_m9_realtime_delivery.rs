//! M9 realtime delivery contracts for presence and deterministic backpressure.
//!
//! This module models PRD M9 behavior as deterministic Rust contracts:
//! owner-scoped dispatch acknowledgements, scoped presence visibility, and
//! queue-cap backpressure escalation markers.

use crate::{
    AgentDid, AgentDidError, AntiSpamDecision, AntiSpamEngine, AntiSpamRejection, ChannelStore,
    DeterministicBackpressureController, KamnDid, KamnDidError, PeerLifecycleState,
    RuntimeBackpressureDecision, RuntimeBackpressureError, RuntimeBackpressureInput,
    RuntimeBackpressurePolicy,
};
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
/// Stable reason marker for channel-membership authorization denials.
pub const DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE: &str =
    "m9_realtime_channel_membership_denied";
/// Stable reason marker for channel policy query failures.
pub const DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE: &str =
    "m9_realtime_channel_policy_query_failed";
/// Stable reason marker for anti-spam insufficient deposit denials.
pub const DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE: &str =
    "m9_realtime_anti_spam_insufficient_deposit";
/// Stable reason marker for anti-spam rate-limit denials.
pub const DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE: &str =
    "m9_realtime_anti_spam_rate_limited";
/// Stable reason marker for anti-spam suspension denials.
pub const DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE: &str =
    "m9_realtime_anti_spam_sender_suspended";
/// Stable reason marker for anti-spam duplicate-message denials.
pub const DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE: &str =
    "m9_realtime_anti_spam_duplicate_message_id";
/// Stable reason marker for runtime backpressure policy projection failures.
pub const DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE: &str =
    "m9_realtime_runtime_backpressure_policy_invalid";
/// Stable reason marker for runtime backpressure input projection failures.
pub const DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE: &str =
    "m9_realtime_runtime_backpressure_input_invalid";
/// Stable reason marker for runtime backpressure evaluation failures.
pub const DATA_LAYER_M9_RUNTIME_BACKPRESSURE_EVALUATION_FAILED_REASON_CODE: &str =
    "m9_realtime_runtime_backpressure_evaluation_failed";
/// Stable reason marker for invalid requester-owner DID inputs.
pub const DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE: &str =
    "m9_realtime_invalid_requester_owner_did";
/// Stable reason marker for invalid target-owner DID inputs.
pub const DATA_LAYER_M9_INVALID_OWNER_DID_REASON_CODE: &str = "m9_realtime_invalid_owner_did";
/// Stable reason marker for invalid connected-agent DID inputs.
pub const DATA_LAYER_M9_INVALID_AGENT_DID_REASON_CODE: &str = "m9_realtime_invalid_agent_did";
/// Stable reason marker for invalid requester-agent DID inputs.
pub const DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE: &str =
    "m9_realtime_invalid_requester_agent_did";
/// Stable reason marker for invalid target-agent DID inputs.
pub const DATA_LAYER_M9_INVALID_TARGET_AGENT_DID_REASON_CODE: &str =
    "m9_realtime_invalid_target_agent_did";
/// Stable reason marker for invalid counterparty-agent DID inputs.
pub const DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE: &str =
    "m9_realtime_invalid_counterparty_agent_did";
/// Stable reason marker for invalid sender-agent DID inputs.
pub const DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE: &str =
    "m9_realtime_invalid_sender_agent_did";
/// Stable reason marker for invalid recipient-agent DID inputs.
pub const DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE: &str =
    "m9_realtime_invalid_recipient_agent_did";

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
        let parsed_agent_did = parse_agent_did(
            request.agent_did.as_str(),
            "agent_did",
            DATA_LAYER_M9_INVALID_AGENT_DID_REASON_CODE,
        )?;
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
            agent_did: parsed_agent_did.as_str().to_owned(),
            connected_since_epoch_seconds: request.connected_since_epoch_seconds,
            last_heartbeat_epoch_seconds: request.last_heartbeat_epoch_seconds,
            gateway_node: request.gateway_node,
            capabilities_active,
        };
        self.presence_by_agent
            .insert(parsed_agent_did.as_str().to_owned(), record.clone());
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
        let requester_agent_did = parse_agent_did(
            request.requester_agent_did.as_str(),
            "requester_agent_did",
            DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
        )?;
        let counterparty_agent_did = parse_agent_did(
            request.counterparty_agent_did.as_str(),
            "counterparty_agent_did",
            DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE,
        )?;
        if requester_agent_did.as_str() == counterparty_agent_did.as_str() {
            return Err(DataLayerM9RealtimeDeliveryError::SameAgentRelationship);
        }
        self.interaction_pairs.insert(normalize_pair(
            requester_agent_did.as_str(),
            counterparty_agent_did.as_str(),
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
        let requester_agent_did = parse_agent_did(
            request.requester_agent_did.as_str(),
            "requester_agent_did",
            DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
        )?;
        let counterparty_agent_did = parse_agent_did(
            request.counterparty_agent_did.as_str(),
            "counterparty_agent_did",
            DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE,
        )?;
        if requester_agent_did.as_str() == counterparty_agent_did.as_str() {
            return Err(DataLayerM9RealtimeDeliveryError::SameAgentRelationship);
        }
        self.shared_escrow_pairs.insert(normalize_pair(
            requester_agent_did.as_str(),
            counterparty_agent_did.as_str(),
        ));
        Ok(())
    }

    /// Queries target presence with scoped visibility controls.
    pub fn query_presence(
        &self,
        query: DataLayerM9PresenceQuery,
    ) -> Result<Option<DataLayerM9PresenceRecord>, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(query.requester_owner_did.as_str(), query.owner_did.as_str())?;
        let requester_agent_did = parse_agent_did(
            query.requester_agent_did.as_str(),
            "requester_agent_did",
            DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
        )?;
        let target_agent_did = parse_agent_did(
            query.target_agent_did.as_str(),
            "target_agent_did",
            DATA_LAYER_M9_INVALID_TARGET_AGENT_DID_REASON_CODE,
        )?;

        let has_visibility = if requester_agent_did.as_str() == target_agent_did.as_str() {
            true
        } else {
            let pair = normalize_pair(requester_agent_did.as_str(), target_agent_did.as_str());
            self.interaction_pairs.contains(&pair) || self.shared_escrow_pairs.contains(&pair)
        };

        if !has_visibility {
            return Err(DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied {
                reason_code: DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
            });
        }

        Ok(self
            .presence_by_agent
            .get(target_agent_did.as_str())
            .cloned())
    }

    /// Snapshots one recipient queue preserving insertion ordering for pending/deferred IDs.
    pub fn snapshot_recipient_queue(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
        recipient_agent_did: &str,
    ) -> Result<DataLayerM9RecipientQueueSnapshot, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(requester_owner_did, owner_did)?;
        let recipient_agent_did = parse_agent_did(
            recipient_agent_did,
            "recipient_agent_did",
            DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        )?;

        let queue_state = self.queue_by_recipient.get(recipient_agent_did.as_str());
        let pending_message_ids = queue_state
            .map(|state| state.pending_message_ids.clone())
            .unwrap_or_default();
        let deferred_message_ids = queue_state
            .map(|state| state.deferred_message_ids.clone())
            .unwrap_or_default();
        let first_full_at_epoch_seconds =
            queue_state.and_then(|state| state.first_full_at_epoch_seconds);

        Ok(DataLayerM9RecipientQueueSnapshot {
            recipient_agent_did: recipient_agent_did.as_str().to_owned(),
            pending_queue_depth: pending_message_ids.len(),
            deferred_count: deferred_message_ids.len(),
            pending_message_ids,
            deferred_message_ids,
            first_full_at_epoch_seconds,
        })
    }

    /// Projects one recipient queue through runtime backpressure contracts.
    pub fn project_runtime_backpressure_for_recipient(
        &self,
        request: DataLayerM9RuntimeBackpressureProjectionRequest,
    ) -> Result<DataLayerM9RuntimeBackpressureProjection, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        let recipient_agent_did = parse_agent_did(
            request.recipient_agent_did.as_str(),
            "recipient_agent_did",
            DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        )?;

        let queue_state = self.queue_by_recipient.get(recipient_agent_did.as_str());
        let pending_queue_depth = queue_state
            .map(|state| state.pending_message_ids.len())
            .unwrap_or_default();
        let deferred_count = queue_state
            .map(|state| state.deferred_message_ids.len())
            .unwrap_or_default();

        let policy = RuntimeBackpressurePolicy::new(
            request.slow_threshold_per_mille,
            request.reject_threshold_per_mille,
            request.purge_disconnected_with_pending_queue,
        )
        .map_err(map_runtime_backpressure_policy_error_to_m9_projection_error)?;

        let input = RuntimeBackpressureInput::new(
            recipient_agent_did.as_str(),
            pending_queue_depth,
            request.queue_capacity,
            request.lifecycle_state,
        )
        .map_err(map_runtime_backpressure_input_error_to_m9_projection_error)?;

        let runtime_decision = DeterministicBackpressureController::new(policy)
            .evaluate(input)
            .map_err(map_runtime_backpressure_evaluation_error_to_m9_projection_error)?;
        let reason_code = runtime_decision.reason_code();

        Ok(DataLayerM9RuntimeBackpressureProjection {
            recipient_agent_did: recipient_agent_did.as_str().to_owned(),
            pending_queue_depth,
            deferred_count,
            runtime_decision,
            reason_code,
        })
    }

    /// Authorizes channel-scoped dispatch by enforcing sender/recipient membership.
    pub fn authorize_channel_dispatch(
        &self,
        channel_store: &ChannelStore,
        request: DataLayerM9ChannelDispatchAuthorizationRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_non_empty(request.channel_id.as_str(), "channel_id")?;
        let sender_agent_did = parse_agent_did(
            request.sender_agent_did.as_str(),
            "sender_agent_did",
            DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
        )?;
        let recipient_agent_did = parse_agent_did(
            request.recipient_agent_did.as_str(),
            "recipient_agent_did",
            DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        )?;

        let sender_member = channel_store
            .is_member(request.channel_id.as_str(), sender_agent_did.as_str())
            .map_err(
                |error| DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed {
                    reason_code: DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        let recipient_member = channel_store
            .is_member(request.channel_id.as_str(), recipient_agent_did.as_str())
            .map_err(
                |error| DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed {
                    reason_code: DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;

        if !sender_member || !recipient_member {
            return Err(DataLayerM9RealtimeDeliveryError::ChannelMembershipDenied {
                reason_code: DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
            });
        }

        Ok(())
    }

    /// Dispatches one message after channel-membership and anti-spam admission controls.
    pub fn dispatch_message_with_controls(
        &mut self,
        channel_store: &ChannelStore,
        anti_spam: &mut AntiSpamEngine,
        channel_id: &str,
        request: DataLayerM9DispatchRequest,
    ) -> Result<DataLayerM9DispatchOutcome, DataLayerM9RealtimeDeliveryError> {
        self.authorize_channel_dispatch(
            channel_store,
            DataLayerM9ChannelDispatchAuthorizationRequest {
                requester_owner_did: request.requester_owner_did.clone(),
                owner_did: request.owner_did.clone(),
                channel_id: channel_id.to_owned(),
                sender_agent_did: request.sender_agent_did.clone(),
                recipient_agent_did: request.recipient_agent_did.clone(),
            },
        )?;

        let anti_spam_decision = anti_spam
            .evaluate(
                request.sender_agent_did.as_str(),
                request.message_id.as_str(),
                request.dispatched_at_epoch_seconds,
            )
            .map_err(
                |error| DataLayerM9RealtimeDeliveryError::AntiSpamEngineError {
                    detail: error.to_string(),
                },
            )?;
        match anti_spam_decision {
            AntiSpamDecision::Accepted => self.dispatch_message(request),
            AntiSpamDecision::Rejected(rejection) => {
                Err(DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
                    reason_code: anti_spam_rejection_reason_code(&rejection),
                })
            }
        }
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
        parse_agent_did(
            request.sender_agent_did.as_str(),
            "sender_agent_did",
            DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
        )?;
        let recipient_agent_did = parse_agent_did(
            request.recipient_agent_did.as_str(),
            "recipient_agent_did",
            DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        )?;
        validate_non_empty(request.message_id.as_str(), "message_id")?;
        if request.dispatched_at_epoch_seconds == 0 {
            return Err(DataLayerM9RealtimeDeliveryError::EmptyField(
                "dispatched_at_epoch_seconds",
            ));
        }

        let queue_state = self
            .queue_by_recipient
            .entry(recipient_agent_did.as_str().to_owned())
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
            .contains_key(recipient_agent_did.as_str());
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
    InvalidDid {
        /// Input field carrying DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
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
    /// Channel policy query failed.
    ChannelPolicyCheckFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable string detail from channel policy module.
        detail: String,
    },
    /// Channel membership validation denied sender/recipient scope.
    ChannelMembershipDenied {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Anti-spam admission denied dispatch.
    AntiSpamAdmissionDenied {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Anti-spam engine failed to evaluate input.
    AntiSpamEngineError {
        /// Stable string detail from anti-spam module.
        detail: String,
    },
    /// Runtime backpressure policy projection failed validation.
    RuntimeBackpressurePolicyInvalid {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from runtime policy validation.
        detail: String,
    },
    /// Runtime backpressure input projection failed validation.
    RuntimeBackpressureInputInvalid {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from runtime input validation.
        detail: String,
    },
    /// Runtime backpressure evaluation failed.
    RuntimeBackpressureEvaluationFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from runtime evaluation.
        detail: String,
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
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(
                f,
                "invalid did field {field}: {reason_code} ({detail})"
            ),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::PresenceVisibilityDenied { reason_code } => {
                write!(f, "presence visibility denied: {reason_code}")
            }
            Self::ChannelPolicyCheckFailed {
                reason_code,
                detail,
            } => {
                write!(f, "channel policy check failed: {reason_code} ({detail})")
            }
            Self::ChannelMembershipDenied { reason_code } => {
                write!(f, "channel membership denied: {reason_code}")
            }
            Self::AntiSpamAdmissionDenied { reason_code } => {
                write!(f, "anti-spam admission denied: {reason_code}")
            }
            Self::AntiSpamEngineError { detail } => {
                write!(f, "anti-spam engine evaluation failed: {detail}")
            }
            Self::RuntimeBackpressurePolicyInvalid {
                reason_code,
                detail,
            } => {
                write!(
                    f,
                    "runtime backpressure policy projection failed: {reason_code} ({detail})"
                )
            }
            Self::RuntimeBackpressureInputInvalid {
                reason_code,
                detail,
            } => {
                write!(
                    f,
                    "runtime backpressure input projection failed: {reason_code} ({detail})"
                )
            }
            Self::RuntimeBackpressureEvaluationFailed {
                reason_code,
                detail,
            } => {
                write!(
                    f,
                    "runtime backpressure evaluation failed: {reason_code} ({detail})"
                )
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

fn anti_spam_rejection_reason_code(rejection: &AntiSpamRejection) -> &'static str {
    match rejection {
        AntiSpamRejection::InsufficientDeposit { .. } => {
            DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE
        }
        AntiSpamRejection::RateLimitExceeded { .. } => {
            DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE
        }
        AntiSpamRejection::SenderSuspended { .. } => DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE,
        AntiSpamRejection::DuplicateMessageId(_) => {
            DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE
        }
    }
}

fn map_runtime_backpressure_policy_error_to_m9_projection_error(
    error: RuntimeBackpressureError,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::RuntimeBackpressurePolicyInvalid {
        reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE,
        detail: error.reason_code().to_owned(),
    }
}

fn map_runtime_backpressure_input_error_to_m9_projection_error(
    error: RuntimeBackpressureError,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::RuntimeBackpressureInputInvalid {
        reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE,
        detail: error.reason_code().to_owned(),
    }
}

fn map_runtime_backpressure_evaluation_error_to_m9_projection_error(
    error: RuntimeBackpressureError,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::RuntimeBackpressureEvaluationFailed {
        reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_EVALUATION_FAILED_REASON_CODE,
        detail: error.reason_code().to_owned(),
    }
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

fn map_agent_did_error(
    error: AgentDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

fn map_kamn_did_error(
    error: KamnDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, DataLayerM9RealtimeDeliveryError> {
    AgentDid::parse(value).map_err(|error| map_agent_did_error(error, field, reason_code))
}

fn parse_kamn_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<KamnDid, DataLayerM9RealtimeDeliveryError> {
    KamnDid::parse(value).map_err(|error| map_kamn_did_error(error, field, reason_code))
}

fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    let requester_owner_did = parse_kamn_did(
        requester_owner_did,
        "requester_owner_did",
        DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE,
    )?;
    let owner_did = parse_kamn_did(
        owner_did,
        "owner_did",
        DATA_LAYER_M9_INVALID_OWNER_DID_REASON_CODE,
    )?;
    if requester_owner_did.as_str() != owner_did.as_str() {
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
