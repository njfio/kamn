use kamn_data_layer::{
    DataLayerM10Phase6BudgetPolicyReport, DataLayerM10Phase6PolicyBudget,
    DataLayerM10Phase6PolicyBudgetDecision, DataLayerM10Phase6PolicyEvaluatorError,
    DataLayerM10Phase6SchedulerBudgetOverflowPolicyProjection, DataLayerM10Phase6TriggerPolicyDecision,
};

use super::super::{
    DataLayerM10PartitionLifecycleError, DataLayerM10Phase6ExecutionBudgetDecision,
    DataLayerM10Phase6ExecutionTickBudget, DataLayerM10Phase6ExecutionTickBudgetReport,
    DataLayerM10Phase6SchedulerTriggerDecision,
};

pub(super) fn map_data_layer_policy_error_to_m10(
    error: DataLayerM10Phase6PolicyEvaluatorError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10Phase6PolicyEvaluatorError::InvalidBudgetField { field, reason_code } => {
            DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget { field, reason_code }
        }
        DataLayerM10Phase6PolicyEvaluatorError::InvalidSchedulerPolicyField {
            field,
            reason_code,
        } => {
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerPolicy { field, reason_code }
        }
        DataLayerM10Phase6PolicyEvaluatorError::InvalidSchedulerSignalField {
            field,
            reason_code,
        } => {
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal { field, reason_code }
        }
    }
}

pub(super) fn map_phase6_policy_budget_from_core(
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> DataLayerM10Phase6PolicyBudget {
    DataLayerM10Phase6PolicyBudget {
        max_due_candidates: budget.max_due_candidates,
        max_shredded_messages: budget.max_shredded_messages,
        max_projection_reports: budget.max_projection_reports,
        max_archived_entries: budget.max_archived_entries,
    }
}

pub(super) fn map_phase6_budget_policy_report_to_core(
    budget_policy_report: DataLayerM10Phase6BudgetPolicyReport,
) -> DataLayerM10Phase6ExecutionTickBudgetReport {
    let decision = match budget_policy_report.decision {
        DataLayerM10Phase6PolicyBudgetDecision::WithinBudget => {
            DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget
        }
        DataLayerM10Phase6PolicyBudgetDecision::Exceeded => {
            DataLayerM10Phase6ExecutionBudgetDecision::Exceeded
        }
    };
    DataLayerM10Phase6ExecutionTickBudgetReport {
        decision,
        reason_code: budget_policy_report.reason_code,
        due_candidate_count: budget_policy_report.due_candidate_count,
        shredded_message_count: budget_policy_report.shredded_message_count,
        projection_report_count: budget_policy_report.projection_report_count,
        archived_entry_count: budget_policy_report.archived_entry_count,
    }
}

pub(super) fn map_phase6_budget_policy_report_from_core(
    budget_report: &DataLayerM10Phase6ExecutionTickBudgetReport,
) -> DataLayerM10Phase6BudgetPolicyReport {
    let decision = match budget_report.decision {
        DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget => {
            DataLayerM10Phase6PolicyBudgetDecision::WithinBudget
        }
        DataLayerM10Phase6ExecutionBudgetDecision::Exceeded => {
            DataLayerM10Phase6PolicyBudgetDecision::Exceeded
        }
    };
    DataLayerM10Phase6BudgetPolicyReport {
        decision,
        reason_code: budget_report.reason_code,
        due_candidate_count: budget_report.due_candidate_count,
        shredded_message_count: budget_report.shredded_message_count,
        projection_report_count: budget_report.projection_report_count,
        archived_entry_count: budget_report.archived_entry_count,
    }
}

pub(super) fn map_phase6_scheduler_trigger_decision_from_policy(
    trigger_policy_decision: DataLayerM10Phase6TriggerPolicyDecision,
) -> DataLayerM10Phase6SchedulerTriggerDecision {
    match trigger_policy_decision {
        DataLayerM10Phase6TriggerPolicyDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        },
        DataLayerM10Phase6TriggerPolicyDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        },
    }
}

pub(super) fn map_phase6_scheduler_trigger_decision_to_policy(
    trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision,
) -> DataLayerM10Phase6TriggerPolicyDecision {
    match trigger_decision {
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => DataLayerM10Phase6TriggerPolicyDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        },
        DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => DataLayerM10Phase6TriggerPolicyDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        },
    }
}

pub(super) fn map_phase6_budget_overflow_projection_to_core(
    projected_overflow: DataLayerM10Phase6SchedulerBudgetOverflowPolicyProjection,
) -> DataLayerM10PartitionLifecycleError {
    DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded {
        reason_code: projected_overflow.reason_code,
        detail: projected_overflow.detail,
    }
}
