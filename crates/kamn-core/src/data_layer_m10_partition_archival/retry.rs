use kamn_data_layer::DataLayerM10ArchivalRetryError;

use super::*;

/// Projects deterministic archival failure recovery decision under bounded retry policy.
pub fn data_layer_m10_project_archival_retry_decision(
    now_unix_seconds: u64,
    current_attempt: u8,
    failure_class: DataLayerM10ArchivalFailureClass,
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<DataLayerM10ArchivalRetryDecision, DataLayerM10PartitionLifecycleError> {
    kamn_data_layer::data_layer_m10_project_archival_retry_decision(
        now_unix_seconds,
        current_attempt,
        failure_class,
        policy,
    )
    .map_err(map_retry_projection_error_to_core)
}

fn map_retry_projection_error_to_core(
    error: DataLayerM10ArchivalRetryError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10ArchivalRetryError::InvalidRetryPolicy { field, reason_code } => {
            DataLayerM10PartitionLifecycleError::InvalidRetryPolicy { field, reason_code }
        }
        DataLayerM10ArchivalRetryError::InvalidRetryAttempt {
            field,
            value,
            reason_code,
        } => DataLayerM10PartitionLifecycleError::InvalidRetryAttempt {
            field,
            value,
            reason_code,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        data_layer_m10_project_archival_retry_decision, DataLayerM10ArchivalFailureClass,
        DataLayerM10ArchivalRecoveryAction, DataLayerM10ArchivalRetryPolicy,
        DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
        DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
    };
    use crate::DataLayerM10PartitionLifecycleError;

    #[test]
    fn unit_retry_projection_rejects_invalid_policy_fields() {
        let invalid_policy = DataLayerM10ArchivalRetryPolicy {
            max_attempts: 0,
            base_backoff_seconds: 30,
            max_backoff_seconds: 300,
        };
        assert_eq!(
            data_layer_m10_project_archival_retry_decision(
                1_735_700_000,
                1,
                DataLayerM10ArchivalFailureClass::Transient,
                invalid_policy
            ),
            Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
                field: "max_attempts",
                reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
            })
        );
    }

    #[test]
    fn unit_retry_projection_schedules_transient_retry_then_exhausts() {
        let policy = DataLayerM10ArchivalRetryPolicy {
            max_attempts: 3,
            base_backoff_seconds: 30,
            max_backoff_seconds: 300,
        };

        let scheduled = data_layer_m10_project_archival_retry_decision(
            1_735_700_000,
            1,
            DataLayerM10ArchivalFailureClass::Transient,
            policy,
        )
        .expect("transient failure under max attempts should schedule retry");
        assert_eq!(
            scheduled.action,
            DataLayerM10ArchivalRecoveryAction::RetryScheduled
        );
        assert_eq!(
            scheduled.reason_code,
            DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE
        );

        let exhausted = data_layer_m10_project_archival_retry_decision(
            1_735_700_000,
            3,
            DataLayerM10ArchivalFailureClass::Transient,
            policy,
        )
        .expect("transient failure at max attempts should fail closed");
        assert_eq!(
            exhausted.action,
            DataLayerM10ArchivalRecoveryAction::FailClosed
        );
        assert_eq!(
            exhausted.reason_code,
            DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE
        );
    }
}
