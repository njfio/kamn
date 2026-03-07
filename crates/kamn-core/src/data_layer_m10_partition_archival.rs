//! M10 partition lifecycle contracts for scaling and archival export controls.
//!
//! This module models PRD M10 behavior as deterministic Rust contracts:
//! monthly partition naming/planning, retention-window archival eligibility,
//! archival index metadata projection, and archived-partition re-attachment.

use std::collections::BTreeMap;

mod error;
mod phase6;
mod registry;
mod retry;
mod shared;

pub use error::DataLayerM10PartitionLifecycleError;
pub use phase6::{
    data_layer_m10_evaluate_phase6_execution_tick_budget,
    data_layer_m10_evaluate_phase6_scheduler_trigger,
    data_layer_m10_execute_phase6_orchestration_tick,
    data_layer_m10_execute_phase6_orchestration_tick_with_port,
    data_layer_m10_execute_phase6_scheduler_cycle,
    data_layer_m10_execute_phase6_scheduler_cycle_with_port,
    data_layer_m10_project_phase6_runtime_evidence_bundle,
};
pub use registry::data_layer_m10_format_partition_name;
pub use retry::data_layer_m10_project_archival_retry_decision;

/// Partition prefix for monthly message partitions.
pub use kamn_data_layer::{
    DataLayerM10ArchivalIndexEntry, DataLayerM10ArchiveDueRequest, DataLayerM10PartitionRecord,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus, DataLayerM10RecoveryDecision,
    DataLayerM10RecoveryReadinessReport, DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
    DATA_LAYER_M10_ARCHIVE_REASON_CODE, DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
    DATA_LAYER_M10_PARTITION_PREFIX, DATA_LAYER_M10_REATTACH_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
};
pub use kamn_data_layer::{
    DataLayerM10ComplianceShredProjectionReport, DataLayerM10ComplianceShredProjectionRequest,
    DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
};
/// Stable reason marker for owner-scope projection denials.
pub const DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE: &str =
    "m10_partition_compliance_owner_scope_denied";
pub use kamn_data_layer::{
    DataLayerM10ArchivalFailureClass, DataLayerM10ArchivalRecoveryAction,
    DataLayerM10ArchivalRetryDecision, DataLayerM10ArchivalRetryPolicy,
    DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
};
/// Stable reason marker when a Phase-6 retention/archival orchestration tick completes.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE: &str =
    "m10_phase6_execution_applied";
/// Stable reason marker when Phase-6 orchestration fails owner-scope authorization.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE: &str =
    "m10_phase6_execution_owner_scope_denied";
/// Stable reason marker when Phase-6 orchestration encounters legal-hold blocking.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE: &str =
    "m10_phase6_execution_legal_hold_active";
/// Stable reason marker when Phase-6 orchestration input is invalid.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE: &str =
    "m10_phase6_execution_input_invalid";
/// Stable reason marker when Phase-6 orchestration projection input is invalid.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE: &str =
    "m10_phase6_execution_projection_input_invalid";
/// Stable reason marker when Phase-6 orchestration projection fails.
pub const DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_FAILED_REASON_CODE: &str =
    "m10_phase6_execution_projection_failed";
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
/// Stable reason marker when Phase-6 scheduler cycle returns deferred without execution.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_deferred";
/// Stable reason marker when Phase-6 scheduler cycle executes and budget evidence is within limits.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_applied";
/// Stable reason marker when Phase-6 stateful scheduler runtime is initialized.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_RUNTIME_INITIALIZED_REASON_CODE: &str =
    "m10_phase6_scheduler_runtime_initialized";
/// Stable reason marker when Phase-6 runtime evidence projection succeeds for an applied cycle.
pub const DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE: &str =
    "m10_phase6_runtime_evidence_applied";
/// Stable reason marker when Phase-6 runtime evidence projection succeeds for a deferred cycle.
pub const DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE: &str =
    "m10_phase6_runtime_evidence_deferred";
/// Stable reason marker when Phase-6 runtime evidence projection input is invalid.
pub const DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE: &str =
    "m10_phase6_runtime_evidence_input_invalid";

/// Phase-6 orchestration request composing retention + projection + archival execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6ExecutionTickRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Current epoch timestamp used for retention-due evaluation.
    pub now_epoch_seconds: u64,
    /// Epoch timestamp applied to newly shredded messages.
    pub shredded_at_epoch_seconds: u64,
    /// Current month identifier as `YYYYMM`.
    pub now_month_id: u32,
    /// Active retention window in months.
    pub active_retention_months: u16,
    /// Object-storage prefix used for archived artifacts.
    pub object_storage_prefix: String,
    /// Partition message membership used for M8->M10 shred-completeness projection.
    pub partition_message_ids_by_month: BTreeMap<u32, Vec<String>>,
}

/// Phase-6 orchestration execution report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6ExecutionTickReport {
    /// Canonical owner scope for execution.
    pub owner_did: String,
    /// Number of retention-due candidates evaluated this tick.
    pub due_candidate_count: usize,
    /// Message ids shredded in this tick (sorted deterministic order).
    pub shredded_message_ids: Vec<String>,
    /// M10 partition shred-completeness projection reports in deterministic order.
    pub projection_reports: Vec<DataLayerM10ComplianceShredProjectionReport>,
    /// Archival entries created in this tick.
    pub archived_entries: Vec<DataLayerM10ArchivalIndexEntry>,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Deterministic Phase-6 execution tick budget limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6ExecutionTickBudget {
    /// Maximum retention-due candidate count allowed in one tick.
    pub max_due_candidates: usize,
    /// Maximum shredded message operations allowed in one tick.
    pub max_shredded_messages: usize,
    /// Maximum partition projections allowed in one tick.
    pub max_projection_reports: usize,
    /// Maximum archived entries allowed in one tick.
    pub max_archived_entries: usize,
}

/// Budget classification decision for one Phase-6 execution tick report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6ExecutionBudgetDecision {
    /// Report is within all configured limits.
    WithinBudget,
    /// One or more limits were exceeded.
    Exceeded,
}

/// Budget evaluation report for one Phase-6 execution tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6ExecutionTickBudgetReport {
    /// Evaluation decision.
    pub decision: DataLayerM10Phase6ExecutionBudgetDecision,
    /// Stable reason marker.
    pub reason_code: &'static str,
    /// Observed due-candidate count.
    pub due_candidate_count: usize,
    /// Observed shredded message count.
    pub shredded_message_count: usize,
    /// Observed projection report count.
    pub projection_report_count: usize,
    /// Observed archived-entry count.
    pub archived_entry_count: usize,
}

/// Deterministic scheduler policy for Phase-6 orchestration tick cadence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerPolicy {
    /// Trigger execution when due-candidate count reaches this threshold.
    pub due_candidate_trigger_threshold: usize,
    /// Trigger execution when elapsed time since previous tick reaches this interval.
    pub max_tick_interval_seconds: u64,
}

/// Scheduler signal projected for Phase-6 tick trigger evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerSignal {
    /// Retention due-candidate count observed at scheduler evaluation time.
    pub due_candidate_count: usize,
    /// Epoch timestamp for previous successful tick, if any.
    pub last_tick_epoch_seconds: Option<u64>,
    /// Current scheduler epoch timestamp.
    pub now_epoch_seconds: u64,
}

/// Deterministic scheduler trigger decision for one Phase-6 cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6SchedulerTriggerDecision {
    /// Trigger is deferred for this cycle.
    Deferred {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Due candidates observed in this cycle.
        due_candidate_count: usize,
        /// Elapsed seconds since previous tick.
        elapsed_since_last_tick_seconds: u64,
    },
    /// Trigger is activated for this cycle.
    Triggered {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Due candidates observed in this cycle.
        due_candidate_count: usize,
        /// Elapsed seconds since previous tick.
        elapsed_since_last_tick_seconds: u64,
    },
}

/// Scheduler-cycle request composing trigger policy, preflight budget, and execution inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerCycleRequest {
    /// Scheduler trigger policy.
    pub scheduler_policy: DataLayerM10Phase6SchedulerPolicy,
    /// Last successful tick timestamp, if any.
    pub last_tick_epoch_seconds: Option<u64>,
    /// Execution budget guardrail limits.
    pub budget: DataLayerM10Phase6ExecutionTickBudget,
    /// Orchestration tick request executed when trigger activates.
    pub execution_request: DataLayerM10Phase6ExecutionTickRequest,
}

/// Deterministic scheduler-cycle execution report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerCycleReport {
    /// Trigger decision for this cycle.
    pub trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision,
    /// Orchestration report when cycle executed.
    pub execution_report: Option<DataLayerM10Phase6ExecutionTickReport>,
    /// Budget evaluation report when cycle executed.
    pub budget_report: Option<DataLayerM10Phase6ExecutionTickBudgetReport>,
    /// Stable cycle result marker.
    pub reason_code: &'static str,
}

/// Stateful runtime snapshot for Phase-6 scheduler continuity and checkpointing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerRuntimeState {
    /// Last successful execution-tick timestamp.
    pub last_successful_tick_epoch_seconds: Option<u64>,
    /// Last observed scheduler timestamp (successful, deferred, or failed attempt).
    pub last_observed_now_epoch_seconds: Option<u64>,
    /// Total scheduler cycles attempted.
    pub total_cycles: u64,
    /// Total scheduler cycles that executed orchestration successfully.
    pub executed_cycles: u64,
    /// Total scheduler cycles deferred by trigger policy.
    pub deferred_cycles: u64,
    /// Total scheduler cycles that failed closed.
    pub fail_closed_cycles: u64,
    /// Stable reason marker from the most recent cycle outcome.
    pub last_reason_code: &'static str,
}

/// Stateful Phase-6 scheduler runtime wrapper over deterministic cycle contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6SchedulerRuntime {
    scheduler_policy: DataLayerM10Phase6SchedulerPolicy,
    budget: DataLayerM10Phase6ExecutionTickBudget,
    state: DataLayerM10Phase6SchedulerRuntimeState,
}

/// Evidence projection input combining one scheduler-cycle report with runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6RuntimeEvidenceInput {
    /// Owner DID for this evidence bundle.
    pub owner_did: String,
    /// Scheduler-cycle report for this evidence bundle.
    pub cycle_report: DataLayerM10Phase6SchedulerCycleReport,
    /// Runtime state snapshot captured after the cycle.
    pub runtime_state: DataLayerM10Phase6SchedulerRuntimeState,
}

/// Canonical evidence bundle projected from Phase-6 scheduler runtime execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6RuntimeEvidenceBundle {
    /// Owner DID for this evidence bundle.
    pub owner_did: String,
    /// Stable reason marker from scheduler-cycle outcome (`applied` or `deferred`).
    pub cycle_reason_code: &'static str,
    /// Stable trigger-decision reason marker.
    pub trigger_reason_code: &'static str,
    /// Stable budget reason marker for applied cycles.
    pub budget_reason_code: Option<&'static str>,
    /// Archived partition names sorted in deterministic order.
    pub archived_partition_names: Vec<String>,
    /// Archived object URIs sorted in deterministic order matching partition names.
    pub archived_object_uris: Vec<String>,
    /// Due-candidate count represented by this cycle evidence.
    pub due_candidate_count: usize,
    /// Shredded message count represented by this cycle evidence.
    pub shredded_message_count: usize,
    /// Projection report count represented by this cycle evidence.
    pub projection_report_count: usize,
    /// Archived-entry count represented by this cycle evidence.
    pub archived_entry_count: usize,
    /// Runtime total cycle count.
    pub runtime_total_cycles: u64,
    /// Runtime executed cycle count.
    pub runtime_executed_cycles: u64,
    /// Runtime deferred cycle count.
    pub runtime_deferred_cycles: u64,
    /// Runtime fail-closed cycle count.
    pub runtime_fail_closed_cycles: u64,
    /// Runtime last successful tick checkpoint.
    pub runtime_last_successful_tick_epoch_seconds: Option<u64>,
    /// Runtime last observed scheduler clock value.
    pub runtime_last_observed_now_epoch_seconds: Option<u64>,
    /// Runtime last outcome reason marker.
    pub runtime_last_reason_code: &'static str,
    /// Evidence projection result marker.
    pub reason_code: &'static str,
}

/// Core compatibility wrapper over the extracted deterministic registry state machine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM10PartitionLifecycleRegistry {
    state_machine: kamn_data_layer::DataLayerM10PartitionRegistryStateMachine,
}
