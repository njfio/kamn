/// Stable reason marker when Phase-6 scheduler cycle is deferred.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_deferred";
/// Stable reason marker when Phase-6 scheduler cycle is applied.
pub const DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE: &str =
    "m10_phase6_scheduler_cycle_applied";
/// Stable reason marker when runtime evidence projection succeeds for an applied cycle.
pub const DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE: &str =
    "m10_phase6_runtime_evidence_applied";
/// Stable reason marker when runtime evidence projection succeeds for a deferred cycle.
pub const DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE: &str =
    "m10_phase6_runtime_evidence_deferred";
/// Stable reason marker when runtime evidence projection input is invalid.
pub const DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE: &str =
    "m10_phase6_runtime_evidence_input_invalid";

/// Fail-closed error taxonomy for phase6 runtime evidence projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6PolicyRuntimeEvidenceError {
    /// Runtime evidence input is invalid.
    InvalidInput {
        /// Invalid field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

/// Scheduler trigger decision used by runtime-evidence projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6PolicySchedulerTriggerDecision {
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

/// Archived-entry subset needed for runtime evidence projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyArchivedEntry {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Archived object URI.
    pub archived_object_uri: String,
}

/// Execution report subset needed for runtime evidence projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyExecutionReport {
    /// Canonical owner scope for execution.
    pub owner_did: String,
    /// Number of retention-due candidates evaluated this tick.
    pub due_candidate_count: usize,
    /// Number of message ids shredded in this tick.
    pub shredded_message_count: usize,
    /// Number of projection reports produced in this tick.
    pub projection_report_count: usize,
    /// Archived entries produced in this tick.
    pub archived_entries: Vec<DataLayerM10Phase6PolicyArchivedEntry>,
}

/// Budget decision subset needed for runtime evidence projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6PolicyBudgetDecision {
    /// Execution completed within configured budget.
    WithinBudget {
        /// Stable budget reason marker.
        reason_code: &'static str,
    },
    /// Execution exceeded configured budget.
    Exceeded {
        /// Stable budget reason marker.
        reason_code: &'static str,
    },
}

/// Scheduler-cycle reason classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10Phase6PolicySchedulerCycleReason {
    /// Cycle executed and produced execution artifacts.
    Applied,
    /// Cycle was deferred and produced no execution artifacts.
    Deferred,
}

/// Scheduler-cycle report subset needed for runtime evidence projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyCycleReport {
    /// Trigger decision for this cycle.
    pub trigger_decision: DataLayerM10Phase6PolicySchedulerTriggerDecision,
    /// Execution report when the cycle executed.
    pub execution_report: Option<DataLayerM10Phase6PolicyExecutionReport>,
    /// Budget decision when the cycle executed.
    pub budget_decision: Option<DataLayerM10Phase6PolicyBudgetDecision>,
    /// Scheduler-cycle reason classification.
    pub reason_code: DataLayerM10Phase6PolicySchedulerCycleReason,
}

/// Runtime state snapshot needed for runtime evidence projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyRuntimeState {
    /// Last successful execution-tick timestamp.
    pub last_successful_tick_epoch_seconds: Option<u64>,
    /// Last observed scheduler timestamp.
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

/// Input contract for runtime evidence projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyRuntimeEvidenceInput {
    /// Owner DID for this evidence bundle.
    pub owner_did: String,
    /// Scheduler-cycle report for this evidence bundle.
    pub cycle_report: DataLayerM10Phase6PolicyCycleReport,
    /// Runtime state snapshot captured after the cycle.
    pub runtime_state: DataLayerM10Phase6PolicyRuntimeState,
}

/// Canonical runtime evidence bundle projected from phase6 scheduler execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10Phase6PolicyRuntimeEvidenceBundle {
    /// Owner DID for this evidence bundle.
    pub owner_did: String,
    /// Scheduler-cycle reason marker (`applied` or `deferred`).
    pub cycle_reason_code: &'static str,
    /// Stable trigger-decision reason marker.
    pub trigger_reason_code: &'static str,
    /// Budget reason marker for applied cycles.
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
