//! M1 batch scheduler contracts for deterministic anchoring trigger evaluation.
//!
//! This module intentionally keeps scheduling logic deterministic and side-effect
//! free so runtime workers can apply the same policy decisions consistently.

use std::fmt;

/// Reason marker for deferred trigger decisions.
pub const DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_DEFERRED: &str = "m1_batch_trigger_deferred";
/// Reason marker for count-threshold trigger decisions.
pub const DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_COUNT_THRESHOLD: &str =
    "m1_batch_trigger_count_threshold_met";
/// Reason marker for window-threshold trigger decisions.
pub const DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_WINDOW_THRESHOLD: &str =
    "m1_batch_trigger_window_threshold_met";

/// Deterministic scheduler policy for M1 merkle batch triggering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM1BatchSchedulerPolicy {
    /// Maximum number of pending messages before immediate trigger.
    pub max_messages_per_batch: usize,
    /// Maximum age for oldest pending message before trigger.
    pub max_batch_window_seconds: u64,
}

impl DataLayerM1BatchSchedulerPolicy {
    /// Builds a scheduler policy and fail-closes on invalid threshold values.
    pub fn new(
        max_messages_per_batch: usize,
        max_batch_window_seconds: u64,
    ) -> Result<Self, DataLayerM1BatchSchedulerError> {
        if max_messages_per_batch == 0 {
            return Err(DataLayerM1BatchSchedulerError::InvalidThreshold {
                field: "max_messages_per_batch",
                detail: "must be greater than zero".to_owned(),
            });
        }
        if max_batch_window_seconds == 0 {
            return Err(DataLayerM1BatchSchedulerError::InvalidThreshold {
                field: "max_batch_window_seconds",
                detail: "must be greater than zero".to_owned(),
            });
        }
        Ok(Self {
            max_messages_per_batch,
            max_batch_window_seconds,
        })
    }
}

/// Pending message candidate considered by batch scheduler policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1PendingBatchMessage {
    /// Stable message identifier.
    pub message_id: String,
    /// Content hash referenced by merkle leaf assembly.
    pub content_hash: String,
    /// Pending message creation epoch in unix seconds.
    pub created_at_unix_seconds: u64,
}

/// Deterministic scheduler trigger decision projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1BatchTriggerDecision {
    /// Trigger was deferred; thresholds were not met.
    Deferred {
        /// Stable decision reason marker.
        reason_code: &'static str,
        /// Current pending queue size.
        pending_count: usize,
        /// Oldest pending age in seconds.
        oldest_pending_age_seconds: u64,
    },
    /// Trigger should execute now due to threshold hit.
    Triggered {
        /// Stable decision reason marker.
        reason_code: &'static str,
        /// Current pending queue size.
        pending_count: usize,
        /// Oldest pending age in seconds.
        oldest_pending_age_seconds: u64,
    },
}

/// Error taxonomy for scheduler policy validation and decision evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1BatchSchedulerError {
    /// Threshold configuration was invalid.
    InvalidThreshold {
        /// Field carrying invalid threshold value.
        field: &'static str,
        /// Validation detail.
        detail: String,
    },
    /// Pending candidate payload was invalid.
    InvalidPendingMessage {
        /// Field carrying invalid pending message payload.
        field: &'static str,
        /// Validation detail.
        detail: String,
    },
}

impl fmt::Display for DataLayerM1BatchSchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidThreshold { field, detail } => {
                write!(formatter, "invalid threshold {field}: {detail}")
            }
            Self::InvalidPendingMessage { field, detail } => {
                write!(formatter, "invalid pending message field {field}: {detail}")
            }
        }
    }
}

impl std::error::Error for DataLayerM1BatchSchedulerError {}

/// Evaluates deterministic M1 scheduler trigger decision.
///
/// Decision precedence is fixed:
/// 1. count-threshold
/// 2. window-threshold
/// 3. deferred
pub fn evaluate_data_layer_m1_batch_trigger(
    policy: &DataLayerM1BatchSchedulerPolicy,
    pending_messages: &[DataLayerM1PendingBatchMessage],
    now_unix_seconds: u64,
) -> Result<DataLayerM1BatchTriggerDecision, DataLayerM1BatchSchedulerError> {
    let pending_count = pending_messages.len();
    let oldest_pending_age_seconds =
        resolve_oldest_pending_age_seconds(pending_messages, now_unix_seconds)?;

    if pending_count >= policy.max_messages_per_batch {
        return Ok(DataLayerM1BatchTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_COUNT_THRESHOLD,
            pending_count,
            oldest_pending_age_seconds,
        });
    }

    if pending_count > 0 && oldest_pending_age_seconds >= policy.max_batch_window_seconds {
        return Ok(DataLayerM1BatchTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_WINDOW_THRESHOLD,
            pending_count,
            oldest_pending_age_seconds,
        });
    }

    Ok(DataLayerM1BatchTriggerDecision::Deferred {
        reason_code: DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_DEFERRED,
        pending_count,
        oldest_pending_age_seconds,
    })
}

fn resolve_oldest_pending_age_seconds(
    pending_messages: &[DataLayerM1PendingBatchMessage],
    now_unix_seconds: u64,
) -> Result<u64, DataLayerM1BatchSchedulerError> {
    if pending_messages.is_empty() {
        return Ok(0);
    }

    let mut oldest_created_at = u64::MAX;
    for message in pending_messages {
        if message.message_id.trim().is_empty() {
            return Err(DataLayerM1BatchSchedulerError::InvalidPendingMessage {
                field: "message_id",
                detail: "must not be empty".to_owned(),
            });
        }
        if message.content_hash.trim().is_empty() {
            return Err(DataLayerM1BatchSchedulerError::InvalidPendingMessage {
                field: "content_hash",
                detail: "must not be empty".to_owned(),
            });
        }
        if message.created_at_unix_seconds > now_unix_seconds {
            return Err(DataLayerM1BatchSchedulerError::InvalidPendingMessage {
                field: "created_at_unix_seconds",
                detail: "pending message timestamp is in the future".to_owned(),
            });
        }
        oldest_created_at = oldest_created_at.min(message.created_at_unix_seconds);
    }

    Ok(now_unix_seconds.saturating_sub(oldest_created_at))
}
