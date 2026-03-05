use std::collections::BTreeSet;

use kamn_data_layer::data_layer_m10_phase6_runtime_evidence::{
    data_layer_m10_project_phase6_runtime_evidence_bundle as data_layer_m10_project_phase6_runtime_evidence_bundle_policy,
    DataLayerM10Phase6PolicyArchivedEntry,
    DataLayerM10Phase6PolicyBudgetDecision as DataLayerM10Phase6RuntimeEvidenceBudgetDecision,
    DataLayerM10Phase6PolicyCycleReport, DataLayerM10Phase6PolicyExecutionReport,
    DataLayerM10Phase6PolicyRuntimeEvidenceBundle, DataLayerM10Phase6PolicyRuntimeEvidenceError,
    DataLayerM10Phase6PolicyRuntimeEvidenceInput, DataLayerM10Phase6PolicyRuntimeState,
    DataLayerM10Phase6PolicySchedulerCycleReason, DataLayerM10Phase6PolicySchedulerTriggerDecision,
};
use kamn_data_layer::{
    data_layer_m10_evaluate_phase6_execution_tick_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_trigger_policy,
    data_layer_m10_validate_phase6_execution_budget_policy,
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config,
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10Phase6BudgetPolicyReport,
    DataLayerM10Phase6CompliancePort, DataLayerM10Phase6CompliancePortError,
    DataLayerM10Phase6CryptoShredInput, DataLayerM10Phase6PolicyBudget,
    DataLayerM10Phase6PolicyBudgetDecision, DataLayerM10Phase6PolicyEvaluatorError,
    DataLayerM10Phase6PolicyReportCounts, DataLayerM10Phase6RetentionDueCandidate,
    DataLayerM10Phase6SchedulerSignalPolicy, DataLayerM10Phase6SchedulerTriggerPolicy,
    DataLayerM10Phase6TriggerPolicyDecision,
};

use crate::{
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest,
    DataLayerM8OwnerScopeQuery,
};

use super::error::{map_phase6_projection_error_to_m10, phase6_execution_failed};
use super::shared::{authorize_owner_scope, parse_kamn_did, validate_non_empty};
use super::*;

struct M8Phase6CompliancePortAdapter<'a> {
    compliance_registry: &'a mut DataLayerM8ComplianceRegistry,
}

impl<'a> M8Phase6CompliancePortAdapter<'a> {
    fn new(compliance_registry: &'a mut DataLayerM8ComplianceRegistry) -> Self {
        Self {
            compliance_registry,
        }
    }
}

impl DataLayerM10Phase6CompliancePort for M8Phase6CompliancePortAdapter<'_> {
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10Phase6CompliancePortError> {
        let owner_did = authorize_owner_scope(requester_owner_did, owner_did)
            .map_err(map_phase6_owner_scope_error_to_phase6_port)?;
        Ok(owner_did.as_str().to_owned())
    }

    fn retention_due_for_owner(
        &self,
        owner_did: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM10Phase6RetentionDueCandidate>, DataLayerM10Phase6CompliancePortError>
    {
        self.compliance_registry
            .retention_due_for_owner(
                DataLayerM8OwnerScopeQuery {
                    requester_owner_did: owner_did.to_owned(),
                    owner_did: owner_did.to_owned(),
                },
                now_epoch_seconds,
            )
            .map_err(map_m8_execution_error_to_phase6_port)
            .map(|candidates| {
                candidates
                    .into_iter()
                    .map(|candidate| DataLayerM10Phase6RetentionDueCandidate {
                        message_id: candidate.message_id,
                    })
                    .collect()
            })
    }

    fn crypto_shred(
        &mut self,
        input: DataLayerM10Phase6CryptoShredInput,
    ) -> Result<(), DataLayerM10Phase6CompliancePortError> {
        self.compliance_registry
            .crypto_shred(DataLayerM8CryptoShredRequest {
                requester_owner_did: input.requester_owner_did,
                owner_did: input.owner_did,
                message_id: input.message_id,
                shredded_at_epoch_seconds: input.shredded_at_epoch_seconds,
            })
            .map(|_| ())
            .map_err(map_m8_execution_error_to_phase6_port)
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10Phase6CompliancePortError>
    {
        self.compliance_registry
            .message_for_owner(owner_did, message_id)
            .map_err(map_m8_execution_error_to_phase6_port)
            .map(|message| DataLayerM10ComplianceProjectionMessageState {
                message_id: message.message_id.clone(),
                legal_hold_active: message.legal_hold_active,
                shredded_at_epoch_seconds: message.shredded_at_epoch_seconds,
            })
    }
}

struct Phase6ProjectionPortBridge<'a, T: DataLayerM10Phase6CompliancePort> {
    phase6_port: &'a T,
}

impl<'a, T: DataLayerM10Phase6CompliancePort> Phase6ProjectionPortBridge<'a, T> {
    fn new(phase6_port: &'a T) -> Self {
        Self { phase6_port }
    }
}

impl<T: DataLayerM10Phase6CompliancePort> DataLayerM10ComplianceProjectionPort
    for Phase6ProjectionPortBridge<'_, T>
{
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10ComplianceProjectionPortError> {
        self.phase6_port
            .authorize_owner_scope(requester_owner_did, owner_did)
            .map_err(map_phase6_port_error_to_projection_port_error)
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<
        DataLayerM10ComplianceProjectionMessageState,
        DataLayerM10ComplianceProjectionPortError,
    > {
        self.phase6_port
            .message_for_owner(owner_did, message_id)
            .map_err(map_phase6_port_error_to_projection_port_error)
    }
}

fn map_phase6_port_error_to_projection_port_error(
    error: DataLayerM10Phase6CompliancePortError,
) -> DataLayerM10ComplianceProjectionPortError {
    match error {
        DataLayerM10Phase6CompliancePortError::OwnerScopeViolation => {
            DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation
        }
        DataLayerM10Phase6CompliancePortError::LookupFailed(detail) => {
            DataLayerM10ComplianceProjectionPortError::LookupFailed(detail)
        }
        DataLayerM10Phase6CompliancePortError::InvalidInput(detail) => {
            DataLayerM10ComplianceProjectionPortError::InvalidInput(detail)
        }
    }
}

fn map_m8_execution_error_to_phase6_port(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10Phase6CompliancePortError {
    match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DataLayerM10Phase6CompliancePortError::OwnerScopeViolation
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. } => {
            DataLayerM10Phase6CompliancePortError::LookupFailed(error.to_string())
        }
        DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::LegalHoldActive { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DataLayerM10Phase6CompliancePortError::InvalidInput(error.to_string())
        }
    }
}

fn map_phase6_owner_scope_error_to_phase6_port(
    error: DataLayerM10PartitionLifecycleError,
) -> DataLayerM10Phase6CompliancePortError {
    match error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { .. } => {
            DataLayerM10Phase6CompliancePortError::OwnerScopeViolation
        }
        DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed { detail, .. } => {
            DataLayerM10Phase6CompliancePortError::InvalidInput(detail)
        }
        other => DataLayerM10Phase6CompliancePortError::InvalidInput(other.to_string()),
    }
}

fn map_phase6_port_error_to_m10(
    error: DataLayerM10Phase6CompliancePortError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10Phase6CompliancePortError::OwnerScopeViolation => phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE,
            "phase6 owner scope authorization failed",
        ),
        DataLayerM10Phase6CompliancePortError::LookupFailed(detail)
        | DataLayerM10Phase6CompliancePortError::InvalidInput(detail) => phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
            detail,
        ),
    }
}

fn map_data_layer_policy_error_to_m10(
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

impl DataLayerM10Phase6SchedulerPolicy {
    /// Creates a scheduler policy and validates threshold/interval values.
    pub fn new(
        due_candidate_trigger_threshold: usize,
        max_tick_interval_seconds: u64,
    ) -> Result<Self, DataLayerM10PartitionLifecycleError> {
        let policy = Self {
            due_candidate_trigger_threshold,
            max_tick_interval_seconds,
        };
        validate_phase6_scheduler_policy(policy)?;
        Ok(policy)
    }
}

impl DataLayerM10Phase6SchedulerRuntime {
    /// Creates a stateful scheduler runtime with deterministic zeroed counters.
    pub fn new(
        scheduler_policy: DataLayerM10Phase6SchedulerPolicy,
        budget: DataLayerM10Phase6ExecutionTickBudget,
    ) -> Result<Self, DataLayerM10PartitionLifecycleError> {
        validate_phase6_scheduler_policy(scheduler_policy)?;
        validate_phase6_execution_tick_budget(budget)?;
        Ok(Self {
            scheduler_policy,
            budget,
            state: DataLayerM10Phase6SchedulerRuntimeState {
                last_successful_tick_epoch_seconds: None,
                last_observed_now_epoch_seconds: None,
                total_cycles: 0,
                executed_cycles: 0,
                deferred_cycles: 0,
                fail_closed_cycles: 0,
                last_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_RUNTIME_INITIALIZED_REASON_CODE,
            },
        })
    }

    /// Returns an immutable snapshot of runtime scheduler state.
    pub fn state(&self) -> &DataLayerM10Phase6SchedulerRuntimeState {
        &self.state
    }

    /// Runs one stateful Phase-6 scheduler cycle and updates runtime checkpoint/counters.
    pub fn run_cycle(
        &mut self,
        compliance_registry: &mut DataLayerM8ComplianceRegistry,
        partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
        execution_request: DataLayerM10Phase6ExecutionTickRequest,
    ) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
        self.state.total_cycles = self.state.total_cycles.saturating_add(1);

        if let Err(error) = validate_phase6_scheduler_runtime_clock(
            execution_request.now_epoch_seconds,
            self.state.last_observed_now_epoch_seconds,
        ) {
            self.state.fail_closed_cycles = self.state.fail_closed_cycles.saturating_add(1);
            self.state.last_reason_code = phase6_scheduler_error_reason_code(&error);
            return Err(error);
        }

        self.state.last_observed_now_epoch_seconds = Some(execution_request.now_epoch_seconds);
        let cycle_result = data_layer_m10_execute_phase6_scheduler_cycle(
            compliance_registry,
            partition_registry,
            DataLayerM10Phase6SchedulerCycleRequest {
                scheduler_policy: self.scheduler_policy,
                last_tick_epoch_seconds: self.state.last_successful_tick_epoch_seconds,
                budget: self.budget,
                execution_request,
            },
        );
        match cycle_result {
            Ok(report) => {
                if report.reason_code == DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE {
                    self.state.executed_cycles = self.state.executed_cycles.saturating_add(1);
                    self.state.last_successful_tick_epoch_seconds =
                        self.state.last_observed_now_epoch_seconds;
                } else {
                    self.state.deferred_cycles = self.state.deferred_cycles.saturating_add(1);
                }
                self.state.last_reason_code = report.reason_code;
                Ok(report)
            }
            Err(error) => {
                self.state.fail_closed_cycles = self.state.fail_closed_cycles.saturating_add(1);
                self.state.last_reason_code = phase6_scheduler_error_reason_code(&error);
                Err(error)
            }
        }
    }
}
/// Projects canonical Phase-6 runtime evidence from one scheduler-cycle report and runtime state.
pub fn data_layer_m10_project_phase6_runtime_evidence_bundle(
    input: DataLayerM10Phase6RuntimeEvidenceInput,
) -> Result<DataLayerM10Phase6RuntimeEvidenceBundle, DataLayerM10PartitionLifecycleError> {
    let policy_input = map_phase6_runtime_evidence_input_to_policy(input)?;
    let policy_bundle = data_layer_m10_project_phase6_runtime_evidence_bundle_policy(policy_input)
        .map_err(map_data_layer_runtime_evidence_error_to_m10)?;
    Ok(map_phase6_runtime_evidence_bundle_from_policy(
        policy_bundle,
    ))
}

fn map_data_layer_runtime_evidence_error_to_m10(
    error: DataLayerM10Phase6PolicyRuntimeEvidenceError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10Phase6PolicyRuntimeEvidenceError::InvalidInput { field, reason_code } => {
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field,
                reason_code,
            }
        }
    }
}

fn map_phase6_runtime_evidence_input_to_policy(
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

fn map_phase6_runtime_evidence_bundle_from_policy(
    bundle: DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
) -> DataLayerM10Phase6RuntimeEvidenceBundle {
    DataLayerM10Phase6RuntimeEvidenceBundle {
        owner_did: bundle.owner_did,
        cycle_reason_code: bundle.cycle_reason_code,
        trigger_reason_code: bundle.trigger_reason_code,
        budget_reason_code: bundle.budget_reason_code,
        archived_partition_names: bundle.archived_partition_names,
        archived_object_uris: bundle.archived_object_uris,
        due_candidate_count: bundle.due_candidate_count,
        shredded_message_count: bundle.shredded_message_count,
        projection_report_count: bundle.projection_report_count,
        archived_entry_count: bundle.archived_entry_count,
        runtime_total_cycles: bundle.runtime_total_cycles,
        runtime_executed_cycles: bundle.runtime_executed_cycles,
        runtime_deferred_cycles: bundle.runtime_deferred_cycles,
        runtime_fail_closed_cycles: bundle.runtime_fail_closed_cycles,
        runtime_last_successful_tick_epoch_seconds: bundle
            .runtime_last_successful_tick_epoch_seconds,
        runtime_last_observed_now_epoch_seconds: bundle.runtime_last_observed_now_epoch_seconds,
        runtime_last_reason_code: bundle.runtime_last_reason_code,
        reason_code: bundle.reason_code,
    }
}

/// Executes one deterministic Phase-6 retention/shred/projection/archive orchestration tick.
pub fn data_layer_m10_execute_phase6_orchestration_tick(
    compliance_registry: &mut DataLayerM8ComplianceRegistry,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6ExecutionTickRequest,
) -> Result<DataLayerM10Phase6ExecutionTickReport, DataLayerM10PartitionLifecycleError> {
    let mut compliance_port = M8Phase6CompliancePortAdapter::new(compliance_registry);
    data_layer_m10_execute_phase6_orchestration_tick_with_port(
        &mut compliance_port,
        partition_registry,
        request,
    )
}

/// Executes one deterministic Phase-6 retention/shred/projection/archive orchestration tick
/// through a core-agnostic compliance seam.
pub fn data_layer_m10_execute_phase6_orchestration_tick_with_port(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6ExecutionTickRequest,
) -> Result<DataLayerM10Phase6ExecutionTickReport, DataLayerM10PartitionLifecycleError> {
    let owner_did = compliance_port
        .authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )
        .map_err(map_phase6_port_error_to_m10)?;
    if request.now_epoch_seconds == 0 {
        return Err(phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
            "now_epoch_seconds must be > 0",
        ));
    }
    if request.shredded_at_epoch_seconds == 0 {
        return Err(phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
            "shredded_at_epoch_seconds must be > 0",
        ));
    }
    validate_non_empty(
        request.object_storage_prefix.as_str(),
        "object_storage_prefix",
    )
    .map_err(map_phase6_projection_error_to_m10)?;

    let due_candidates = compliance_port
        .retention_due_for_owner(owner_did.as_str(), request.now_epoch_seconds)
        .map_err(map_phase6_port_error_to_m10)?;
    let due_candidate_count = due_candidates.len();

    let mut shredded_message_ids = Vec::with_capacity(due_candidate_count);
    for candidate in due_candidates {
        compliance_port
            .crypto_shred(DataLayerM10Phase6CryptoShredInput {
                requester_owner_did: owner_did.clone(),
                owner_did: owner_did.clone(),
                message_id: candidate.message_id.clone(),
                shredded_at_epoch_seconds: request.shredded_at_epoch_seconds,
            })
            .map_err(map_phase6_port_error_to_m10)?;
        shredded_message_ids.push(candidate.message_id);
    }
    shredded_message_ids.sort();

    let mut projection_reports = Vec::with_capacity(request.partition_message_ids_by_month.len());
    for (partition_month_id, partition_message_ids) in request.partition_message_ids_by_month {
        if partition_message_ids.is_empty() {
            return Err(phase6_execution_failed(
                DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
                format!("partition {partition_month_id} message set is empty"),
            ));
        }

        let mut deduped_partition_message_ids = BTreeSet::new();
        for message_id in partition_message_ids {
            validate_non_empty(message_id.as_str(), "partition_message_ids")
                .map_err(map_phase6_projection_error_to_m10)?;
            let message = compliance_port
                .message_for_owner(owner_did.as_str(), message_id.as_str())
                .map_err(map_phase6_port_error_to_m10)?;
            if message.legal_hold_active {
                return Err(phase6_execution_failed(
                    DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
                    format!("message {} is under legal hold", message.message_id),
                ));
            }
            deduped_partition_message_ids.insert(message_id);
        }

        let projection_port = Phase6ProjectionPortBridge::new(&*compliance_port);
        let projection_report = partition_registry
            .project_partition_shred_completeness_with_port(
                &projection_port,
                DataLayerM10ComplianceShredProjectionRequest {
                    requester_owner_did: owner_did.clone(),
                    owner_did: owner_did.clone(),
                    partition_month_id,
                    partition_message_ids: deduped_partition_message_ids.into_iter().collect(),
                },
            )
            .map_err(map_phase6_projection_error_to_m10)?;
        projection_reports.push(projection_report);
    }
    projection_reports.sort_by(|left, right| {
        left.partition_month_id
            .cmp(&right.partition_month_id)
            .then(left.partition_name.cmp(&right.partition_name))
    });

    let archived_entries = partition_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: request.now_month_id,
            active_retention_months: request.active_retention_months,
            object_storage_prefix: request.object_storage_prefix,
        })
        .map_err(map_phase6_projection_error_to_m10)?;

    Ok(DataLayerM10Phase6ExecutionTickReport {
        owner_did,
        due_candidate_count,
        shredded_message_ids,
        projection_reports,
        archived_entries,
        reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
    })
}

/// Evaluates one Phase-6 execution report against deterministic per-tick budget limits.
pub fn data_layer_m10_evaluate_phase6_execution_tick_budget(
    report: &DataLayerM10Phase6ExecutionTickReport,
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> Result<DataLayerM10Phase6ExecutionTickBudgetReport, DataLayerM10PartitionLifecycleError> {
    let budget_policy_report = data_layer_m10_evaluate_phase6_execution_tick_budget_policy(
        DataLayerM10Phase6PolicyReportCounts {
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

fn map_phase6_policy_budget_from_core(
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> DataLayerM10Phase6PolicyBudget {
    DataLayerM10Phase6PolicyBudget {
        max_due_candidates: budget.max_due_candidates,
        max_shredded_messages: budget.max_shredded_messages,
        max_projection_reports: budget.max_projection_reports,
        max_archived_entries: budget.max_archived_entries,
    }
}

fn map_phase6_budget_policy_report_to_core(
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

/// Evaluates deterministic scheduler trigger decision for a Phase-6 tick cycle.
///
/// Decision precedence is fixed:
/// 1. due-candidate threshold reached
/// 2. interval elapsed
/// 3. deferred
pub fn data_layer_m10_evaluate_phase6_scheduler_trigger(
    policy: DataLayerM10Phase6SchedulerPolicy,
    signal: DataLayerM10Phase6SchedulerSignal,
) -> Result<DataLayerM10Phase6SchedulerTriggerDecision, DataLayerM10PartitionLifecycleError> {
    let trigger_decision = data_layer_m10_evaluate_phase6_scheduler_trigger_policy(
        map_phase6_scheduler_trigger_policy_from_core(policy),
        DataLayerM10Phase6SchedulerSignalPolicy {
            due_candidate_count: signal.due_candidate_count,
            last_tick_epoch_seconds: signal.last_tick_epoch_seconds,
            now_epoch_seconds: signal.now_epoch_seconds,
        },
    )
    .map_err(map_data_layer_policy_error_to_m10)?;
    match trigger_decision {
        DataLayerM10Phase6TriggerPolicyDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => Ok(DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        }),
        DataLayerM10Phase6TriggerPolicyDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        } => Ok(DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code,
            due_candidate_count,
            elapsed_since_last_tick_seconds,
        }),
    }
}

/// Executes one deterministic Phase-6 scheduler cycle with trigger + budget guardrails.
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

/// Executes one deterministic Phase-6 scheduler cycle with trigger + budget guardrails
/// through a core-agnostic compliance seam.
pub fn data_layer_m10_execute_phase6_scheduler_cycle_with_port(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    mut request: DataLayerM10Phase6SchedulerCycleRequest,
) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
    validate_phase6_execution_tick_budget(request.budget)?;
    let owner_did = compliance_port
        .authorize_owner_scope(
            request.execution_request.requester_owner_did.as_str(),
            request.execution_request.owner_did.as_str(),
        )
        .map_err(map_phase6_port_error_to_m10)?;
    request.execution_request.requester_owner_did = owner_did.clone();
    request.execution_request.owner_did = owner_did;

    let due_candidates = compliance_port
        .retention_due_for_owner(
            request.execution_request.owner_did.as_str(),
            request.execution_request.now_epoch_seconds,
        )
        .map_err(map_phase6_port_error_to_m10)?;
    let due_candidate_count = due_candidates.len();
    let trigger_decision = data_layer_m10_evaluate_phase6_scheduler_trigger(
        request.scheduler_policy,
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count,
            last_tick_epoch_seconds: request.last_tick_epoch_seconds,
            now_epoch_seconds: request.execution_request.now_epoch_seconds,
        },
    )?;
    if matches!(
        trigger_decision,
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred { .. }
    ) {
        return Ok(DataLayerM10Phase6SchedulerCycleReport {
            trigger_decision,
            execution_report: None,
            budget_report: None,
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
        });
    }

    let preflight_budget = evaluate_phase6_scheduler_preflight_budget(
        due_candidate_count,
        request
            .execution_request
            .partition_message_ids_by_month
            .len(),
        request.budget,
    )?;
    if preflight_budget.decision == DataLayerM10Phase6ExecutionBudgetDecision::Exceeded {
        return Err(
            DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded {
                reason_code: preflight_budget.reason_code,
                detail: format!(
                    "due={},shredded={},projections={},archives={}",
                    preflight_budget.due_candidate_count,
                    preflight_budget.shredded_message_count,
                    preflight_budget.projection_report_count,
                    preflight_budget.archived_entry_count
                ),
            },
        );
    }

    let execution_report = data_layer_m10_execute_phase6_orchestration_tick_with_port(
        compliance_port,
        partition_registry,
        request.execution_request,
    )?;
    let budget_report =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&execution_report, request.budget)?;
    if budget_report.decision == DataLayerM10Phase6ExecutionBudgetDecision::Exceeded {
        return Err(
            DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded {
                reason_code: budget_report.reason_code,
                detail: "post-execution budget overflow indicates stale preflight assumptions"
                    .to_owned(),
            },
        );
    }

    Ok(DataLayerM10Phase6SchedulerCycleReport {
        trigger_decision,
        execution_report: Some(execution_report),
        budget_report: Some(budget_report),
        reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
    })
}

fn validate_phase6_execution_tick_budget(
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    data_layer_m10_validate_phase6_execution_budget_policy(map_phase6_policy_budget_from_core(
        budget,
    ))
    .map_err(map_data_layer_policy_error_to_m10)
}

fn validate_phase6_scheduler_policy(
    policy: DataLayerM10Phase6SchedulerPolicy,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config(
        map_phase6_scheduler_trigger_policy_from_core(policy),
    )
    .map_err(map_data_layer_policy_error_to_m10)
}

fn map_phase6_scheduler_trigger_policy_from_core(
    policy: DataLayerM10Phase6SchedulerPolicy,
) -> DataLayerM10Phase6SchedulerTriggerPolicy {
    DataLayerM10Phase6SchedulerTriggerPolicy {
        due_candidate_trigger_threshold: policy.due_candidate_trigger_threshold,
        max_tick_interval_seconds: policy.max_tick_interval_seconds,
    }
}

fn validate_phase6_scheduler_runtime_clock(
    now_epoch_seconds: u64,
    last_observed_now_epoch_seconds: Option<u64>,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if now_epoch_seconds == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
                field: "now_epoch_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
            },
        );
    }
    if let Some(last_observed_now_epoch_seconds) = last_observed_now_epoch_seconds {
        if now_epoch_seconds < last_observed_now_epoch_seconds {
            return Err(
                DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
                    field: "now_epoch_seconds",
                    reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
                },
            );
        }
    }
    Ok(())
}

fn evaluate_phase6_scheduler_preflight_budget(
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

fn phase6_scheduler_error_reason_code(error: &DataLayerM10PartitionLifecycleError) -> &'static str {
    match error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { reason_code }
        | DataLayerM10PartitionLifecycleError::InvalidRetryPolicy { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::InvalidRetryAttempt { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
            reason_code, ..
        }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerPolicy {
            reason_code, ..
        }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
            reason_code, ..
        }
        | DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded {
            reason_code,
            ..
        }
        | DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
            reason_code,
            ..
        } => reason_code,
        DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::InvalidLifecycleTransition { reason_code, .. }
        | DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed { reason_code, .. } => {
            reason_code
        }
        DataLayerM10PartitionLifecycleError::EmptyField(_)
        | DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(_)
        | DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(_)
        | DataLayerM10PartitionLifecycleError::PartitionNotFound(_) => {
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE
        }
    }
}
