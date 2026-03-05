//! M10 phase6 policy evaluators extracted from core.
//!
//! This module hosts deterministic budget and scheduler-trigger policy
//! projections for the M10 phase6 execution loop.

/// Stable reason marker when Phase-6 execution tick budget is within configured limits.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE: &str =
    "m10_phase6_execution_budget_within_limit";
/// Stable reason marker when Phase-6 due candidate count exceeds budget.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE: &str =
    "m10_phase6_execution_budget_due_candidates_exceeded";
/// Stable reason marker when Phase-6 shred operation count exceeds budget.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE: &str =
    "m10_phase6_execution_budget_shredded_messages_exceeded";
/// Stable reason marker when Phase-6 projection count exceeds budget.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE: &str =
    "m10_phase6_execution_budget_projections_exceeded";
/// Stable reason marker when Phase-6 archive entry count exceeds budget.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE: &str =
    "m10_phase6_execution_budget_archive_entries_exceeded";
/// Stable reason marker when Phase-6 execution budget configuration is invalid.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE: &str =
    "m10_phase6_execution_budget_invalid";
/// Stable reason marker when Phase-6 scheduler trigger decision is deferred.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE: &str =
    "m10_phase6_scheduler_trigger_deferred";
/// Stable reason marker when Phase-6 scheduler trigger fires on due-candidate threshold.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE: &str =
    "m10_phase6_scheduler_trigger_due_threshold";
/// Stable reason marker when Phase-6 scheduler trigger fires on elapsed tick interval.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_INTERVAL_ELAPSED_REASON_CODE: &str =
    "m10_phase6_scheduler_trigger_interval_elapsed";
/// Stable reason marker when Phase-6 scheduler policy configuration is invalid.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE: &str =
    "m10_phase6_scheduler_policy_invalid";
/// Stable reason marker when Phase-6 scheduler signal metadata is invalid.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE: &str =
    "m10_phase6_scheduler_signal_invalid";

/// Fail-closed policy error taxonomy for M10 phase6 evaluator contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6PolicyEvaluatorError {
    /// Invalid phase6 execution budget configuration field.
    InvalidBudgetField {
        /// Invalid field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Invalid scheduler policy configuration field.
    InvalidSchedulerPolicyField {
        /// Invalid field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Invalid scheduler signal field.
    InvalidSchedulerSignalField {
        /// Invalid field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

/// Phase6 execution budget limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyBudget {
    /// Max due candidates per tick.
    pub max_due_candidates: usize,
    /// Max shredded message operations per tick.
    pub max_shredded_messages: usize,
    /// Max projection reports per tick.
    pub max_projection_reports: usize,
    /// Max archived entries per tick.
    pub max_archived_entries: usize,
}

/// Count summary from one phase6 execution report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyReportCounts {
    /// Due candidate count.
    pub due_candidate_count: usize,
    /// Shredded message count.
    pub shredded_message_count: usize,
    /// Projection report count.
    pub projection_report_count: usize,
    /// Archived entry count.
    pub archived_entry_count: usize,
}

/// Budget policy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6PolicyBudgetDecision {
    /// Counts are within budget.
    WithinBudget,
    /// At least one dimension exceeds budget.
    Exceeded,
}

/// Budget evaluation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6BudgetPolicyReport {
    /// Budget decision.
    pub decision: DataLayerM10Phase6PolicyBudgetDecision,
    /// Stable reason marker.
    pub reason_code: &'static str,
    /// Due candidate count.
    pub due_candidate_count: usize,
    /// Shredded message count.
    pub shredded_message_count: usize,
    /// Projection report count.
    pub projection_report_count: usize,
    /// Archived entry count.
    pub archived_entry_count: usize,
}

/// Scheduler trigger policy inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerTriggerPolicy {
    /// Threshold that forces a trigger based on due candidates.
    pub due_candidate_trigger_threshold: usize,
    /// Maximum interval allowed between phase6 ticks.
    pub max_tick_interval_seconds: u64,
}

/// Scheduler signal inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerSignalPolicy {
    /// Due candidate count observed for this cycle.
    pub due_candidate_count: usize,
    /// Last successful tick timestamp in epoch seconds.
    pub last_tick_epoch_seconds: Option<u64>,
    /// Current time in epoch seconds.
    pub now_epoch_seconds: u64,
}

/// Scheduler trigger policy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6TriggerPolicyDecision {
    /// Trigger is deferred.
    Deferred {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Due candidate count.
        due_candidate_count: usize,
        /// Elapsed seconds since last tick.
        elapsed_since_last_tick_seconds: u64,
    },
    /// Trigger is fired.
    Triggered {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Due candidate count.
        due_candidate_count: usize,
        /// Elapsed seconds since last tick.
        elapsed_since_last_tick_seconds: u64,
    },
}

/// Evaluates one phase6 execution report against deterministic per-tick budget limits.
pub fn data_layer_m10_evaluate_phase6_execution_tick_budget_policy(
    counts: DataLayerM10Phase6PolicyReportCounts,
    budget: DataLayerM10Phase6PolicyBudget,
) -> Result<DataLayerM10Phase6BudgetPolicyReport, DataLayerM10Phase6PolicyEvaluatorError> {
    validate_budget(budget)?;
    let budget_result = DataLayerM10Phase6BudgetPolicyReport {
        decision: DataLayerM10Phase6PolicyBudgetDecision::WithinBudget,
        reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
        due_candidate_count: counts.due_candidate_count,
        shredded_message_count: counts.shredded_message_count,
        projection_report_count: counts.projection_report_count,
        archived_entry_count: counts.archived_entry_count,
    };
    if budget_result.due_candidate_count > budget.max_due_candidates {
        return Ok(DataLayerM10Phase6BudgetPolicyReport {
            decision: DataLayerM10Phase6PolicyBudgetDecision::Exceeded,
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.shredded_message_count > budget.max_shredded_messages {
        return Ok(DataLayerM10Phase6BudgetPolicyReport {
            decision: DataLayerM10Phase6PolicyBudgetDecision::Exceeded,
            reason_code:
                DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.projection_report_count > budget.max_projection_reports {
        return Ok(DataLayerM10Phase6BudgetPolicyReport {
            decision: DataLayerM10Phase6PolicyBudgetDecision::Exceeded,
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    if budget_result.archived_entry_count > budget.max_archived_entries {
        return Ok(DataLayerM10Phase6BudgetPolicyReport {
            decision: DataLayerM10Phase6PolicyBudgetDecision::Exceeded,
            reason_code:
                DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE,
            ..budget_result
        });
    }
    Ok(budget_result)
}

/// Evaluates scheduler preflight budget using deterministic count-shaping assumptions.
///
/// Preflight assumptions:
/// - every due candidate is expected to be shredded in this cycle
/// - every projection report is expected to create one archived entry
pub fn data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy(
    due_candidate_count: usize,
    projection_report_count: usize,
    budget: DataLayerM10Phase6PolicyBudget,
) -> Result<DataLayerM10Phase6BudgetPolicyReport, DataLayerM10Phase6PolicyEvaluatorError> {
    data_layer_m10_evaluate_phase6_execution_tick_budget_policy(
        DataLayerM10Phase6PolicyReportCounts {
            due_candidate_count,
            shredded_message_count: due_candidate_count,
            projection_report_count,
            archived_entry_count: projection_report_count,
        },
        budget,
    )
}

/// Evaluates deterministic scheduler trigger decision for a phase6 tick cycle.
pub fn data_layer_m10_evaluate_phase6_scheduler_trigger_policy(
    policy: DataLayerM10Phase6SchedulerTriggerPolicy,
    signal: DataLayerM10Phase6SchedulerSignalPolicy,
) -> Result<DataLayerM10Phase6TriggerPolicyDecision, DataLayerM10Phase6PolicyEvaluatorError> {
    validate_scheduler_policy(policy)?;
    let elapsed_since_last_tick_seconds = resolve_elapsed(signal)?;
    if signal.due_candidate_count >= policy.due_candidate_trigger_threshold {
        return Ok(DataLayerM10Phase6TriggerPolicyDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
            due_candidate_count: signal.due_candidate_count,
            elapsed_since_last_tick_seconds,
        });
    }
    if elapsed_since_last_tick_seconds >= policy.max_tick_interval_seconds {
        return Ok(DataLayerM10Phase6TriggerPolicyDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_INTERVAL_ELAPSED_REASON_CODE,
            due_candidate_count: signal.due_candidate_count,
            elapsed_since_last_tick_seconds,
        });
    }
    Ok(DataLayerM10Phase6TriggerPolicyDecision::Deferred {
        reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
        due_candidate_count: signal.due_candidate_count,
        elapsed_since_last_tick_seconds,
    })
}

fn validate_budget(
    budget: DataLayerM10Phase6PolicyBudget,
) -> Result<(), DataLayerM10Phase6PolicyEvaluatorError> {
    if budget.max_due_candidates == 0 {
        return Err(DataLayerM10Phase6PolicyEvaluatorError::InvalidBudgetField {
            field: "max_due_candidates",
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
        });
    }
    if budget.max_shredded_messages == 0 {
        return Err(DataLayerM10Phase6PolicyEvaluatorError::InvalidBudgetField {
            field: "max_shredded_messages",
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
        });
    }
    if budget.max_projection_reports == 0 {
        return Err(DataLayerM10Phase6PolicyEvaluatorError::InvalidBudgetField {
            field: "max_projection_reports",
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
        });
    }
    if budget.max_archived_entries == 0 {
        return Err(DataLayerM10Phase6PolicyEvaluatorError::InvalidBudgetField {
            field: "max_archived_entries",
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
        });
    }
    Ok(())
}

fn validate_scheduler_policy(
    policy: DataLayerM10Phase6SchedulerTriggerPolicy,
) -> Result<(), DataLayerM10Phase6PolicyEvaluatorError> {
    if policy.due_candidate_trigger_threshold == 0 {
        return Err(
            DataLayerM10Phase6PolicyEvaluatorError::InvalidSchedulerPolicyField {
                field: "due_candidate_trigger_threshold",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
            },
        );
    }
    if policy.max_tick_interval_seconds == 0 {
        return Err(
            DataLayerM10Phase6PolicyEvaluatorError::InvalidSchedulerPolicyField {
                field: "max_tick_interval_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
            },
        );
    }
    Ok(())
}

fn resolve_elapsed(
    signal: DataLayerM10Phase6SchedulerSignalPolicy,
) -> Result<u64, DataLayerM10Phase6PolicyEvaluatorError> {
    if signal.now_epoch_seconds == 0 {
        return Err(
            DataLayerM10Phase6PolicyEvaluatorError::InvalidSchedulerSignalField {
                field: "now_epoch_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
            },
        );
    }
    match signal.last_tick_epoch_seconds {
        Some(last_tick_epoch_seconds) => {
            if last_tick_epoch_seconds > signal.now_epoch_seconds {
                return Err(
                    DataLayerM10Phase6PolicyEvaluatorError::InvalidSchedulerSignalField {
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
