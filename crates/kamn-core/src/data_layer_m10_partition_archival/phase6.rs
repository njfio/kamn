use std::collections::BTreeSet;

use crate::{
    DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest, DataLayerM8OwnerScopeQuery,
};

use super::error::{
    map_m8_execution_error_to_m10, map_phase6_owner_scope_error_to_m10,
    map_phase6_projection_error_to_m10, phase6_execution_failed,
};
use super::shared::{authorize_owner_scope, parse_kamn_did, validate_non_empty};
use super::*;

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
    validate_non_empty(input.owner_did.as_str(), "owner_did")?;
    let owner_did = parse_kamn_did(input.owner_did.as_str())?;
    if input.runtime_state.total_cycles
        != input.runtime_state.executed_cycles
            + input.runtime_state.deferred_cycles
            + input.runtime_state.fail_closed_cycles
    {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field: "runtime_state",
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
            },
        );
    }

    let trigger_reason_code = phase6_trigger_reason_code(&input.cycle_report.trigger_decision);
    let runtime_total_cycles = input.runtime_state.total_cycles;
    let runtime_executed_cycles = input.runtime_state.executed_cycles;
    let runtime_deferred_cycles = input.runtime_state.deferred_cycles;
    let runtime_fail_closed_cycles = input.runtime_state.fail_closed_cycles;
    let runtime_last_successful_tick_epoch_seconds =
        input.runtime_state.last_successful_tick_epoch_seconds;
    let runtime_last_observed_now_epoch_seconds =
        input.runtime_state.last_observed_now_epoch_seconds;
    let runtime_last_reason_code = input.runtime_state.last_reason_code;

    match input.cycle_report.reason_code {
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE => {
            let execution_report = input.cycle_report.execution_report.ok_or(
                DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                    field: "cycle_report",
                    reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
                },
            )?;
            let budget_report = input.cycle_report.budget_report.ok_or(
                DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                    field: "cycle_report",
                    reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
                },
            )?;
            let execution_owner_did = parse_kamn_did(execution_report.owner_did.as_str())?;
            if owner_did.as_str() != execution_owner_did.as_str() {
                return Err(
                    DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                        field: "owner_did",
                        reason_code:
                            DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
                    },
                );
            }

            let mut archived_entries = execution_report.archived_entries;
            archived_entries.sort_by(|left, right| {
                left.partition_month_id
                    .cmp(&right.partition_month_id)
                    .then(left.partition_name.cmp(&right.partition_name))
            });

            let archived_partition_names = archived_entries
                .iter()
                .map(|entry| entry.partition_name.clone())
                .collect();
            let archived_object_uris = archived_entries
                .iter()
                .map(|entry| entry.archived_object_uri.clone())
                .collect();

            Ok(DataLayerM10Phase6RuntimeEvidenceBundle {
                owner_did: owner_did.as_str().to_owned(),
                cycle_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
                trigger_reason_code,
                budget_reason_code: Some(budget_report.reason_code),
                archived_partition_names,
                archived_object_uris,
                due_candidate_count: execution_report.due_candidate_count,
                shredded_message_count: execution_report.shredded_message_ids.len(),
                projection_report_count: execution_report.projection_reports.len(),
                archived_entry_count: archived_entries.len(),
                runtime_total_cycles,
                runtime_executed_cycles,
                runtime_deferred_cycles,
                runtime_fail_closed_cycles,
                runtime_last_successful_tick_epoch_seconds,
                runtime_last_observed_now_epoch_seconds,
                runtime_last_reason_code,
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE,
            })
        }
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE => {
            if input.cycle_report.execution_report.is_some()
                || input.cycle_report.budget_report.is_some()
            {
                return Err(
                    DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                        field: "cycle_report",
                        reason_code:
                            DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
                    },
                );
            }
            let due_candidate_count = match input.cycle_report.trigger_decision {
                DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
                    due_candidate_count,
                    ..
                }
                | DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
                    due_candidate_count,
                    ..
                } => due_candidate_count,
            };

            Ok(DataLayerM10Phase6RuntimeEvidenceBundle {
                owner_did: owner_did.as_str().to_owned(),
                cycle_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
                trigger_reason_code,
                budget_reason_code: None,
                archived_partition_names: Vec::new(),
                archived_object_uris: Vec::new(),
                due_candidate_count,
                shredded_message_count: 0,
                projection_report_count: 0,
                archived_entry_count: 0,
                runtime_total_cycles,
                runtime_executed_cycles,
                runtime_deferred_cycles,
                runtime_fail_closed_cycles,
                runtime_last_successful_tick_epoch_seconds,
                runtime_last_observed_now_epoch_seconds,
                runtime_last_reason_code,
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE,
            })
        }
        _ => Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field: "cycle_report",
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
            },
        ),
    }
}

/// Executes one deterministic Phase-6 retention/shred/projection/archive orchestration tick.
pub fn data_layer_m10_execute_phase6_orchestration_tick(
    compliance_registry: &mut DataLayerM8ComplianceRegistry,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6ExecutionTickRequest,
) -> Result<DataLayerM10Phase6ExecutionTickReport, DataLayerM10PartitionLifecycleError> {
    let owner_did = authorize_owner_scope(
        request.requester_owner_did.as_str(),
        request.owner_did.as_str(),
    )
    .map_err(map_phase6_owner_scope_error_to_m10)?;
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

    let due_candidates = compliance_registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: request.requester_owner_did.clone(),
                owner_did: request.owner_did.clone(),
            },
            request.now_epoch_seconds,
        )
        .map_err(map_m8_execution_error_to_m10)?;
    let due_candidate_count = due_candidates.len();

    let mut shredded_message_ids = Vec::with_capacity(due_candidate_count);
    for candidate in due_candidates {
        compliance_registry
            .crypto_shred(DataLayerM8CryptoShredRequest {
                requester_owner_did: request.requester_owner_did.clone(),
                owner_did: request.owner_did.clone(),
                message_id: candidate.message_id.clone(),
                shredded_at_epoch_seconds: request.shredded_at_epoch_seconds,
            })
            .map_err(map_m8_execution_error_to_m10)?;
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
            let message = compliance_registry
                .message_for_owner(owner_did.as_str(), message_id.as_str())
                .map_err(map_m8_execution_error_to_m10)?;
            if message.legal_hold_active {
                return Err(phase6_execution_failed(
                    DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
                    format!("message {} is under legal hold", message.message_id),
                ));
            }
            deduped_partition_message_ids.insert(message_id);
        }

        let projection_report = partition_registry
            .project_partition_shred_completeness_from_m8(
                compliance_registry,
                DataLayerM10ComplianceShredProjectionRequest {
                    requester_owner_did: request.requester_owner_did.clone(),
                    owner_did: request.owner_did.clone(),
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
        owner_did: owner_did.as_str().to_owned(),
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
    validate_phase6_execution_tick_budget(budget)?;

    let budget_result = DataLayerM10Phase6ExecutionTickBudgetReport {
        decision: DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget,
        reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
        due_candidate_count: report.due_candidate_count,
        shredded_message_count: report.shredded_message_ids.len(),
        projection_report_count: report.projection_reports.len(),
        archived_entry_count: report.archived_entries.len(),
    };

    if budget_result.due_candidate_count > budget.max_due_candidates {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.shredded_message_count > budget.max_shredded_messages {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code:
                DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.projection_report_count > budget.max_projection_reports {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.archived_entry_count > budget.max_archived_entries {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code:
                DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }

    Ok(budget_result)
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
    validate_phase6_scheduler_policy(policy)?;
    let elapsed_since_last_tick_seconds = resolve_phase6_scheduler_elapsed(signal)?;
    if signal.due_candidate_count >= policy.due_candidate_trigger_threshold {
        return Ok(DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
            due_candidate_count: signal.due_candidate_count,
            elapsed_since_last_tick_seconds,
        });
    }
    if elapsed_since_last_tick_seconds >= policy.max_tick_interval_seconds {
        return Ok(DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_INTERVAL_ELAPSED_REASON_CODE,
            due_candidate_count: signal.due_candidate_count,
            elapsed_since_last_tick_seconds,
        });
    }
    Ok(DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
        reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
        due_candidate_count: signal.due_candidate_count,
        elapsed_since_last_tick_seconds,
    })
}

/// Executes one deterministic Phase-6 scheduler cycle with trigger + budget guardrails.
pub fn data_layer_m10_execute_phase6_scheduler_cycle(
    compliance_registry: &mut DataLayerM8ComplianceRegistry,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6SchedulerCycleRequest,
) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
    validate_phase6_execution_tick_budget(request.budget)?;
    let due_candidates = compliance_registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: request.execution_request.requester_owner_did.clone(),
                owner_did: request.execution_request.owner_did.clone(),
            },
            request.execution_request.now_epoch_seconds,
        )
        .map_err(map_m8_execution_error_to_m10)?;
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

    let execution_report = data_layer_m10_execute_phase6_orchestration_tick(
        compliance_registry,
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

fn phase6_trigger_reason_code(
    trigger_decision: &DataLayerM10Phase6SchedulerTriggerDecision,
) -> &'static str {
    match trigger_decision {
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred { reason_code, .. }
        | DataLayerM10Phase6SchedulerTriggerDecision::Triggered { reason_code, .. } => reason_code,
    }
}

fn validate_phase6_execution_tick_budget(
    budget: DataLayerM10Phase6ExecutionTickBudget,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if budget.max_due_candidates == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
                field: "max_due_candidates",
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
            },
        );
    }
    if budget.max_shredded_messages == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
                field: "max_shredded_messages",
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
            },
        );
    }
    if budget.max_projection_reports == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
                field: "max_projection_reports",
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
            },
        );
    }
    if budget.max_archived_entries == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
                field: "max_archived_entries",
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
            },
        );
    }
    Ok(())
}

fn validate_phase6_scheduler_policy(
    policy: DataLayerM10Phase6SchedulerPolicy,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if policy.due_candidate_trigger_threshold == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerPolicy {
                field: "due_candidate_trigger_threshold",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
            },
        );
    }
    if policy.max_tick_interval_seconds == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerPolicy {
                field: "max_tick_interval_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
            },
        );
    }
    Ok(())
}

fn resolve_phase6_scheduler_elapsed(
    signal: DataLayerM10Phase6SchedulerSignal,
) -> Result<u64, DataLayerM10PartitionLifecycleError> {
    if signal.now_epoch_seconds == 0 {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
                field: "now_epoch_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
            },
        );
    }
    match signal.last_tick_epoch_seconds {
        Some(last_tick_epoch_seconds) => {
            if last_tick_epoch_seconds > signal.now_epoch_seconds {
                return Err(
                    DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
                        field: "last_tick_epoch_seconds",
                        reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
                    },
                );
            }
            Ok(signal
                .now_epoch_seconds
                .saturating_sub(last_tick_epoch_seconds))
        }
        None => Ok(signal.now_epoch_seconds),
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
    validate_phase6_execution_tick_budget(budget)?;

    let budget_result = DataLayerM10Phase6ExecutionTickBudgetReport {
        decision: DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget,
        reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
        due_candidate_count,
        shredded_message_count: due_candidate_count,
        projection_report_count,
        archived_entry_count: projection_report_count,
    };
    if budget_result.due_candidate_count > budget.max_due_candidates {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.shredded_message_count > budget.max_shredded_messages {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code:
                DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.projection_report_count > budget.max_projection_reports {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.archived_entry_count > budget.max_archived_entries {
        return Ok(DataLayerM10Phase6ExecutionTickBudgetReport {
            decision: DataLayerM10Phase6ExecutionBudgetDecision::Exceeded,
            reason_code:
                DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    Ok(budget_result)
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
