use kamn_core::{
    data_layer_m10_evaluate_phase6_execution_tick_budget,
    data_layer_m10_evaluate_phase6_scheduler_trigger,
    data_layer_m10_execute_phase6_orchestration_tick,
    data_layer_m10_execute_phase6_orchestration_tick_with_port,
    data_layer_m10_execute_phase6_scheduler_cycle, data_layer_m10_format_partition_name,
    data_layer_m10_project_archival_retry_decision,
    data_layer_m10_project_phase6_runtime_evidence_bundle, DataLayerM10ArchivalFailureClass,
    DataLayerM10ArchivalRetryDecision,
    DataLayerM10ArchivalIndexEntry, DataLayerM10ArchivalRecoveryAction,
    DataLayerM10ArchivalRetryPolicy, DataLayerM10ArchiveDueRequest,
    DataLayerM10ComplianceShredProjectionReport, DataLayerM10ComplianceShredProjectionRequest,
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus,
    DataLayerM10Phase6ExecutionBudgetDecision, DataLayerM10Phase6ExecutionTickBudget,
    DataLayerM10Phase6ExecutionTickBudgetReport, DataLayerM10Phase6ExecutionTickReport,
    DataLayerM10Phase6ExecutionTickRequest, DataLayerM10Phase6RuntimeEvidenceBundle,
    DataLayerM10Phase6RuntimeEvidenceInput, DataLayerM10Phase6SchedulerCycleReport,
    DataLayerM10Phase6SchedulerCycleRequest, DataLayerM10Phase6SchedulerPolicy,
    DataLayerM10Phase6SchedulerRuntime, DataLayerM10Phase6SchedulerRuntimeState,
    DataLayerM10Phase6SchedulerSignal, DataLayerM10Phase6SchedulerTriggerDecision,
    DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest, DataLayerM8LegalHoldRequest,
    DataLayerM8MessageRecordInput, DataLayerM8RetentionClass, DataLayerM8WrappedCekInput,
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
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE, DATA_LAYER_M10_PARTITION_PREFIX,
    DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
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
use kamn_data_layer::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10Phase6CompliancePort,
    DataLayerM10Phase6CompliancePortError, DataLayerM10Phase6CryptoShredInput,
    DataLayerM10Phase6RetentionDueCandidate,
};
use std::collections::BTreeMap;

#[path = "data_layer_m10_partition_archival/execution_budget_cases.rs"]
mod execution_budget_cases;
#[path = "data_layer_m10_partition_archival/lifecycle_basics_cases.rs"]
mod lifecycle_basics_cases;
#[path = "data_layer_m10_partition_archival/orchestration_ordering_cases.rs"]
mod orchestration_ordering_cases;
#[path = "data_layer_m10_partition_archival/retry_policy_cases.rs"]
mod retry_policy_cases;
#[path = "data_layer_m10_partition_archival/runtime_evidence_cases.rs"]
mod runtime_evidence_cases;
#[path = "data_layer_m10_partition_archival/scheduler_cycle_cases.rs"]
mod scheduler_cycle_cases;
#[path = "data_layer_m10_partition_archival/scheduler_runtime_cases.rs"]
mod scheduler_runtime_cases;
#[path = "data_layer_m10_partition_archival/seam_port_cases.rs"]
mod seam_port_cases;

fn partition_input(
    partition_month_id: u32,
    all_messages_shredded: bool,
) -> DataLayerM10PartitionRecordInput {
    DataLayerM10PartitionRecordInput {
        partition_month_id,
        all_messages_shredded,
    }
}

fn m8_message_input(
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

fn project_request(
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

fn phase6_request(
    owner_did: &str,
    partition_message_ids_by_month: BTreeMap<u32, Vec<String>>,
) -> DataLayerM10Phase6ExecutionTickRequest {
    DataLayerM10Phase6ExecutionTickRequest {
        requester_owner_did: owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        now_epoch_seconds: 1_700_000_000,
        shredded_at_epoch_seconds: 1_700_000_300,
        now_month_id: 202602,
        active_retention_months: 2,
        object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        partition_message_ids_by_month,
    }
}

fn phase6_budget(
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

fn phase6_scheduler_policy(
    due_candidate_trigger_threshold: usize,
    max_tick_interval_seconds: u64,
) -> DataLayerM10Phase6SchedulerPolicy {
    DataLayerM10Phase6SchedulerPolicy {
        due_candidate_trigger_threshold,
        max_tick_interval_seconds,
    }
}

fn phase6_runtime_state(
    total_cycles: u64,
    executed_cycles: u64,
    deferred_cycles: u64,
    fail_closed_cycles: u64,
    last_reason_code: &'static str,
) -> DataLayerM10Phase6SchedulerRuntimeState {
    DataLayerM10Phase6SchedulerRuntimeState {
        last_successful_tick_epoch_seconds: Some(1_700_000_000),
        last_observed_now_epoch_seconds: Some(1_700_000_010),
        total_cycles,
        executed_cycles,
        deferred_cycles,
        fail_closed_cycles,
        last_reason_code,
    }
}

#[test]
fn spec_c01_partition_naming_and_future_planning_are_deterministic() {
    lifecycle_basics_cases::run_spec_c01_partition_naming_and_future_planning_are_deterministic();
}

#[test]
fn spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness() {
    lifecycle_basics_cases::run_spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness();
}

#[test]
fn spec_c03_archival_index_records_and_reattach_transition_are_deterministic() {
    lifecycle_basics_cases::run_spec_c03_archival_index_records_and_reattach_transition_are_deterministic();
}

#[test]
fn spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed() {
    lifecycle_basics_cases::run_spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed();
}

#[test]
fn spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced() {
    lifecycle_basics_cases::run_spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced();
}

#[test]
fn spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records() {
    let owner_did = "kamn:did:owner:alpha";
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-1", 1_708_560_100))
        .expect("message one should register");
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-2", 1_708_560_110))
        .expect("message two should register");

    let initial_projection: DataLayerM10ComplianceShredProjectionReport = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, 202401, vec!["m10-m8-msg-1", "m10-m8-msg-2"]),
        )
        .expect("initial projection should succeed");
    assert_eq!(
        initial_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
    );
    assert!(!initial_projection.all_messages_shredded);
    assert_eq!(initial_projection.shredded_partition_messages, 0);
    assert_eq!(
        initial_projection.projection_reason_code,
        DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE
    );

    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-1".to_owned(),
            shredded_at_epoch_seconds: 1_708_560_200,
        })
        .expect("first message should shred");
    let mid_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, 202401, vec!["m10-m8-msg-1", "m10-m8-msg-2"]),
        )
        .expect("mid projection should succeed");
    assert_eq!(
        mid_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
    );
    assert_eq!(mid_projection.shredded_partition_messages, 1);

    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-2".to_owned(),
            shredded_at_epoch_seconds: 1_708_560_210,
        })
        .expect("second message should shred");
    let final_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, 202401, vec!["m10-m8-msg-1", "m10-m8-msg-2"]),
        )
        .expect("final projection should succeed");
    assert_eq!(
        final_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
    );
    assert!(final_projection.all_messages_shredded);
    assert_eq!(final_projection.shredded_partition_messages, 2);

    let archived = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 1,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
}

#[test]
fn spec_c07_partition_shred_projection_fails_closed_when_m8_message_lookup_is_missing() {
    let owner_did = "kamn:did:owner:alpha";
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(
            owner_did,
            "m10-m8-msg-present",
            1_708_560_100,
        ))
        .expect("message should register");

    let missing = m10_registry.project_partition_shred_completeness_from_m8(
        &m8_registry,
        project_request(
            owner_did,
            202401,
            vec!["m10-m8-msg-present", "m10-m8-msg-missing"],
        ),
    );
    assert!(matches!(
        missing,
        Err(
            DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
                reason_code: DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
                ..
            }
        )
    ));
}

#[test]
fn spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids() {
    let owner_did = "kamn:did:owner:alpha";
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-1", 1_708_560_100))
        .expect("message should register");

    let projection = m10_registry.project_partition_shred_completeness_from_m8(
        &m8_registry,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: "  kamn:did:owner:alpha  ".to_owned(),
            owner_did: owner_did.to_owned(),
            partition_month_id: 202401,
            partition_message_ids: vec!["m10-m8-msg-1".to_owned()],
        },
    );
    assert!(
        projection.is_ok(),
        "canonical-equivalent owner DIDs should authorize projection scope"
    );
}

#[test]
fn spec_c09_partition_projection_denies_non_equivalent_owner_dids() {
    let owner_did = "kamn:did:owner:alpha";
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-1", 1_708_560_100))
        .expect("message should register");

    let denied = m10_registry.project_partition_shred_completeness_from_m8(
        &m8_registry,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: "kamn:did:owner:beta".to_owned(),
            owner_did: owner_did.to_owned(),
            partition_month_id: 202401,
            partition_message_ids: vec!["m10-m8-msg-1".to_owned()],
        },
    );
    assert!(matches!(
        denied,
        Err(DataLayerM10PartitionLifecycleError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c10_partition_projection_marks_legal_hold_as_archival_denied_reason() {
    let owner_did = "kamn:did:owner:alpha";
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-1", 1_708_560_100))
        .expect("message one should register");
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-2", 1_708_560_110))
        .expect("message two should register");

    m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-2".to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should apply");
    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-1".to_owned(),
            shredded_at_epoch_seconds: 1_708_560_200,
        })
        .expect("first message should shred");

    let projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, 202401, vec!["m10-m8-msg-1", "m10-m8-msg-2"]),
        )
        .expect("projection should succeed");
    assert_eq!(
        projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
    );
    assert!(!projection.all_messages_shredded);
    assert_eq!(projection.shredded_partition_messages, 1);
    assert_eq!(
        projection.projection_reason_code,
        DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE
    );
}

#[test]
fn spec_c11_partition_archival_remains_blocked_until_legal_hold_is_released_and_shred_completes() {
    let owner_did = "kamn:did:owner:alpha";
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-1", 1_708_560_100))
        .expect("message one should register");
    m8_registry
        .register_message(m8_message_input(owner_did, "m10-m8-msg-2", 1_708_560_110))
        .expect("message two should register");

    m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-2".to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should apply");
    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-1".to_owned(),
            shredded_at_epoch_seconds: 1_708_560_200,
        })
        .expect("first message should shred");

    let hold_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, 202401, vec!["m10-m8-msg-1", "m10-m8-msg-2"]),
        )
        .expect("hold projection should succeed");
    assert_eq!(
        hold_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
    );
    let blocked_archive = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 1,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    assert!(blocked_archive.is_empty());

    m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-2".to_owned(),
            legal_hold_active: false,
        })
        .expect("legal hold release should apply");
    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "m10-m8-msg-2".to_owned(),
            shredded_at_epoch_seconds: 1_708_560_220,
        })
        .expect("second message should shred after hold release");

    let final_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, 202401, vec!["m10-m8-msg-1", "m10-m8-msg-2"]),
        )
        .expect("final projection should succeed");
    assert_eq!(
        final_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
    );
    assert!(final_projection.all_messages_shredded);

    let archived = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 1,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
}

#[test]
fn spec_c12_transient_archival_failure_projects_deterministic_retry_window() {
    retry_policy_cases::run_spec_c12_transient_archival_failure_projects_deterministic_retry_window();
}

#[test]
fn spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum() {
    retry_policy_cases::run_spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum();
}

#[test]
fn spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed() {
    retry_policy_cases::run_spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed();
}

#[test]
fn spec_c15_archival_retry_policy_and_attempt_validation_fail_closed() {
    retry_policy_cases::run_spec_c15_archival_retry_policy_and_attempt_validation_fail_closed();
}

#[test]
fn spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive() {
    orchestration_ordering_cases::run_spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive();
}

#[test]
fn spec_c17_phase6_orchestration_tick_orders_outputs_deterministically() {
    orchestration_ordering_cases::run_spec_c17_phase6_orchestration_tick_orders_outputs_deterministically();
}

#[test]
fn spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival() {
    execution_budget_cases::run_spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival();
}

#[test]
fn spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries() {
    execution_budget_cases::run_spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries();
}

#[test]
fn spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic() {
    execution_budget_cases::run_spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic();
}

#[test]
fn spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed() {
    execution_budget_cases::run_spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed();
}

#[test]
fn spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed() {
    execution_budget_cases::run_spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed();
}

#[test]
fn spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred() {
    scheduler_cycle_cases::run_spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred();
}

#[test]
fn spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects() {
    scheduler_cycle_cases::run_spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects();
}

#[test]
fn spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution() {
    scheduler_cycle_cases::run_spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution();
}

#[test]
fn spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence() {
    scheduler_cycle_cases::run_spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence();
}

#[test]
fn spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed() {
    scheduler_cycle_cases::run_spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed();
}

#[test]
fn spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint() {
    scheduler_runtime_cases::run_spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint();
}

#[test]
fn spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint() {
    scheduler_runtime_cases::run_spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint();
}

#[test]
fn spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters() {
    scheduler_runtime_cases::run_spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters();
}

#[test]
fn spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance(
) {
    scheduler_runtime_cases::run_spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance();
}

#[test]
fn spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint() {
    scheduler_runtime_cases::run_spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint();
}

#[test]
fn spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts() {
    runtime_evidence_cases::run_spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts();
}

#[test]
fn spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts() {
    runtime_evidence_cases::run_spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts();
}

#[test]
fn spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete() {
    runtime_evidence_cases::run_spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete();
}

#[test]
fn spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data(
) {
    runtime_evidence_cases::run_spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data();
}

#[test]
fn spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument()
{
    seam_port_cases::run_spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument();
}

#[test]
fn spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument() {
    seam_port_cases::run_spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument();
}
