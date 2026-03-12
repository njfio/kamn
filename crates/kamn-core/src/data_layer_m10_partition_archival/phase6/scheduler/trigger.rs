use kamn_data_layer::{
    data_layer_m10_evaluate_phase6_scheduler_trigger_policy,
    data_layer_m10_validate_phase6_scheduler_runtime_clock_signal,
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config,
    DataLayerM10Phase6SchedulerSignalPolicy, DataLayerM10Phase6SchedulerTriggerPolicy,
};

use super::super::policy_mapping::{
    map_data_layer_policy_error_to_m10, map_phase6_scheduler_trigger_decision_from_policy,
};
use super::super::super::{
    DataLayerM10PartitionLifecycleError, DataLayerM10Phase6SchedulerPolicy,
    DataLayerM10Phase6SchedulerSignal, DataLayerM10Phase6SchedulerTriggerDecision,
};

pub(crate) fn validate_phase6_scheduler_policy(
    policy: DataLayerM10Phase6SchedulerPolicy,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config(
        map_phase6_scheduler_trigger_policy_from_core(policy),
    )
    .map_err(map_data_layer_policy_error_to_m10)
}

pub(crate) fn validate_phase6_scheduler_runtime_clock(
    now_epoch_seconds: u64,
    last_observed_now_epoch_seconds: Option<u64>,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    data_layer_m10_validate_phase6_scheduler_runtime_clock_signal(
        now_epoch_seconds,
        last_observed_now_epoch_seconds,
    )
    .map_err(map_data_layer_policy_error_to_m10)
}

/// Evaluates whether one Phase-6 scheduler signal should execute or defer a cycle.
pub fn data_layer_m10_evaluate_phase6_scheduler_trigger(
    policy: DataLayerM10Phase6SchedulerPolicy,
    signal: DataLayerM10Phase6SchedulerSignal,
) -> Result<DataLayerM10Phase6SchedulerTriggerDecision, DataLayerM10PartitionLifecycleError> {
    let trigger_policy_decision = data_layer_m10_evaluate_phase6_scheduler_trigger_policy(
        map_phase6_scheduler_trigger_policy_from_core(policy),
        DataLayerM10Phase6SchedulerSignalPolicy {
            due_candidate_count: signal.due_candidate_count,
            last_tick_epoch_seconds: signal.last_tick_epoch_seconds,
            now_epoch_seconds: signal.now_epoch_seconds,
        },
    )
    .map_err(map_data_layer_policy_error_to_m10)?;
    Ok(map_phase6_scheduler_trigger_decision_from_policy(
        trigger_policy_decision,
    ))
}

fn map_phase6_scheduler_trigger_policy_from_core(
    policy: DataLayerM10Phase6SchedulerPolicy,
) -> DataLayerM10Phase6SchedulerTriggerPolicy {
    DataLayerM10Phase6SchedulerTriggerPolicy {
        due_candidate_trigger_threshold: policy.due_candidate_trigger_threshold,
        max_tick_interval_seconds: policy.max_tick_interval_seconds,
    }
}

pub(crate) fn phase6_scheduler_error_reason_code(
    error: &DataLayerM10PartitionLifecycleError,
) -> &'static str {
    structured_scheduler_error_reason_code(error).unwrap_or(
        crate::DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
    )
}

fn structured_scheduler_error_reason_code(
    error: &DataLayerM10PartitionLifecycleError,
) -> Option<&'static str> {
    match simple_scheduler_error_reason_code(error) {
        Some(reason_code) => Some(reason_code),
        None => complex_scheduler_error_reason_code(error),
    }
}

fn simple_scheduler_error_reason_code(
    error: &DataLayerM10PartitionLifecycleError,
) -> Option<&'static str> {
    match basic_scheduler_error_reason_code(error) {
        Some(reason_code) => Some(reason_code),
        None => budget_scheduler_error_reason_code(error),
    }
}

fn basic_scheduler_error_reason_code(
    error: &DataLayerM10PartitionLifecycleError,
) -> Option<&'static str> {
    match error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { reason_code }
        | DataLayerM10PartitionLifecycleError::InvalidRetryPolicy { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::InvalidRetryAttempt { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerPolicy {
            reason_code, ..
        }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
            reason_code, ..
        } => Some(reason_code),
        _ => None,
    }
}

fn budget_scheduler_error_reason_code(
    error: &DataLayerM10PartitionLifecycleError,
) -> Option<&'static str> {
    match error {
        DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
            reason_code, ..
        }
        | DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded {
            reason_code,
            ..
        }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
            reason_code,
            ..
        } => Some(reason_code),
        _ => None,
    }
}

fn complex_scheduler_error_reason_code(
    error: &DataLayerM10PartitionLifecycleError,
) -> Option<&'static str> {
    match error {
        DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::InvalidLifecycleTransition { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed { reason_code, .. } => {
            Some(reason_code)
        }
        DataLayerM10PartitionLifecycleError::EmptyField(_)
        | DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(_)
        | DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(_)
        | DataLayerM10PartitionLifecycleError::PartitionNotFound(_)
        | _ => None,
    }
}
