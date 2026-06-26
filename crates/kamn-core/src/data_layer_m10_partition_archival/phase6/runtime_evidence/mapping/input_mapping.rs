use kamn_data_layer::data_layer_m10_phase6_runtime_evidence::{
    DataLayerM10Phase6PolicyArchivedEntry,
    DataLayerM10Phase6PolicyBudgetDecision as DataLayerM10Phase6RuntimeEvidenceBudgetDecision,
    DataLayerM10Phase6PolicyCycleReport, DataLayerM10Phase6PolicyExecutionReport,
    DataLayerM10Phase6PolicyRuntimeEvidenceInput, DataLayerM10Phase6PolicyRuntimeState,
    DataLayerM10Phase6PolicySchedulerCycleReason, DataLayerM10Phase6PolicySchedulerTriggerDecision,
};

use crate::data_layer_m10_partition_archival::shared::{parse_kamn_did, validate_non_empty};
use crate::data_layer_m10_partition_archival::{
    DataLayerM10PartitionLifecycleError, DataLayerM10Phase6ExecutionBudgetDecision,
    DataLayerM10Phase6ExecutionTickBudgetReport, DataLayerM10Phase6ExecutionTickReport,
    DataLayerM10Phase6RuntimeEvidenceInput, DataLayerM10Phase6SchedulerCycleReport,
    DataLayerM10Phase6SchedulerRuntimeState, DataLayerM10Phase6SchedulerTriggerDecision,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
};

pub(crate) fn map_phase6_runtime_evidence_input_to_policy(
    input: DataLayerM10Phase6RuntimeEvidenceInput,
) -> Result<DataLayerM10Phase6PolicyRuntimeEvidenceInput, DataLayerM10PartitionLifecycleError> {
    validate_non_empty(input.owner_did.as_str(), "owner_did")?;
    let owner_did = parse_kamn_did(input.owner_did.as_str())?;
    let cycle_report =
        map_phase6_runtime_evidence_cycle_report_to_policy(input.cycle_report, owner_did.as_str())?;
    Ok(DataLayerM10Phase6PolicyRuntimeEvidenceInput {
        owner_did: owner_did.as_str().to_owned(),
        cycle_report,
        runtime_state: map_phase6_runtime_state_to_policy(input.runtime_state),
    })
}

fn map_phase6_runtime_evidence_cycle_report_to_policy(
    report: DataLayerM10Phase6SchedulerCycleReport,
    owner_did: &str,
) -> Result<DataLayerM10Phase6PolicyCycleReport, DataLayerM10PartitionLifecycleError> {
    let execution_report = report
        .execution_report
        .map(map_phase6_runtime_execution_report_to_policy)
        .transpose()?;
    ensure_phase6_runtime_evidence_owner_matches(&execution_report, owner_did)?;
    Ok(DataLayerM10Phase6PolicyCycleReport {
        trigger_decision: map_phase6_runtime_trigger_decision_to_policy(report.trigger_decision),
        execution_report,
        budget_decision: report
            .budget_report
            .map(map_phase6_runtime_budget_decision_to_policy),
        reason_code: map_phase6_runtime_cycle_reason_to_policy(report.reason_code)?,
    })
}

fn ensure_phase6_runtime_evidence_owner_matches(
    execution_report: &Option<DataLayerM10Phase6PolicyExecutionReport>,
    owner_did: &str,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if execution_report
        .as_ref()
        .map(|execution_report| execution_report.owner_did.as_str())
        .is_some_and(|execution_owner_did| execution_owner_did != owner_did)
    {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field: "owner_did",
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
            },
        );
    }
    Ok(())
}

fn map_phase6_runtime_execution_report_to_policy(
    report: DataLayerM10Phase6ExecutionTickReport,
) -> Result<DataLayerM10Phase6PolicyExecutionReport, DataLayerM10PartitionLifecycleError> {
    let owner_did = parse_kamn_did(report.owner_did.as_str())?;
    Ok(DataLayerM10Phase6PolicyExecutionReport {
        owner_did: owner_did.as_str().to_owned(),
        due_candidate_count: report.due_candidate_count,
        shredded_message_count: report.shredded_message_ids.len(),
        projection_report_count: report.projection_reports.len(),
        archived_entries: report
            .archived_entries
            .into_iter()
            .map(|entry| DataLayerM10Phase6PolicyArchivedEntry {
                partition_month_id: entry.partition_month_id,
                partition_name: entry.partition_name,
                archived_object_uri: entry.archived_object_uri,
            })
            .collect(),
    })
}

fn map_phase6_runtime_budget_decision_to_policy(
    report: DataLayerM10Phase6ExecutionTickBudgetReport,
) -> DataLayerM10Phase6RuntimeEvidenceBudgetDecision {
    match report.decision {
        DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget => {
            DataLayerM10Phase6RuntimeEvidenceBudgetDecision::WithinBudget {
                reason_code: report.reason_code,
            }
        }
        DataLayerM10Phase6ExecutionBudgetDecision::Exceeded => {
            DataLayerM10Phase6RuntimeEvidenceBudgetDecision::Exceeded {
                reason_code: report.reason_code,
            }
        }
    }
}

fn map_phase6_runtime_cycle_reason_to_policy(
    reason_code: &'static str,
) -> Result<DataLayerM10Phase6PolicySchedulerCycleReason, DataLayerM10PartitionLifecycleError> {
    match reason_code {
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE => {
            Ok(DataLayerM10Phase6PolicySchedulerCycleReason::Applied)
        }
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE => {
            Ok(DataLayerM10Phase6PolicySchedulerCycleReason::Deferred)
        }
        _ => Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field: "cycle_report",
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
            },
        ),
    }
}

fn map_phase6_runtime_trigger_decision_to_policy(
    decision: DataLayerM10Phase6SchedulerTriggerDecision,
) -> DataLayerM10Phase6PolicySchedulerTriggerDecision {
    match decision {
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => DataLayerM10Phase6PolicySchedulerTriggerDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        },
        DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => DataLayerM10Phase6PolicySchedulerTriggerDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        },
    }
}

fn map_phase6_runtime_state_to_policy(
    runtime_state: DataLayerM10Phase6SchedulerRuntimeState,
) -> DataLayerM10Phase6PolicyRuntimeState {
    DataLayerM10Phase6PolicyRuntimeState {
        last_successful_tick_epoch_seconds: runtime_state.last_successful_tick_epoch_seconds,
        last_observed_now_epoch_seconds: runtime_state.last_observed_now_epoch_seconds,
        total_cycles: runtime_state.total_cycles,
        executed_cycles: runtime_state.executed_cycles,
        deferred_cycles: runtime_state.deferred_cycles,
        fail_closed_cycles: runtime_state.fail_closed_cycles,
        last_reason_code: runtime_state.last_reason_code,
    }
}
