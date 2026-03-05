use super::{
    DataLayerM10Phase6PolicyArchivedEntry, DataLayerM10Phase6PolicyBudgetDecision,
    DataLayerM10Phase6PolicyCycleReport, DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
    DataLayerM10Phase6PolicyRuntimeEvidenceError, DataLayerM10Phase6PolicyRuntimeEvidenceInput,
    DataLayerM10Phase6PolicyRuntimeState, DataLayerM10Phase6PolicySchedulerCycleReason,
    DataLayerM10Phase6PolicySchedulerTriggerDecision,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
};

/// Projects canonical phase6 runtime evidence from one scheduler-cycle report and runtime state.
pub fn data_layer_m10_project_phase6_runtime_evidence_bundle(
    input: DataLayerM10Phase6PolicyRuntimeEvidenceInput,
) -> Result<
    DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
    DataLayerM10Phase6PolicyRuntimeEvidenceError,
> {
    let DataLayerM10Phase6PolicyRuntimeEvidenceInput {
        owner_did,
        cycle_report,
        runtime_state,
    } = input;
    validate_non_empty(owner_did.as_str(), "owner_did")?;
    validate_runtime_state(runtime_state)?;
    match cycle_report.reason_code {
        DataLayerM10Phase6PolicySchedulerCycleReason::Applied => {
            project_applied_cycle(owner_did, cycle_report, runtime_state)
        }
        DataLayerM10Phase6PolicySchedulerCycleReason::Deferred => {
            project_deferred_cycle(owner_did, cycle_report, runtime_state)
        }
    }
}

fn project_applied_cycle(
    owner_did: String,
    cycle_report: DataLayerM10Phase6PolicyCycleReport,
    runtime_state: DataLayerM10Phase6PolicyRuntimeState,
) -> Result<
    DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
    DataLayerM10Phase6PolicyRuntimeEvidenceError,
> {
    let trigger_reason_code = trigger_reason_code(&cycle_report.trigger_decision);
    let execution_report = cycle_report
        .execution_report
        .ok_or(invalid_input("cycle_report"))?;
    let budget_decision = cycle_report
        .budget_decision
        .ok_or(invalid_input("cycle_report"))?;
    if owner_did != execution_report.owner_did {
        return Err(invalid_input("owner_did"));
    }

    let mut archived_entries = execution_report.archived_entries;
    archived_entries.sort_by(|left, right| {
        left.partition_month_id
            .cmp(&right.partition_month_id)
            .then(left.partition_name.cmp(&right.partition_name))
    });
    let archived_partition_names = archived_partition_names(&archived_entries);
    let archived_object_uris = archived_object_uris(&archived_entries);

    Ok(DataLayerM10Phase6PolicyRuntimeEvidenceBundle {
        owner_did,
        cycle_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
        trigger_reason_code,
        budget_reason_code: Some(budget_reason_code(budget_decision)),
        archived_partition_names,
        archived_object_uris,
        due_candidate_count: execution_report.due_candidate_count,
        shredded_message_count: execution_report.shredded_message_count,
        projection_report_count: execution_report.projection_report_count,
        archived_entry_count: archived_entries.len(),
        runtime_total_cycles: runtime_state.total_cycles,
        runtime_executed_cycles: runtime_state.executed_cycles,
        runtime_deferred_cycles: runtime_state.deferred_cycles,
        runtime_fail_closed_cycles: runtime_state.fail_closed_cycles,
        runtime_last_successful_tick_epoch_seconds: runtime_state
            .last_successful_tick_epoch_seconds,
        runtime_last_observed_now_epoch_seconds: runtime_state.last_observed_now_epoch_seconds,
        runtime_last_reason_code: runtime_state.last_reason_code,
        reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE,
    })
}

fn project_deferred_cycle(
    owner_did: String,
    cycle_report: DataLayerM10Phase6PolicyCycleReport,
    runtime_state: DataLayerM10Phase6PolicyRuntimeState,
) -> Result<
    DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
    DataLayerM10Phase6PolicyRuntimeEvidenceError,
> {
    if cycle_report.execution_report.is_some() || cycle_report.budget_decision.is_some() {
        return Err(invalid_input("cycle_report"));
    }
    Ok(DataLayerM10Phase6PolicyRuntimeEvidenceBundle {
        owner_did,
        cycle_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
        trigger_reason_code: trigger_reason_code(&cycle_report.trigger_decision),
        budget_reason_code: None,
        archived_partition_names: Vec::new(),
        archived_object_uris: Vec::new(),
        due_candidate_count: due_candidate_count(&cycle_report.trigger_decision),
        shredded_message_count: 0,
        projection_report_count: 0,
        archived_entry_count: 0,
        runtime_total_cycles: runtime_state.total_cycles,
        runtime_executed_cycles: runtime_state.executed_cycles,
        runtime_deferred_cycles: runtime_state.deferred_cycles,
        runtime_fail_closed_cycles: runtime_state.fail_closed_cycles,
        runtime_last_successful_tick_epoch_seconds: runtime_state
            .last_successful_tick_epoch_seconds,
        runtime_last_observed_now_epoch_seconds: runtime_state.last_observed_now_epoch_seconds,
        runtime_last_reason_code: runtime_state.last_reason_code,
        reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE,
    })
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM10Phase6PolicyRuntimeEvidenceError> {
    if value.trim().is_empty() {
        return Err(invalid_input(field));
    }
    Ok(())
}

fn validate_runtime_state(
    runtime_state: DataLayerM10Phase6PolicyRuntimeState,
) -> Result<(), DataLayerM10Phase6PolicyRuntimeEvidenceError> {
    if runtime_state.total_cycles
        != runtime_state.executed_cycles
            + runtime_state.deferred_cycles
            + runtime_state.fail_closed_cycles
    {
        return Err(invalid_input("runtime_state"));
    }
    Ok(())
}

fn budget_reason_code(decision: DataLayerM10Phase6PolicyBudgetDecision) -> &'static str {
    match decision {
        DataLayerM10Phase6PolicyBudgetDecision::WithinBudget { reason_code }
        | DataLayerM10Phase6PolicyBudgetDecision::Exceeded { reason_code } => reason_code,
    }
}

fn trigger_reason_code(trigger: &DataLayerM10Phase6PolicySchedulerTriggerDecision) -> &'static str {
    match trigger {
        DataLayerM10Phase6PolicySchedulerTriggerDecision::Deferred { reason_code, .. }
        | DataLayerM10Phase6PolicySchedulerTriggerDecision::Triggered { reason_code, .. } => {
            reason_code
        }
    }
}

fn due_candidate_count(trigger: &DataLayerM10Phase6PolicySchedulerTriggerDecision) -> usize {
    match trigger {
        DataLayerM10Phase6PolicySchedulerTriggerDecision::Deferred {
            due_candidate_count,
            ..
        }
        | DataLayerM10Phase6PolicySchedulerTriggerDecision::Triggered {
            due_candidate_count,
            ..
        } => *due_candidate_count,
    }
}

fn archived_partition_names(entries: &[DataLayerM10Phase6PolicyArchivedEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| entry.partition_name.clone())
        .collect()
}

fn archived_object_uris(entries: &[DataLayerM10Phase6PolicyArchivedEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| entry.archived_object_uri.clone())
        .collect()
}

fn invalid_input(field: &'static str) -> DataLayerM10Phase6PolicyRuntimeEvidenceError {
    DataLayerM10Phase6PolicyRuntimeEvidenceError::InvalidInput {
        field,
        reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
    }
}
