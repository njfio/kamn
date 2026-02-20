use super::*;

/// Projects deterministic archival failure recovery decision under bounded retry policy.
pub fn data_layer_m10_project_archival_retry_decision(
    now_unix_seconds: u64,
    current_attempt: u8,
    failure_class: DataLayerM10ArchivalFailureClass,
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<DataLayerM10ArchivalRetryDecision, DataLayerM10PartitionLifecycleError> {
    validate_archival_retry_policy(policy)?;
    if current_attempt == 0 {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryAttempt {
            field: "current_attempt",
            value: current_attempt,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
        });
    }

    match failure_class {
        DataLayerM10ArchivalFailureClass::Transient if current_attempt < policy.max_attempts => {
            let exponent = u32::from(current_attempt.saturating_sub(1)).min(20);
            let multiplier = 1_u64 << exponent;
            let retry_backoff_seconds = policy
                .base_backoff_seconds
                .saturating_mul(multiplier)
                .min(policy.max_backoff_seconds);
            let retry_after_unix_seconds = now_unix_seconds.saturating_add(retry_backoff_seconds);
            let attempts_remaining = policy.max_attempts.saturating_sub(current_attempt);
            Ok(DataLayerM10ArchivalRetryDecision {
                failure_class,
                action: DataLayerM10ArchivalRecoveryAction::RetryScheduled,
                current_attempt,
                next_attempt: Some(current_attempt.saturating_add(1)),
                retry_backoff_seconds: Some(retry_backoff_seconds),
                retry_after_unix_seconds: Some(retry_after_unix_seconds),
                attempts_remaining,
                reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
            })
        }
        DataLayerM10ArchivalFailureClass::Transient => Ok(DataLayerM10ArchivalRetryDecision {
            failure_class,
            action: DataLayerM10ArchivalRecoveryAction::FailClosed,
            current_attempt,
            next_attempt: None,
            retry_backoff_seconds: None,
            retry_after_unix_seconds: None,
            attempts_remaining: 0,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
        }),
        DataLayerM10ArchivalFailureClass::Permanent => Ok(DataLayerM10ArchivalRetryDecision {
            failure_class,
            action: DataLayerM10ArchivalRecoveryAction::FailClosed,
            current_attempt,
            next_attempt: None,
            retry_backoff_seconds: None,
            retry_after_unix_seconds: None,
            attempts_remaining: 0,
            reason_code: DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
        }),
    }
}

fn validate_archival_retry_policy(
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if policy.max_attempts == 0 {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "max_attempts",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    if policy.base_backoff_seconds == 0 {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "base_backoff_seconds",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    if policy.max_backoff_seconds == 0 || policy.max_backoff_seconds < policy.base_backoff_seconds {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "max_backoff_seconds",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    Ok(())
}
