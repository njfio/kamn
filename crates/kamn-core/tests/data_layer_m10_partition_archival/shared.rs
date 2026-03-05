pub(super) use std::collections::BTreeMap;

pub(super) use kamn_core::{
    data_layer_m10_evaluate_phase6_execution_tick_budget,
    data_layer_m10_evaluate_phase6_scheduler_trigger,
    data_layer_m10_execute_phase6_orchestration_tick,
    data_layer_m10_execute_phase6_orchestration_tick_with_port,
    data_layer_m10_execute_phase6_scheduler_cycle, data_layer_m10_format_partition_name,
    data_layer_m10_project_archival_retry_decision,
    data_layer_m10_project_phase6_runtime_evidence_bundle, DataLayerM10ArchivalFailureClass,
    DataLayerM10ArchivalRetryDecision, DataLayerM10ArchivalIndexEntry,
    DataLayerM10ArchivalRecoveryAction, DataLayerM10ArchivalRetryPolicy,
    DataLayerM10ArchiveDueRequest, DataLayerM10ComplianceShredProjectionReport,
    DataLayerM10ComplianceShredProjectionRequest, DataLayerM10PartitionLifecycleError,
    DataLayerM10PartitionLifecycleRegistry, DataLayerM10PartitionRecordInput,
    DataLayerM10PartitionStatus, DataLayerM10Phase6ExecutionBudgetDecision,
    DataLayerM10Phase6ExecutionTickBudget, DataLayerM10Phase6ExecutionTickBudgetReport,
    DataLayerM10Phase6ExecutionTickReport, DataLayerM10Phase6ExecutionTickRequest,
    DataLayerM10Phase6RuntimeEvidenceBundle, DataLayerM10Phase6RuntimeEvidenceInput,
    DataLayerM10Phase6SchedulerCycleReport, DataLayerM10Phase6SchedulerCycleRequest,
    DataLayerM10Phase6SchedulerPolicy, DataLayerM10Phase6SchedulerRuntime,
    DataLayerM10Phase6SchedulerRuntimeState, DataLayerM10Phase6SchedulerSignal,
    DataLayerM10Phase6SchedulerTriggerDecision, DataLayerM8ComplianceRegistry,
    DataLayerM8CryptoShredRequest, DataLayerM8LegalHoldRequest, DataLayerM8MessageRecordInput,
    DataLayerM8RetentionClass, DataLayerM8WrappedCekInput,
    DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
    DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
    DATA_LAYER_M10_PARTITION_PREFIX, DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_RUNTIME_INITIALIZED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_INTERVAL_ELAPSED_REASON_CODE,
    DATA_LAYER_M10_REATTACH_REASON_CODE,
};
pub(super) use kamn_data_layer::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10Phase6CompliancePort,
    DataLayerM10Phase6CompliancePortError, DataLayerM10Phase6CryptoShredInput,
    DataLayerM10Phase6RetentionDueCandidate,
};

const PHASE6_NOW_EPOCH_SECONDS: u64 = 1_700_000_000;
const PHASE6_SHREDDED_AT_EPOCH_SECONDS: u64 = 1_700_000_300;
const PHASE6_NOW_MONTH_ID: u32 = 202602;
const PHASE6_OBJECT_STORAGE_PREFIX: &str = "s3://kamn-archive/messages";
const PHASE6_LAST_OBSERVED_NOW_EPOCH_SECONDS: u64 = 1_700_000_010;

pub(super) fn partition_input(
    partition_month_id: u32,
    all_messages_shredded: bool,
) -> DataLayerM10PartitionRecordInput {
    DataLayerM10PartitionRecordInput {
        partition_month_id,
        all_messages_shredded,
    }
}

pub(super) fn m8_message_input(
    owner_did: &str,
    message_id: &str,
    created_at_epoch_seconds: u64,
) -> DataLayerM8MessageRecordInput {
    DataLayerM8MessageRecordInput {
        owner_did: owner_did.to_owned(),
        message_id: message_id.to_owned(),
        created_at_epoch_seconds,
        content_hash: format!("hash:{message_id}"),
        hash_chain_prev: format!("prev:{message_id}"),
        retention_class: DataLayerM8RetentionClass::Standard,
        retention_extension_seconds: 0,
        wrapped_keys: vec![DataLayerM8WrappedCekInput {
            recipient_did: "kamn:did:agent:alpha-recipient".to_owned(),
            wrapped_cek: format!("cek:{message_id}"),
        }],
    }
}

pub(super) fn project_request(
    owner_did: &str,
    partition_month_id: u32,
    partition_message_ids: Vec<&str>,
) -> DataLayerM10ComplianceShredProjectionRequest {
    DataLayerM10ComplianceShredProjectionRequest {
        requester_owner_did: owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        partition_month_id,
        partition_message_ids: partition_message_ids
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }
}

pub(super) fn phase6_request(
    owner_did: &str,
    partition_message_ids_by_month: BTreeMap<u32, Vec<String>>,
) -> DataLayerM10Phase6ExecutionTickRequest {
    DataLayerM10Phase6ExecutionTickRequest {
        requester_owner_did: owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        now_epoch_seconds: PHASE6_NOW_EPOCH_SECONDS,
        shredded_at_epoch_seconds: PHASE6_SHREDDED_AT_EPOCH_SECONDS,
        now_month_id: PHASE6_NOW_MONTH_ID,
        active_retention_months: 2,
        object_storage_prefix: PHASE6_OBJECT_STORAGE_PREFIX.to_owned(),
        partition_message_ids_by_month,
    }
}

pub(super) fn phase6_budget(
    max_due_candidates: usize,
    max_shredded_messages: usize,
    max_projection_reports: usize,
    max_archived_entries: usize,
) -> DataLayerM10Phase6ExecutionTickBudget {
    DataLayerM10Phase6ExecutionTickBudget {
        max_due_candidates,
        max_shredded_messages,
        max_projection_reports,
        max_archived_entries,
    }
}

pub(super) fn phase6_scheduler_policy(
    due_candidate_trigger_threshold: usize,
    max_tick_interval_seconds: u64,
) -> DataLayerM10Phase6SchedulerPolicy {
    DataLayerM10Phase6SchedulerPolicy {
        due_candidate_trigger_threshold,
        max_tick_interval_seconds,
    }
}

pub(super) fn phase6_runtime_state(
    total_cycles: u64,
    executed_cycles: u64,
    deferred_cycles: u64,
    fail_closed_cycles: u64,
    last_reason_code: &'static str,
) -> DataLayerM10Phase6SchedulerRuntimeState {
    DataLayerM10Phase6SchedulerRuntimeState {
        last_successful_tick_epoch_seconds: Some(PHASE6_NOW_EPOCH_SECONDS),
        last_observed_now_epoch_seconds: Some(PHASE6_LAST_OBSERVED_NOW_EPOCH_SECONDS),
        total_cycles,
        executed_cycles,
        deferred_cycles,
        fail_closed_cycles,
        last_reason_code,
    }
}
