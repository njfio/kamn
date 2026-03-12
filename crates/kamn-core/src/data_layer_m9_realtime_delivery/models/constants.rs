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
