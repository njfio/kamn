#![warn(missing_docs)]
//! Extracted data-layer shared helpers from `kamn-core`.

/// Shared SHA-256 helpers used by data-layer hash-chain contracts.
pub mod data_layer_hashing;
/// M10 archival retry projection contracts extracted from core data-layer module.
pub mod data_layer_m10_archival_retry;
/// M10 compliance-projection bookkeeping extracted from core.
pub mod data_layer_m10_compliance_projection_bookkeeping;
/// M10 compliance projection seam contracts shared by extraction adapters.
pub mod data_layer_m10_compliance_projection_port;
/// M10 partition month-id parsing and naming policy extracted from core.
pub mod data_layer_m10_partition_month_policy;
/// M10 deterministic partition registry lifecycle state machine extracted from core.
pub mod data_layer_m10_partition_registry_state_machine;
/// M10 phase-6 compliance seam contracts shared by extraction adapters.
pub mod data_layer_m10_phase6_compliance_port;
/// M10 phase-6 policy evaluator contracts extracted from core.
pub mod data_layer_m10_phase6_policy_evaluator;
/// M10 phase-6 runtime evidence projector contracts extracted from core.
pub mod data_layer_m10_phase6_runtime_evidence;
/// M11 closure-evidence acceptance policy contracts extracted from core.
pub mod data_layer_m11_closure_evidence;
/// M11 hardening matrix contracts for scenario tracking and operator readiness decisions.
pub mod data_layer_m11_hardening_readiness;
/// M1 batch scheduler trigger policy extracted from core.
pub mod data_layer_m1_batch_scheduler;
/// M7 billing projection and reconciliation contracts extracted from core telemetry policy.
pub mod data_layer_m7_billing_reconciliation;
/// M7 observability projection contracts extracted from core telemetry policy.
pub mod data_layer_m7_observability_projection;
/// PRD critical-scenario conformance contracts extracted from core.
pub mod data_layer_prd_critical_scenario_conformance;
/// Shell-neutral orchestration and ratio-budget policy contracts extracted from core.
pub mod data_layer_shell_neutral_policy;

pub use data_layer_m10_archival_retry::{
    DataLayerM10ArchivalFailureClass, DataLayerM10ArchivalRecoveryAction,
    DataLayerM10ArchivalRetryDecision, DataLayerM10ArchivalRetryError,
    DataLayerM10ArchivalRetryPolicy, DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
    data_layer_m10_project_archival_retry_decision,
};
pub use data_layer_m10_compliance_projection_bookkeeping::{
    DataLayerM10ComplianceProjectionBookkeepingError, DataLayerM10ComplianceShredProjectionReport,
    DataLayerM10ComplianceShredProjectionRequest,
    DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
    data_layer_m10_project_partition_shred_completeness_with_port,
};
pub use data_layer_m10_compliance_projection_port::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError,
};
pub use data_layer_m10_partition_month_policy::{
    DataLayerM10PartitionMonthPolicyError, DATA_LAYER_M10_PARTITION_PREFIX,
    data_layer_m10_add_months, data_layer_m10_deterministic_checksum_marker,
    data_layer_m10_format_partition_name, data_layer_m10_month_distance,
    data_layer_m10_split_month_id, data_layer_m10_validate_partition_month_id,
};
pub use data_layer_m10_partition_registry_state_machine::{
    DataLayerM10ArchivalIndexEntry, DataLayerM10ArchiveDueRequest,
    DataLayerM10PartitionRecord, DataLayerM10PartitionRecordInput,
    DataLayerM10PartitionRegistryStateMachine, DataLayerM10PartitionRegistryStateMachineError,
    DataLayerM10PartitionStatus, DataLayerM10RecoveryDecision,
    DataLayerM10RecoveryReadinessReport, DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
    DATA_LAYER_M10_ARCHIVE_REASON_CODE, DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
    DATA_LAYER_M10_REATTACH_REASON_CODE, DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
};
pub use data_layer_m10_phase6_compliance_port::{
    DataLayerM10Phase6CompliancePort, DataLayerM10Phase6CompliancePortError,
    DataLayerM10Phase6CryptoShredInput, DataLayerM10Phase6RetentionDueCandidate,
};
pub use data_layer_m10_phase6_policy_evaluator::{
    DataLayerM10Phase6BudgetPolicyReport, DataLayerM10Phase6PolicyBudget,
    DataLayerM10Phase6PolicyBudgetDecision, DataLayerM10Phase6PolicyEvaluatorError,
    DataLayerM10Phase6PolicyReportCounts, DataLayerM10Phase6SchedulerBudgetOverflowPolicyProjection,
    DataLayerM10Phase6SchedulerBudgetOverflowStage, DataLayerM10Phase6SchedulerCyclePolicyReport,
    DataLayerM10Phase6SchedulerSignalPolicy, DataLayerM10Phase6SchedulerTriggerPolicy,
    DataLayerM10Phase6TriggerPolicyDecision,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_INTERVAL_ELAPSED_REASON_CODE,
    data_layer_m10_evaluate_phase6_execution_tick_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_trigger_policy,
    data_layer_m10_project_phase6_scheduler_budget_overflow_policy_error,
    data_layer_m10_project_phase6_scheduler_cycle_policy_report,
    data_layer_m10_validate_phase6_execution_budget_policy,
    data_layer_m10_validate_phase6_scheduler_runtime_clock_signal,
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config,
};
pub use data_layer_m11_closure_evidence::{
    DataLayerM11ClosureAcceptanceDecision, DataLayerM11ClosureEvidenceError,
    DataLayerM11ClosureEvidenceInput, DataLayerM11ClosureEvidenceReport,
    DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
    data_layer_m11_evaluate_closure_evidence,
};
pub use data_layer_m11_hardening_readiness::{
    DataLayerM11HardeningMatrix, DataLayerM11HardeningMatrixError,
    DataLayerM11OperatorReadinessDecision, DataLayerM11OperatorReadinessReport,
    DataLayerM11ScenarioDefinition, DataLayerM11ScenarioDomain, DataLayerM11ScenarioOutcomeInput,
    DataLayerM11ScenarioOutcomeRecord, DataLayerM11ScenarioSeverity, DataLayerM11ScenarioStatus,
    DATA_LAYER_M11_BLOCK_CRITICAL_FAILURE_REASON_CODE,
    DATA_LAYER_M11_BLOCK_REQUIRED_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M11_INVALID_TRANSITION_REASON_CODE, DATA_LAYER_M11_READINESS_GO_REASON_CODE,
};
pub use data_layer_m1_batch_scheduler::{
    DataLayerM1BatchSchedulerError, DataLayerM1BatchSchedulerPolicy,
    DataLayerM1BatchTriggerDecision, DataLayerM1PendingBatchMessage,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_COUNT_THRESHOLD,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_DEFERRED,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_WINDOW_THRESHOLD, evaluate_data_layer_m1_batch_trigger,
};
pub use data_layer_m7_billing_reconciliation::{
    DataLayerM7BillingDailyProjection, DataLayerM7BillingProjectionSampleInput,
    DataLayerM7BillingReconciliationDecision, DataLayerM7BillingReconciliationError,
    DataLayerM7BillingReconciliationInput, DataLayerM7BillingReconciliationReport,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE,
    project_data_layer_m7_owner_billing_daily, reconcile_data_layer_m7_owner_billing_daily,
};
pub use data_layer_m7_observability_projection::{
    DataLayerM7ObservabilityProjection, DataLayerM7ObservabilityProjectionInput,
    project_data_layer_m7_observability_sample,
};
pub use data_layer_prd_critical_scenario_conformance::{
    DataLayerPrdCriticalScenarioConformanceDecision,
    DataLayerPrdCriticalScenarioConformanceError,
    DataLayerPrdCriticalScenarioConformanceMatrix, DataLayerPrdCriticalScenarioConformanceReport,
    DataLayerPrdCriticalScenarioMode, DataLayerPrdCriticalScenarioResultInput,
    DataLayerPrdCriticalScenarioResultRecord,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE,
};
pub use data_layer_shell_neutral_policy::{
    DataLayerShellNeutralPolicyDecision, DataLayerShellNeutralPolicyError,
    DataLayerShellNeutralPolicyInput, DataLayerShellNeutralPolicyReasonCode,
    DataLayerShellNeutralPolicyReasonCodeParseError, DataLayerShellNeutralPolicyReport,
    data_layer_evaluate_shell_neutral_policy,
};
