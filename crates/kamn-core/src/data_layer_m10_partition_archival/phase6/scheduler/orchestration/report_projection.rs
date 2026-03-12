use kamn_data_layer::{
    data_layer_m10_project_phase6_scheduler_cycle_policy_report,
    DataLayerM10Phase6SchedulerCyclePolicyReport,
};

use crate::data_layer_m10_partition_archival::{
    DataLayerM10Phase6ExecutionTickBudgetReport, DataLayerM10Phase6ExecutionTickReport,
    DataLayerM10Phase6SchedulerCycleReport, DataLayerM10Phase6SchedulerTriggerDecision,
};

use super::super::super::policy_mapping::{
    map_phase6_scheduler_trigger_decision_from_policy,
    map_phase6_scheduler_trigger_decision_to_policy,
};

pub(super) fn project_phase6_scheduler_cycle_report(
    trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision,
    execution_report: Option<DataLayerM10Phase6ExecutionTickReport>,
    budget_report: Option<DataLayerM10Phase6ExecutionTickBudgetReport>,
) -> DataLayerM10Phase6SchedulerCycleReport {
    let cycle_policy_report: DataLayerM10Phase6SchedulerCyclePolicyReport<
        DataLayerM10Phase6ExecutionTickReport,
        DataLayerM10Phase6ExecutionTickBudgetReport,
    > = data_layer_m10_project_phase6_scheduler_cycle_policy_report(
        map_phase6_scheduler_trigger_decision_to_policy(trigger_decision),
        execution_report,
        budget_report,
    );
    DataLayerM10Phase6SchedulerCycleReport {
        trigger_decision: map_phase6_scheduler_trigger_decision_from_policy(
            cycle_policy_report.trigger_decision,
        ),
        execution_report: cycle_policy_report.execution_report,
        budget_report: cycle_policy_report.budget_report,
        reason_code: cycle_policy_report.reason_code,
    }
}
