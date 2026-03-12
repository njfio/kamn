use kamn_data_layer::{
    data_layer_m10_evaluate_phase6_execution_tick_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy,
    data_layer_m10_project_phase6_scheduler_budget_overflow_policy_error,
    data_layer_m10_validate_phase6_execution_budget_policy,
    DataLayerM10Phase6SchedulerBudgetOverflowStage,
};

use super::super::policy_mapping::{
    map_data_layer_policy_error_to_m10, map_phase6_budget_overflow_projection_to_core,
    map_phase6_budget_policy_report_from_core, map_phase6_budget_policy_report_to_core,
    map_phase6_policy_budget_from_core,
};
use super::super::super::{
    DataLayerM10PartitionLifecycleError, DataLayerM10Phase6ExecutionTickBudget,
    DataLayerM10Phase6ExecutionTickBudgetReport,
};

pub(crate) fn validate_phase6_execution_tick_budget(
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    data_layer_m10_validate_phase6_execution_budget_policy(map_phase6_policy_budget_from_core(
        budget,
    ))
    .map_err(map_data_layer_policy_error_to_m10)
}

pub fn data_layer_m10_evaluate_phase6_execution_tick_budget(
    report: &crate::DataLayerM10Phase6ExecutionTickReport,
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> Result<DataLayerM10Phase6ExecutionTickBudgetReport, DataLayerM10PartitionLifecycleError> {
    let budget_policy_report = data_layer_m10_evaluate_phase6_execution_tick_budget_policy(
        kamn_data_layer::DataLayerM10Phase6PolicyReportCounts {
            due_candidate_count: report.due_candidate_count,
            shredded_message_count: report.shredded_message_ids.len(),
            projection_report_count: report.projection_reports.len(),
            archived_entry_count: report.archived_entries.len(),
        },
        map_phase6_policy_budget_from_core(budget),
    )
    .map_err(map_data_layer_policy_error_to_m10)?;
    Ok(map_phase6_budget_policy_report_to_core(
        budget_policy_report,
    ))
}

pub(crate) fn evaluate_phase6_scheduler_preflight_budget(
    due_candidate_count: usize,
    projection_report_count: usize,
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> Result<DataLayerM10Phase6ExecutionTickBudgetReport, DataLayerM10PartitionLifecycleError> {
    let budget_policy_report = data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy(
        due_candidate_count,
        projection_report_count,
        map_phase6_policy_budget_from_core(budget),
    )
    .map_err(map_data_layer_policy_error_to_m10)?;
    Ok(map_phase6_budget_policy_report_to_core(
        budget_policy_report,
    ))
}

pub(crate) fn project_phase6_scheduler_budget_overflow_error(
    budget_report: &DataLayerM10Phase6ExecutionTickBudgetReport,
    stage: DataLayerM10Phase6SchedulerBudgetOverflowStage,
) -> Option<DataLayerM10PartitionLifecycleError> {
    data_layer_m10_project_phase6_scheduler_budget_overflow_policy_error(
        map_phase6_budget_policy_report_from_core(budget_report),
        stage,
    )
    .map(map_phase6_budget_overflow_projection_to_core)
}
