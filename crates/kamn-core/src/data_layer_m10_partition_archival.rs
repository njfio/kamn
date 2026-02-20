//! M10 partition lifecycle contracts for scaling and archival export controls.
//!
//! This module models PRD M10 behavior as deterministic Rust contracts:
//! monthly partition naming/planning, retention-window archival eligibility,
//! archival index metadata projection, and archived-partition re-attachment.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest,
    DataLayerM8OwnerScopeQuery, KamnDid,
};

/// Partition prefix for monthly message partitions.
pub const DATA_LAYER_M10_PARTITION_PREFIX: &str = "messages_";
/// Archive format marker for exported partition artifacts.
pub const DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD: &str = "parquet-zstd";
/// Stable reason marker for archived lifecycle transitions.
pub const DATA_LAYER_M10_ARCHIVE_REASON_CODE: &str = "m10_partition_archived";
/// Stable reason marker for archived -> reattached transitions.
pub const DATA_LAYER_M10_REATTACH_REASON_CODE: &str = "m10_partition_reattached";
/// Stable reason marker for invalid lifecycle transitions.
pub const DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE: &str = "m10_partition_transition_invalid";
/// Stable reason marker when partition recoverability is ready for historical replay.
pub const DATA_LAYER_M10_RECOVERY_READY_REASON_CODE: &str = "m10_partition_recovery_ready";
/// Stable reason marker when partition status is not eligible for historical recovery.
pub const DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE: &str =
    "m10_partition_recovery_status_ineligible";
/// Stable reason marker when historical partition metadata is incomplete.
pub const DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE: &str =
    "m10_partition_recovery_metadata_incomplete";
/// Stable reason marker when M10 partition shred-completeness projection from M8 is applied.
pub const DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE: &str =
    "m10_partition_compliance_projection_applied";
/// Stable reason marker when M8 projection resolves that partition is not fully shredded.
pub const DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE: &str =
    "m10_partition_compliance_shred_incomplete";
/// Stable reason marker when M8 projection resolves that partition is fully shredded.
pub const DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE: &str =
    "m10_partition_compliance_shred_complete";
/// Stable reason marker when M8 projection resolves that legal hold still blocks archival.
pub const DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE: &str =
    "m10_partition_compliance_legal_hold_active";
/// Stable reason marker for owner-scope projection denials.
pub const DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE: &str =
    "m10_partition_compliance_owner_scope_denied";
/// Stable reason marker when M8 lookup fails during projection.
pub const DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE: &str =
    "m10_partition_compliance_lookup_failed";
/// Stable reason marker when M8 projection input is invalid.
pub const DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE: &str =
    "m10_partition_compliance_input_invalid";
/// Stable reason marker when archival transient failure schedules a retry.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE: &str =
    "m10_archival_retry_scheduled";
/// Stable reason marker when archival retry budget is exhausted.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE: &str =
    "m10_archival_retry_exhausted";
/// Stable reason marker when archival failure is permanent and must fail closed.
pub const DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE: &str =
    "m10_archival_failure_permanent";
/// Stable reason marker when archival retry policy configuration is invalid.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE: &str =
    "m10_archival_retry_policy_invalid";
/// Stable reason marker when archival retry attempt metadata is invalid.
pub const DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE: &str =
    "m10_archival_retry_attempt_invalid";
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

/// Partition lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10PartitionStatus {
    /// Active partition in primary query path.
    Active,
    /// Archived partition with export metadata in archival index.
    Archived,
    /// Archived partition reattached for historical query access.
    Reattached,
}

/// Recoverability decision for one partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10RecoveryDecision {
    /// Partition has complete archival metadata and can be recovered.
    Ready,
    /// Partition cannot be recovered under current state/metadata.
    Blocked,
}

/// Retry classification for archival export failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10ArchivalFailureClass {
    /// Failure may succeed on a later attempt.
    Transient,
    /// Failure must fail closed immediately.
    Permanent,
}

/// Recovery action projected for an archival export failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10ArchivalRecoveryAction {
    /// Schedule one more retry attempt.
    RetryScheduled,
    /// Fail closed and stop retrying.
    FailClosed,
}

/// Bounded retry policy for archival failure recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10ArchivalRetryPolicy {
    /// Total attempts allowed, including the current attempt.
    pub max_attempts: u8,
    /// Base retry backoff in seconds.
    pub base_backoff_seconds: u64,
    /// Maximum retry backoff cap in seconds.
    pub max_backoff_seconds: u64,
}

/// Deterministic decision projected for an archival export failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10ArchivalRetryDecision {
    /// Failure classification used for this projection.
    pub failure_class: DataLayerM10ArchivalFailureClass,
    /// Recovery action.
    pub action: DataLayerM10ArchivalRecoveryAction,
    /// Current failed attempt number.
    pub current_attempt: u8,
    /// Next attempt number when a retry is scheduled.
    pub next_attempt: Option<u8>,
    /// Retry delay in seconds when a retry is scheduled.
    pub retry_backoff_seconds: Option<u64>,
    /// Retry-at timestamp in epoch seconds when a retry is scheduled.
    pub retry_after_unix_seconds: Option<u64>,
    /// Remaining attempts after this decision.
    pub attempts_remaining: u8,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Partition registration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10PartitionRecordInput {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// True when all rows in partition are shred-complete and eligible for archival export.
    pub all_messages_shredded: bool,
}

/// Partition lifecycle record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10PartitionRecord {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Shred-complete marker for archival eligibility checks.
    pub all_messages_shredded: bool,
    /// Current lifecycle status.
    pub lifecycle_status: DataLayerM10PartitionStatus,
    /// Archived object URI when partition is archived.
    pub archived_object_uri: Option<String>,
    /// Archive format marker when partition is archived.
    pub archive_format_marker: Option<&'static str>,
    /// Deterministic checksum marker when partition is archived.
    pub checksum_marker: Option<String>,
    /// Last lifecycle transition reason marker.
    pub last_reason_code: Option<&'static str>,
}

/// Archive evaluation request envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ArchiveDueRequest {
    /// Current month identifier as `YYYYMM`.
    pub now_month_id: u32,
    /// Active retention window in months.
    pub active_retention_months: u16,
    /// Object-storage prefix used for archived artifacts.
    pub object_storage_prefix: String,
}

/// Compliance projection request to derive partition shred completeness from M8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ComplianceShredProjectionRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Message identifiers that belong to the partition scope.
    pub partition_message_ids: Vec<String>,
}

/// Archival index projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ArchivalIndexEntry {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Archived object URI.
    pub archived_object_uri: String,
    /// Archive format marker.
    pub archive_format_marker: &'static str,
    /// Deterministic checksum marker.
    pub checksum_marker: String,
    /// Lifecycle status after archive transition.
    pub lifecycle_status: DataLayerM10PartitionStatus,
}

/// Projection report for M8-derived partition shred completeness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ComplianceShredProjectionReport {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Total message identifiers evaluated from projection input.
    pub total_partition_messages: usize,
    /// Number of messages currently shredded in M8.
    pub shredded_partition_messages: usize,
    /// Derived partition completeness marker.
    pub all_messages_shredded: bool,
    /// Completeness reason marker (`complete` or `incomplete`).
    pub reason_code: &'static str,
    /// Stable projection-applied marker.
    pub projection_reason_code: &'static str,
}

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

/// Recoverability readiness projection for one partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10RecoveryReadinessReport {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Recoverability decision.
    pub decision: DataLayerM10RecoveryDecision,
    /// Stable reason marker for the decision.
    pub reason_code: &'static str,
    /// Current lifecycle status.
    pub lifecycle_status: DataLayerM10PartitionStatus,
    /// Archived object URI.
    pub archived_object_uri: Option<String>,
    /// Archive format marker.
    pub archive_format_marker: Option<&'static str>,
    /// Deterministic checksum marker.
    pub checksum_marker: Option<String>,
}

/// Projects deterministic archival failure recovery decision under bounded retry policy.
pub fn data_layer_m10_project_archival_retry_decision(
    now_unix_seconds: u64,
    current_attempt: u8,
    failure_class: DataLayerM10ArchivalFailureClass,
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<DataLayerM10ArchivalRetryDecision, DataLayerM10PartitionLifecycleError> {
    validate_archival_retry_policy(policy)?;
    if current_attempt == 0 {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryAttempt {
            field: "current_attempt",
            value: current_attempt,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
        });
    }

    match failure_class {
        DataLayerM10ArchivalFailureClass::Transient if current_attempt < policy.max_attempts => {
            let exponent = u32::from(current_attempt.saturating_sub(1)).min(20);
            let multiplier = 1_u64 << exponent;
            let retry_backoff_seconds = policy
                .base_backoff_seconds
                .saturating_mul(multiplier)
                .min(policy.max_backoff_seconds);
            let retry_after_unix_seconds = now_unix_seconds.saturating_add(retry_backoff_seconds);
            let attempts_remaining = policy.max_attempts.saturating_sub(current_attempt);
            Ok(DataLayerM10ArchivalRetryDecision {
                failure_class,
                action: DataLayerM10ArchivalRecoveryAction::RetryScheduled,
                current_attempt,
                next_attempt: Some(current_attempt.saturating_add(1)),
                retry_backoff_seconds: Some(retry_backoff_seconds),
                retry_after_unix_seconds: Some(retry_after_unix_seconds),
                attempts_remaining,
                reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
            })
        }
        DataLayerM10ArchivalFailureClass::Transient => Ok(DataLayerM10ArchivalRetryDecision {
            failure_class,
            action: DataLayerM10ArchivalRecoveryAction::FailClosed,
            current_attempt,
            next_attempt: None,
            retry_backoff_seconds: None,
            retry_after_unix_seconds: None,
            attempts_remaining: 0,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
        }),
        DataLayerM10ArchivalFailureClass::Permanent => Ok(DataLayerM10ArchivalRetryDecision {
            failure_class,
            action: DataLayerM10ArchivalRecoveryAction::FailClosed,
            current_attempt,
            next_attempt: None,
            retry_backoff_seconds: None,
            retry_after_unix_seconds: None,
            attempts_remaining: 0,
            reason_code: DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
        }),
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

/// M10 partition lifecycle registry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM10PartitionLifecycleRegistry {
    partitions: BTreeMap<u32, DataLayerM10PartitionRecord>,
}

impl DataLayerM10PartitionLifecycleRegistry {
    /// Creates an empty partition lifecycle registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one monthly partition lifecycle record.
    pub fn register_partition(
        &mut self,
        input: DataLayerM10PartitionRecordInput,
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionLifecycleError> {
        validate_partition_month_id(input.partition_month_id)?;
        if self.partitions.contains_key(&input.partition_month_id) {
            return Err(
                DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(
                    input.partition_month_id,
                ),
            );
        }

        let record = DataLayerM10PartitionRecord {
            partition_month_id: input.partition_month_id,
            partition_name: data_layer_m10_format_partition_name(input.partition_month_id)?,
            all_messages_shredded: input.all_messages_shredded,
            lifecycle_status: DataLayerM10PartitionStatus::Active,
            archived_object_uri: None,
            archive_format_marker: None,
            checksum_marker: None,
            last_reason_code: None,
        };
        self.partitions
            .insert(input.partition_month_id, record.clone());
        Ok(record)
    }

    /// Derives partition shred completeness from M8 lifecycle records and updates partition state.
    pub fn project_partition_shred_completeness_from_m8(
        &mut self,
        compliance_registry: &DataLayerM8ComplianceRegistry,
        request: DataLayerM10ComplianceShredProjectionRequest,
    ) -> Result<DataLayerM10ComplianceShredProjectionReport, DataLayerM10PartitionLifecycleError>
    {
        let owner_did = authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_partition_month_id(request.partition_month_id)?;
        if request.partition_message_ids.is_empty() {
            return Err(DataLayerM10PartitionLifecycleError::EmptyField(
                "partition_message_ids",
            ));
        }

        let mut message_ids = BTreeSet::new();
        for message_id in request.partition_message_ids {
            validate_non_empty(message_id.as_str(), "partition_message_ids")?;
            message_ids.insert(message_id);
        }

        let total_partition_messages = message_ids.len();
        let mut shredded_partition_messages = 0usize;
        let mut legal_hold_active_messages = 0usize;
        for message_id in &message_ids {
            let message = compliance_registry
                .message_for_owner(owner_did.as_str(), message_id.as_str())
                .map_err(map_m8_projection_error_to_m10)?;
            if message.legal_hold_active {
                legal_hold_active_messages += 1;
            }
            if message.shredded_at_epoch_seconds.is_some() {
                shredded_partition_messages += 1;
            }
        }
        let all_messages_shredded = shredded_partition_messages == total_partition_messages;
        let reason_code = if legal_hold_active_messages > 0 {
            DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
        } else if all_messages_shredded {
            DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
        } else {
            DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
        };

        let partition_name = data_layer_m10_format_partition_name(request.partition_month_id)?;
        let record = self
            .partitions
            .get_mut(&request.partition_month_id)
            .ok_or_else(|| {
                DataLayerM10PartitionLifecycleError::PartitionNotFound(partition_name.clone())
            })?;
        record.all_messages_shredded = all_messages_shredded;
        record.last_reason_code = Some(DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE);

        Ok(DataLayerM10ComplianceShredProjectionReport {
            partition_month_id: record.partition_month_id,
            partition_name: record.partition_name.clone(),
            total_partition_messages,
            shredded_partition_messages,
            all_messages_shredded,
            reason_code,
            projection_reason_code: DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
        })
    }

    /// Plans future partition names for `months_ahead` months after `reference_month_id`.
    pub fn plan_future_partition_names(
        &self,
        reference_month_id: u32,
        months_ahead: u8,
    ) -> Result<Vec<String>, DataLayerM10PartitionLifecycleError> {
        validate_partition_month_id(reference_month_id)?;
        let mut result = Vec::with_capacity(months_ahead as usize);
        for offset in 1..=u32::from(months_ahead) {
            let month_id = add_months(reference_month_id, offset)?;
            result.push(data_layer_m10_format_partition_name(month_id)?);
        }
        Ok(result)
    }

    /// Archives all due partitions and returns archival-index projections.
    pub fn archive_due_partitions(
        &mut self,
        request: DataLayerM10ArchiveDueRequest,
    ) -> Result<Vec<DataLayerM10ArchivalIndexEntry>, DataLayerM10PartitionLifecycleError> {
        validate_partition_month_id(request.now_month_id)?;
        validate_non_empty(
            request.object_storage_prefix.as_str(),
            "object_storage_prefix",
        )?;

        let mut entries = Vec::new();
        for record in self.partitions.values_mut() {
            if record.lifecycle_status != DataLayerM10PartitionStatus::Active {
                continue;
            }
            if !record.all_messages_shredded {
                continue;
            }
            if record.partition_month_id > request.now_month_id {
                continue;
            }

            let age_months = month_distance(record.partition_month_id, request.now_month_id)?;
            if age_months <= u32::from(request.active_retention_months) {
                continue;
            }

            let archived_object_uri = format!(
                "{}/{}.parquet.zst",
                request.object_storage_prefix.trim_end_matches('/'),
                record.partition_name
            );
            let checksum_marker = deterministic_checksum_marker(
                record.partition_name.as_str(),
                record.partition_month_id,
            );

            record.lifecycle_status = DataLayerM10PartitionStatus::Archived;
            record.archived_object_uri = Some(archived_object_uri.clone());
            record.archive_format_marker = Some(DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD);
            record.checksum_marker = Some(checksum_marker.clone());
            record.last_reason_code = Some(DATA_LAYER_M10_ARCHIVE_REASON_CODE);

            entries.push(DataLayerM10ArchivalIndexEntry {
                partition_month_id: record.partition_month_id,
                partition_name: record.partition_name.clone(),
                archived_object_uri,
                archive_format_marker: DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
                checksum_marker,
                lifecycle_status: record.lifecycle_status,
            });
        }

        entries.sort_by(|left, right| {
            left.partition_month_id
                .cmp(&right.partition_month_id)
                .then(left.partition_name.cmp(&right.partition_name))
        });
        Ok(entries)
    }

    /// Re-attaches one archived partition for historical query access.
    pub fn reattach_partition(
        &mut self,
        partition_name: &str,
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionLifecycleError> {
        validate_non_empty(partition_name, "partition_name")?;
        let record = self
            .partitions
            .values_mut()
            .find(|entry| entry.partition_name == partition_name)
            .ok_or_else(|| {
                DataLayerM10PartitionLifecycleError::PartitionNotFound(partition_name.to_owned())
            })?;

        if record.lifecycle_status != DataLayerM10PartitionStatus::Archived {
            return Err(
                DataLayerM10PartitionLifecycleError::InvalidLifecycleTransition {
                    partition_name: record.partition_name.clone(),
                    from_status: record.lifecycle_status,
                    to_status: DataLayerM10PartitionStatus::Reattached,
                    reason_code: DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
                },
            );
        }

        record.lifecycle_status = DataLayerM10PartitionStatus::Reattached;
        record.last_reason_code = Some(DATA_LAYER_M10_REATTACH_REASON_CODE);
        Ok(record.clone())
    }

    /// Evaluates recoverability readiness for one partition.
    pub fn evaluate_partition_recovery_readiness(
        &self,
        partition_name: &str,
    ) -> Result<DataLayerM10RecoveryReadinessReport, DataLayerM10PartitionLifecycleError> {
        validate_non_empty(partition_name, "partition_name")?;
        let record = self
            .partitions
            .values()
            .find(|entry| entry.partition_name == partition_name)
            .ok_or_else(|| {
                DataLayerM10PartitionLifecycleError::PartitionNotFound(partition_name.to_owned())
            })?;
        Ok(project_partition_recovery_readiness(record))
    }

    /// Lists recoverability readiness for historical partitions in deterministic order.
    pub fn list_historical_recovery_readiness(&self) -> Vec<DataLayerM10RecoveryReadinessReport> {
        let mut reports: Vec<DataLayerM10RecoveryReadinessReport> = self
            .partitions
            .values()
            .filter(|record| record.lifecycle_status != DataLayerM10PartitionStatus::Active)
            .map(project_partition_recovery_readiness)
            .collect();
        reports.sort_by(|left, right| {
            left.partition_month_id
                .cmp(&right.partition_month_id)
                .then(left.partition_name.cmp(&right.partition_name))
        });
        reports
    }
}

/// Formats partition month id (`YYYYMM`) as `messages_YYYY_MM`.
pub fn data_layer_m10_format_partition_name(
    partition_month_id: u32,
) -> Result<String, DataLayerM10PartitionLifecycleError> {
    let (year, month) = split_month_id(partition_month_id)?;
    Ok(format!(
        "{DATA_LAYER_M10_PARTITION_PREFIX}{year:04}_{month:02}"
    ))
}

/// Error taxonomy for M10 partition lifecycle contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10PartitionLifecycleError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Partition month id failed `YYYYMM` validation.
    InvalidPartitionMonthId(u32),
    /// Duplicate partition month id registration.
    DuplicatePartitionMonthId(u32),
    /// Named partition does not exist in registry.
    PartitionNotFound(String),
    /// Owner-scope projection request was denied.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Compliance projection failed before partition update could be applied.
    ComplianceProjectionFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from compliance lookup/projection step.
        detail: String,
    },
    /// Lifecycle transition was not allowed from current state.
    InvalidLifecycleTransition {
        /// Partition name.
        partition_name: String,
        /// Current lifecycle status.
        from_status: DataLayerM10PartitionStatus,
        /// Requested lifecycle status.
        to_status: DataLayerM10PartitionStatus,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Archival retry policy configuration is invalid.
    InvalidRetryPolicy {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Current retry attempt metadata is invalid.
    InvalidRetryAttempt {
        /// Invalid field.
        field: &'static str,
        /// Invalid value.
        value: u8,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 orchestration failed before completing execution.
    Phase6ExecutionFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail marker for diagnostics.
        detail: String,
    },
    /// Phase-6 execution budget configuration is invalid.
    InvalidPhase6ExecutionBudget {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 scheduler policy configuration is invalid.
    InvalidPhase6SchedulerPolicy {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 scheduler signal metadata is invalid.
    InvalidPhase6SchedulerSignal {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 scheduler preflight budget check exceeded limits and failed closed.
    Phase6SchedulerBudgetPreflightExceeded {
        /// Stable reason marker describing exceeded dimension.
        reason_code: &'static str,
        /// Stable detail marker for diagnostics.
        detail: String,
    },
    /// Phase-6 runtime evidence input payload is invalid.
    InvalidPhase6RuntimeEvidenceInput {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM10PartitionLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidPartitionMonthId(value) => {
                write!(f, "invalid partition month id: {value}")
            }
            Self::DuplicatePartitionMonthId(value) => {
                write!(f, "duplicate partition month id: {value}")
            }
            Self::PartitionNotFound(value) => write!(f, "partition not found: {value}"),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::ComplianceProjectionFailed {
                reason_code,
                detail,
            } => {
                write!(f, "compliance projection failed: {reason_code} ({detail})")
            }
            Self::InvalidLifecycleTransition {
                partition_name,
                from_status,
                to_status,
                reason_code,
            } => write!(
                f,
                "invalid lifecycle transition for {partition_name}: {from_status:?} -> {to_status:?} ({reason_code})"
            ),
            Self::InvalidRetryPolicy { field, reason_code } => {
                write!(f, "invalid archival retry policy field {field} ({reason_code})")
            }
            Self::InvalidRetryAttempt {
                field,
                value,
                reason_code,
            } => write!(
                f,
                "invalid archival retry attempt for {field}: {value} ({reason_code})"
            ),
            Self::Phase6ExecutionFailed {
                reason_code,
                detail,
            } => {
                write!(f, "phase6 execution failed: {reason_code} ({detail})")
            }
            Self::InvalidPhase6ExecutionBudget { field, reason_code } => write!(
                f,
                "invalid phase6 execution budget field {field} ({reason_code})"
            ),
            Self::InvalidPhase6SchedulerPolicy { field, reason_code } => write!(
                f,
                "invalid phase6 scheduler policy field {field} ({reason_code})"
            ),
            Self::InvalidPhase6SchedulerSignal { field, reason_code } => write!(
                f,
                "invalid phase6 scheduler signal field {field} ({reason_code})"
            ),
            Self::Phase6SchedulerBudgetPreflightExceeded { reason_code, detail } => {
                write!(
                    f,
                    "phase6 scheduler budget preflight exceeded: {reason_code} ({detail})"
                )
            }
            Self::InvalidPhase6RuntimeEvidenceInput { field, reason_code } => write!(
                f,
                "invalid phase6 runtime evidence input field {field} ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerM10PartitionLifecycleError {}

fn map_m8_projection_error_to_m10(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10PartitionLifecycleError {
    let reason_code = match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. } => {
            DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE
        }
        DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::LegalHoldActive { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE
        }
    };

    DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
        reason_code,
        detail: error.to_string(),
    }
}

fn map_m8_execution_error_to_m10(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10PartitionLifecycleError {
    let reason_code = match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE
        }
        DataLayerM8ComplianceError::LegalHoldActive { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. }
        | DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE
        }
    };
    phase6_execution_failed(reason_code, error.to_string())
}

fn map_phase6_projection_error_to_m10(
    error: DataLayerM10PartitionLifecycleError,
) -> DataLayerM10PartitionLifecycleError {
    let reason_code = match &error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE
        }
        DataLayerM10PartitionLifecycleError::EmptyField(field)
            if *field == "partition_message_ids" || *field == "object_storage_prefix" =>
        {
            DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE
        }
        _ => DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_FAILED_REASON_CODE,
    };
    phase6_execution_failed(reason_code, error.to_string())
}

fn map_phase6_owner_scope_error_to_m10(
    error: DataLayerM10PartitionLifecycleError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { .. } => phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE,
            "phase6 owner scope authorization failed",
        ),
        other => other,
    }
}

fn phase6_execution_failed(
    reason_code: &'static str,
    detail: impl Into<String>,
) -> DataLayerM10PartitionLifecycleError {
    DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed {
        reason_code,
        detail: detail.into(),
    }
}

fn phase6_trigger_reason_code(
    trigger_decision: &DataLayerM10Phase6SchedulerTriggerDecision,
) -> &'static str {
    match trigger_decision {
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred { reason_code, .. }
        | DataLayerM10Phase6SchedulerTriggerDecision::Triggered { reason_code, .. } => reason_code,
    }
}

fn parse_kamn_did(value: &str) -> Result<KamnDid, DataLayerM10PartitionLifecycleError> {
    KamnDid::parse(value).map_err(|_| {
        DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
            reason_code: DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
            detail: format!("invalid did: {value}"),
        }
    })
}

fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<KamnDid, DataLayerM10PartitionLifecycleError> {
    let requester_owner_did = parse_kamn_did(requester_owner_did)?;
    let owner_did = parse_kamn_did(owner_did)?;
    if requester_owner_did.as_str() != owner_did.as_str() {
        return Err(DataLayerM10PartitionLifecycleError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
        });
    }
    Ok(owner_did)
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if value.trim().is_empty() {
        return Err(DataLayerM10PartitionLifecycleError::EmptyField(field));
    }
    Ok(())
}

fn validate_partition_month_id(
    partition_month_id: u32,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    let _ = split_month_id(partition_month_id)?;
    Ok(())
}

fn split_month_id(
    partition_month_id: u32,
) -> Result<(u32, u32), DataLayerM10PartitionLifecycleError> {
    let year = partition_month_id / 100;
    let month = partition_month_id % 100;
    if year < 1970 || !(1..=12).contains(&month) {
        return Err(
            DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(partition_month_id),
        );
    }
    Ok((year, month))
}

fn add_months(
    partition_month_id: u32,
    months_to_add: u32,
) -> Result<u32, DataLayerM10PartitionLifecycleError> {
    let (year, month) = split_month_id(partition_month_id)?;
    let base = year * 12 + (month - 1);
    let future = base + months_to_add;
    let future_year = future / 12;
    let future_month = (future % 12) + 1;
    Ok(future_year * 100 + future_month)
}

fn month_distance(
    older_partition_month_id: u32,
    newer_partition_month_id: u32,
) -> Result<u32, DataLayerM10PartitionLifecycleError> {
    let (older_year, older_month) = split_month_id(older_partition_month_id)?;
    let (newer_year, newer_month) = split_month_id(newer_partition_month_id)?;
    let older = older_year * 12 + (older_month - 1);
    let newer = newer_year * 12 + (newer_month - 1);
    Ok(newer.saturating_sub(older))
}

fn deterministic_checksum_marker(partition_name: &str, partition_month_id: u32) -> String {
    format!("sha256:{partition_name}:{partition_month_id}")
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

fn validate_archival_retry_policy(
    policy: DataLayerM10ArchivalRetryPolicy,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if policy.max_attempts == 0 {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "max_attempts",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    if policy.base_backoff_seconds == 0 {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "base_backoff_seconds",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    if policy.max_backoff_seconds == 0 || policy.max_backoff_seconds < policy.base_backoff_seconds {
        return Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "max_backoff_seconds",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        });
    }
    Ok(())
}

fn project_partition_recovery_readiness(
    record: &DataLayerM10PartitionRecord,
) -> DataLayerM10RecoveryReadinessReport {
    let metadata_complete = record
        .archived_object_uri
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        && record.archive_format_marker == Some(DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD)
        && record
            .checksum_marker
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());

    let (decision, reason_code) = match record.lifecycle_status {
        DataLayerM10PartitionStatus::Active => (
            DataLayerM10RecoveryDecision::Blocked,
            DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
        ),
        DataLayerM10PartitionStatus::Archived | DataLayerM10PartitionStatus::Reattached => {
            if metadata_complete {
                (
                    DataLayerM10RecoveryDecision::Ready,
                    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
                )
            } else {
                (
                    DataLayerM10RecoveryDecision::Blocked,
                    DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
                )
            }
        }
    };

    DataLayerM10RecoveryReadinessReport {
        partition_month_id: record.partition_month_id,
        partition_name: record.partition_name.clone(),
        decision,
        reason_code,
        lifecycle_status: record.lifecycle_status,
        archived_object_uri: record.archived_object_uri.clone(),
        archive_format_marker: record.archive_format_marker,
        checksum_marker: record.checksum_marker.clone(),
    }
}
