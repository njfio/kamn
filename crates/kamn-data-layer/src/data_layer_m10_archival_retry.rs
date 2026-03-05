//! M10 archival retry projection contracts extracted from `kamn-core`.
//!
//! This surface models bounded retry/fail-closed behavior for transient and
//! permanent archival failures.

use std::fmt;

/// Retry classification for archival export failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10ArchivalFailureClass {
    /// Failure may succeed on a later attempt.
    Transient,
    /// Failure must fail closed immediately.
    Permanent,
}

/// Recovery action projected for an archival export failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10ArchivalRecoveryAction {
    /// Schedule one more retry attempt.
    RetryScheduled,
    /// Fail closed and stop retrying.
    FailClosed,
}

/// Bounded retry policy for archival failure recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10ArchivalRetryPolicy {
    /// Total attempts allowed, including the current attempt.
    pub max_attempts: u8,
    /// Base retry backoff in seconds.
    pub base_backoff_seconds: u64,
    /// Maximum retry backoff cap in seconds.
    pub max_backoff_seconds: u64,
}

/// Deterministic decision projected for an archival export failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10ArchivalRetryDecision {
    /// Failure classification used for this projection.
    pub failure_class: DataLayerM10ArchivalFailureClass,
    /// Recovery action.
    pub action: DataLayerM10ArchivalRecoveryAction,
    /// Current failed attempt number.
    pub current_attempt: u8,
    /// Next attempt number when a retry is scheduled.
    pub next_attempt: Option<u8>,
    /// Retry delay in seconds when a retry is scheduled.
    pub retry_backoff_seconds: Option<u64>,
    /// Retry-at timestamp in epoch seconds when a retry is scheduled.
    pub retry_after_unix_seconds: Option<u64>,
    /// Remaining attempts after this decision.
    pub attempts_remaining: u8,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Stable reason marker when archival transient failure schedules a retry.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE: &str =
    "m10_archival_retry_scheduled";
/// Stable reason marker when archival retry budget is exhausted.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE: &str =
    "m10_archival_retry_exhausted";
/// Stable reason marker when archival failure is permanent and must fail closed.
pub const DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE: &str =
    "m10_archival_failure_permanent";
/// Stable reason marker when archival retry policy configuration is invalid.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE: &str =
    "m10_archival_retry_policy_invalid";
/// Stable reason marker when archival retry attempt metadata is invalid.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE: &str =
    "m10_archival_retry_attempt_invalid";

/// Error taxonomy for M10 archival retry projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10ArchivalRetryError {
    /// Archival retry policy configuration is invalid.
    InvalidRetryPolicy {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Current retry attempt metadata is invalid.
    InvalidRetryAttempt {
        /// Invalid field.
        field: &'static str,
        /// Invalid value.
        value: u8,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM10ArchivalRetryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRetryPolicy { field, reason_code } => {
                write!(
                    formatter,
                    "invalid archival retry policy field {field} ({reason_code})"
                )
            }
            Self::InvalidRetryAttempt {
                field,
                value,
                reason_code,
            } => write!(
                formatter,
                "invalid archival retry attempt for {field}: {value} ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerM10ArchivalRetryError {}

/// Projects deterministic archival failure recovery decision under bounded retry policy.
pub fn data_layer_m10_project_archival_retry_decision(
    now_unix_seconds: u64,
    current_attempt: u8,
    failure_class: DataLayerM10ArchivalFailureClass,
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<DataLayerM10ArchivalRetryDecision, DataLayerM10ArchivalRetryError> {
    validate_archival_retry_policy(policy)?;
    if current_attempt == 0 {
        return Err(DataLayerM10ArchivalRetryError::InvalidRetryAttempt {
            field: "current_attempt",
            value: current_attempt,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
        });
    }

    let decision = match failure_class {
        DataLayerM10ArchivalFailureClass::Transient if current_attempt < policy.max_attempts => {
            project_transient_retry(now_unix_seconds, current_attempt, policy)
        }
        DataLayerM10ArchivalFailureClass::Transient => {
            project_fail_closed_decision(
                failure_class,
                current_attempt,
                DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
            )
        }
        DataLayerM10ArchivalFailureClass::Permanent => project_fail_closed_decision(
            failure_class,
            current_attempt,
            DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
        ),
    };
    Ok(decision)
}

fn validate_archival_retry_policy(
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<(), DataLayerM10ArchivalRetryError> {
    if policy.max_attempts == 0 {
        return Err(DataLayerM10ArchivalRetryError::InvalidRetryPolicy {
            field: "max_attempts",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    if policy.base_backoff_seconds == 0 {
        return Err(DataLayerM10ArchivalRetryError::InvalidRetryPolicy {
            field: "base_backoff_seconds",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    if policy.max_backoff_seconds == 0 || policy.max_backoff_seconds < policy.base_backoff_seconds {
        return Err(DataLayerM10ArchivalRetryError::InvalidRetryPolicy {
            field: "max_backoff_seconds",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    Ok(())
}

fn project_transient_retry(
    now_unix_seconds: u64,
    current_attempt: u8,
    policy: DataLayerM10ArchivalRetryPolicy,
) -> DataLayerM10ArchivalRetryDecision {
    let exponent = u32::from(current_attempt.saturating_sub(1)).min(20);
    let multiplier = 1_u64 << exponent;
    let retry_backoff_seconds = policy
        .base_backoff_seconds
        .saturating_mul(multiplier)
        .min(policy.max_backoff_seconds);
    let retry_after_unix_seconds = now_unix_seconds.saturating_add(retry_backoff_seconds);
    let attempts_remaining = policy.max_attempts.saturating_sub(current_attempt);
    DataLayerM10ArchivalRetryDecision {
        failure_class: DataLayerM10ArchivalFailureClass::Transient,
        action: DataLayerM10ArchivalRecoveryAction::RetryScheduled,
        current_attempt,
        next_attempt: Some(current_attempt.saturating_add(1)),
        retry_backoff_seconds: Some(retry_backoff_seconds),
        retry_after_unix_seconds: Some(retry_after_unix_seconds),
        attempts_remaining,
        reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
    }
}

fn project_fail_closed_decision(
    failure_class: DataLayerM10ArchivalFailureClass,
    current_attempt: u8,
    reason_code: &'static str,
) -> DataLayerM10ArchivalRetryDecision {
    DataLayerM10ArchivalRetryDecision {
        failure_class,
        action: DataLayerM10ArchivalRecoveryAction::FailClosed,
        current_attempt,
        next_attempt: None,
        retry_backoff_seconds: None,
        retry_after_unix_seconds: None,
        attempts_remaining: 0,
        reason_code,
    }
}
