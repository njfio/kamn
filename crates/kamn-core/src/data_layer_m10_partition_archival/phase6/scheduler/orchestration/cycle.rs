use kamn_data_layer::{
    DataLayerM10Phase6CompliancePort, DataLayerM10Phase6SchedulerBudgetOverflowStage,
};

use crate::data_layer_m10_partition_archival::{
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10Phase6SchedulerCycleReport, DataLayerM10Phase6SchedulerCycleRequest,
    DataLayerM10Phase6SchedulerSignal, DataLayerM10Phase6SchedulerTriggerDecision,
};
use crate::DataLayerM8ComplianceRegistry;

use super::report_projection::project_phase6_scheduler_cycle_report;
use super::tick::data_layer_m10_execute_phase6_orchestration_tick_with_port;
use super::super::budget::{
    data_layer_m10_evaluate_phase6_execution_tick_budget,
    evaluate_phase6_scheduler_preflight_budget, project_phase6_scheduler_budget_overflow_error,
    validate_phase6_execution_tick_budget,
};
use super::super::trigger::data_layer_m10_evaluate_phase6_scheduler_trigger;
use super::super::super::adapters::bridge::M8Phase6CompliancePortAdapter;
use super::super::super::adapters::error_mapping::map_phase6_port_error_to_m10;

pub fn data_layer_m10_execute_phase6_scheduler_cycle(
    compliance_registry: &mut DataLayerM8ComplianceRegistry,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6SchedulerCycleRequest,
) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
    let mut compliance_port = M8Phase6CompliancePortAdapter::new(compliance_registry);
    data_layer_m10_execute_phase6_scheduler_cycle_with_port(
        &mut compliance_port,
        partition_registry,
        request,
    )
}

pub fn data_layer_m10_execute_phase6_scheduler_cycle_with_port(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    mut request: DataLayerM10Phase6SchedulerCycleRequest,
) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
    validate_phase6_execution_tick_budget(request.budget)?;
    authorize_scheduler_owner(compliance_port, &mut request)?;
    let due_candidate_count = count_due_candidates(compliance_port, &request)?;
    let trigger_decision = evaluate_trigger(&request, due_candidate_count)?;
    if matches!(trigger_decision, DataLayerM10Phase6SchedulerTriggerDecision::Deferred { .. }) {
        return Ok(deferred_cycle_report(trigger_decision));
    }
    enforce_preflight_budget(&request, due_candidate_count)?;
    let execution_report =
        execute_scheduler_cycle(compliance_port, partition_registry, request.execution_request)?;
    finalize_applied_cycle_report(trigger_decision, execution_report, request.budget)
}

fn deferred_cycle_report(
    trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision,
) -> DataLayerM10Phase6SchedulerCycleReport {
    project_phase6_scheduler_cycle_report(trigger_decision, None, None)
}

fn execute_scheduler_cycle(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    execution_request: crate::DataLayerM10Phase6ExecutionTickRequest,
) -> Result<crate::DataLayerM10Phase6ExecutionTickReport, DataLayerM10PartitionLifecycleError> {
    data_layer_m10_execute_phase6_orchestration_tick_with_port(
        compliance_port,
        partition_registry,
        execution_request,
    )
}

fn finalize_applied_cycle_report(
    trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision,
    execution_report: crate::DataLayerM10Phase6ExecutionTickReport,
    budget: crate::DataLayerM10Phase6ExecutionTickBudget,
) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
    let budget_report = data_layer_m10_evaluate_phase6_execution_tick_budget(&execution_report, budget)?;
    enforce_post_execution_budget(&budget_report)?;
    Ok(project_phase6_scheduler_cycle_report(
        trigger_decision,
        Some(execution_report),
        Some(budget_report),
    ))
}

fn authorize_scheduler_owner(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    request: &mut DataLayerM10Phase6SchedulerCycleRequest,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    let owner_did = compliance_port
        .authorize_owner_scope(
            request.execution_request.requester_owner_did.as_str(),
            request.execution_request.owner_did.as_str(),
        )
        .map_err(map_phase6_port_error_to_m10)?;
    request.execution_request.requester_owner_did = owner_did.clone();
    request.execution_request.owner_did = owner_did;
    Ok(())
}

fn count_due_candidates(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    request: &DataLayerM10Phase6SchedulerCycleRequest,
) -> Result<usize, DataLayerM10PartitionLifecycleError> {
    Ok(compliance_port
        .retention_due_for_owner(
            request.execution_request.owner_did.as_str(),
            request.execution_request.now_epoch_seconds,
        )
        .map_err(map_phase6_port_error_to_m10)?
        .len())
}

fn evaluate_trigger(
    request: &DataLayerM10Phase6SchedulerCycleRequest,
    due_candidate_count: usize,
) -> Result<DataLayerM10Phase6SchedulerTriggerDecision, DataLayerM10PartitionLifecycleError> {
    data_layer_m10_evaluate_phase6_scheduler_trigger(
        request.scheduler_policy,
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count,
            last_tick_epoch_seconds: request.last_tick_epoch_seconds,
            now_epoch_seconds: request.execution_request.now_epoch_seconds,
        },
    )
}

fn enforce_preflight_budget(
    request: &DataLayerM10Phase6SchedulerCycleRequest,
    due_candidate_count: usize,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    let preflight_budget = evaluate_phase6_scheduler_preflight_budget(
        due_candidate_count,
        request.execution_request.partition_message_ids_by_month.len(),
        request.budget,
    )?;
    if let Some(error) = project_phase6_scheduler_budget_overflow_error(
        &preflight_budget,
        DataLayerM10Phase6SchedulerBudgetOverflowStage::Preflight,
    ) {
        return Err(error);
    }
    Ok(())
}

fn enforce_post_execution_budget(
    budget_report: &crate::DataLayerM10Phase6ExecutionTickBudgetReport,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if let Some(error) = project_phase6_scheduler_budget_overflow_error(
        budget_report,
        DataLayerM10Phase6SchedulerBudgetOverflowStage::PostExecution,
    ) {
        return Err(error);
    }
    Ok(())
}
