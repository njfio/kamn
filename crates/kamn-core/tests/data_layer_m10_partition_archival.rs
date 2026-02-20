use kamn_core::{
    data_layer_m10_execute_phase6_orchestration_tick, data_layer_m10_format_partition_name,
    data_layer_m10_project_archival_retry_decision, DataLayerM10ArchivalFailureClass,
    DataLayerM10ArchivalRecoveryAction, DataLayerM10ArchivalRetryPolicy,
    DataLayerM10ArchiveDueRequest, DataLayerM10ComplianceShredProjectionReport,
    DataLayerM10ComplianceShredProjectionRequest, DataLayerM10PartitionLifecycleError,
    DataLayerM10PartitionLifecycleRegistry, DataLayerM10PartitionRecordInput,
    DataLayerM10PartitionStatus, DataLayerM10Phase6ExecutionTickRequest,
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
    DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_REATTACH_REASON_CODE,
};
use std::collections::BTreeMap;

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

#[test]
fn spec_c01_partition_naming_and_future_planning_are_deterministic() {
    let registry = DataLayerM10PartitionLifecycleRegistry::new();
    assert_eq!(
        data_layer_m10_format_partition_name(202602).expect("month should format"),
        "messages_2026_02"
    );

    let planned = registry
        .plan_future_partition_names(202602, 3)
        .expect("future planning should succeed");
    assert_eq!(
        planned,
        vec!["messages_2026_03", "messages_2026_04", "messages_2026_05"]
    );
}

#[test]
fn spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202401, true))
        .expect("old shred-complete partition should register");
    registry
        .register_partition(partition_input(202402, false))
        .expect("old non-shredded partition should register");
    registry
        .register_partition(partition_input(202601, true))
        .expect("recent partition should register");

    let archived = registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");

    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
    assert_eq!(
        archived[0].lifecycle_status,
        DataLayerM10PartitionStatus::Archived
    );
}

#[test]
fn spec_c03_archival_index_records_and_reattach_transition_are_deterministic() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202401, true))
        .expect("old shred-complete partition should register");

    let archived = registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 1,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    assert_eq!(archived.len(), 1);
    assert!(archived[0]
        .archived_object_uri
        .starts_with("s3://kamn-archive/messages/messages_2024_01"));
    assert_eq!(
        archived[0].archive_format_marker,
        DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD
    );

    let reattached = registry
        .reattach_partition("messages_2024_01")
        .expect("reattach should succeed");
    assert_eq!(
        reattached.lifecycle_status,
        DataLayerM10PartitionStatus::Reattached
    );
    assert_eq!(
        reattached.last_reason_code,
        Some(DATA_LAYER_M10_REATTACH_REASON_CODE)
    );
}

#[test]
fn spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    let invalid = registry.register_partition(partition_input(202613, true));
    assert!(matches!(
        invalid,
        Err(DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(202613))
    ));

    registry
        .register_partition(partition_input(202602, true))
        .expect("valid partition should register");
    let illegal = registry.reattach_partition("messages_2026_02");
    assert!(matches!(
        illegal,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidLifecycleTransition {
                reason_code: "m10_partition_transition_invalid",
                ..
            }
        )
    ));
}

#[test]
fn spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202512, true))
        .expect("partition should register");
    let duplicate = registry.register_partition(partition_input(202512, true));
    assert!(matches!(
        duplicate,
        Err(DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(202512))
    ));

    let planned = registry
        .plan_future_partition_names(202512, 1)
        .expect("future planning should succeed");
    assert_eq!(planned.len(), 1);
    assert!(planned[0].starts_with(DATA_LAYER_M10_PARTITION_PREFIX));
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
    let policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 4,
        base_backoff_seconds: 5,
        max_backoff_seconds: 60,
    };
    let decision = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        2,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("transient failure should project retry");

    assert_eq!(
        decision.action,
        DataLayerM10ArchivalRecoveryAction::RetryScheduled
    );
    assert_eq!(decision.current_attempt, 2);
    assert_eq!(decision.next_attempt, Some(3));
    assert_eq!(decision.retry_backoff_seconds, Some(10));
    assert_eq!(decision.retry_after_unix_seconds, Some(1_700_000_010));
    assert_eq!(decision.attempts_remaining, 2);
    assert_eq!(
        decision.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE
    );
}

#[test]
fn spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum() {
    let policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 16,
        base_backoff_seconds: 4,
        max_backoff_seconds: 20,
    };
    let decision = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        8,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("transient failure should project retry");

    assert_eq!(
        decision.action,
        DataLayerM10ArchivalRecoveryAction::RetryScheduled
    );
    assert_eq!(decision.retry_backoff_seconds, Some(20));
    assert_eq!(decision.retry_after_unix_seconds, Some(1_700_000_020));
    assert_eq!(decision.next_attempt, Some(9));
    assert_eq!(
        decision.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE
    );
}

#[test]
fn spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed() {
    let policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 3,
        base_backoff_seconds: 5,
        max_backoff_seconds: 30,
    };
    let exhausted = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        3,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("exhausted transient should project fail-closed");
    assert_eq!(
        exhausted.action,
        DataLayerM10ArchivalRecoveryAction::FailClosed
    );
    assert_eq!(exhausted.next_attempt, None);
    assert_eq!(exhausted.retry_backoff_seconds, None);
    assert_eq!(exhausted.retry_after_unix_seconds, None);
    assert_eq!(exhausted.attempts_remaining, 0);
    assert_eq!(
        exhausted.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE
    );

    let permanent = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        1,
        DataLayerM10ArchivalFailureClass::Permanent,
        policy,
    )
    .expect("permanent failure should project fail-closed");
    assert_eq!(
        permanent.action,
        DataLayerM10ArchivalRecoveryAction::FailClosed
    );
    assert_eq!(permanent.next_attempt, None);
    assert_eq!(permanent.retry_backoff_seconds, None);
    assert_eq!(permanent.retry_after_unix_seconds, None);
    assert_eq!(permanent.attempts_remaining, 0);
    assert_eq!(
        permanent.reason_code,
        DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE
    );
}

#[test]
fn spec_c15_archival_retry_policy_and_attempt_validation_fail_closed() {
    let invalid_policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 0,
        base_backoff_seconds: 5,
        max_backoff_seconds: 30,
    };
    let invalid_policy_error = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        1,
        DataLayerM10ArchivalFailureClass::Transient,
        invalid_policy,
    );
    assert!(matches!(
        invalid_policy_error,
        Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "max_attempts",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        })
    ));

    let invalid_attempt = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        0,
        DataLayerM10ArchivalFailureClass::Transient,
        DataLayerM10ArchivalRetryPolicy {
            max_attempts: 3,
            base_backoff_seconds: 5,
            max_backoff_seconds: 30,
        },
    );
    assert!(matches!(
        invalid_attempt,
        Err(DataLayerM10PartitionLifecycleError::InvalidRetryAttempt {
            field: "current_attempt",
            value: 0,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive() {
    let owner_did = "kamn:did:owner:phase6-alpha";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    let mut message_a = m8_message_input(owner_did, "message-a", 1_699_800_000);
    message_a.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(message_a)
        .expect("message-a should register");
    let mut message_b = m8_message_input(owner_did, "message-b", 1_699_810_000);
    message_b.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(message_b)
        .expect("message-b should register");

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(202401, vec!["message-b".to_owned(), "message-a".to_owned()])]),
        ),
    )
    .expect("phase6 execution tick should succeed");

    assert_eq!(report.owner_did, owner_did);
    assert_eq!(report.due_candidate_count, 2);
    assert_eq!(
        report.shredded_message_ids,
        vec!["message-a".to_owned(), "message-b".to_owned()]
    );
    assert_eq!(report.projection_reports.len(), 1);
    assert!(report.projection_reports[0].all_messages_shredded);
    assert_eq!(report.archived_entries.len(), 1);
    assert_eq!(
        report.archived_entries[0].partition_name,
        "messages_2024_01"
    );
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE
    );
}

#[test]
fn spec_c17_phase6_orchestration_tick_orders_outputs_deterministically() {
    let owner_did = "kamn:did:owner:phase6-beta";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");
    m10_registry
        .register_partition(partition_input(202402, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("m-01", 1_699_700_000_u64),
        ("m-02", 1_699_700_100_u64),
        ("m-11", 1_699_700_200_u64),
        ("m-12", 1_699_700_300_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([
                (202402, vec!["m-12".to_owned(), "m-11".to_owned()]),
                (202401, vec!["m-02".to_owned(), "m-01".to_owned()]),
            ]),
        ),
    )
    .expect("phase6 execution tick should succeed");

    let projection_months: Vec<u32> = report
        .projection_reports
        .iter()
        .map(|report| report.partition_month_id)
        .collect();
    assert_eq!(projection_months, vec![202401, 202402]);
    assert_eq!(
        report.shredded_message_ids,
        vec![
            "m-01".to_owned(),
            "m-02".to_owned(),
            "m-11".to_owned(),
            "m-12".to_owned(),
        ]
    );
    let archived_partition_names: Vec<String> = report
        .archived_entries
        .iter()
        .map(|entry| entry.partition_name.clone())
        .collect();
    assert_eq!(
        archived_partition_names,
        vec!["messages_2024_01".to_owned(), "messages_2024_02".to_owned()]
    );
}

#[test]
fn spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival() {
    let owner_did = "kamn:did:owner:phase6-gamma";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202601, false))
        .expect("partition should register");

    let mut recent_message = m8_message_input(owner_did, "message-z", 1_699_999_900);
    recent_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(recent_message)
        .expect("message should register");

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(202601, vec!["message-z".to_owned()])]),
        ),
    )
    .expect("phase6 execution tick should succeed");

    assert_eq!(report.due_candidate_count, 0);
    assert!(report.shredded_message_ids.is_empty());
    assert_eq!(report.projection_reports.len(), 1);
    assert!(!report.projection_reports[0].all_messages_shredded);
    assert!(report.archived_entries.is_empty());
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE
    );
}

#[test]
fn spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries() {
    let owner_did = "kamn:did:owner:phase6-delta";

    let mut hold_m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut hold_m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    hold_m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");
    let mut held_message = m8_message_input(owner_did, "message-held", 1_699_700_000);
    held_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    hold_m8_registry
        .register_message(held_message)
        .expect("message should register");
    hold_m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "message-held".to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should apply");
    let legal_hold_error = data_layer_m10_execute_phase6_orchestration_tick(
        &mut hold_m8_registry,
        &mut hold_m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(202401, vec!["message-held".to_owned()])]),
        ),
    );
    assert!(matches!(
        legal_hold_error,
        Err(DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed {
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
            ..
        })
    ));

    let mut empty_m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut empty_m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    empty_m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");
    empty_m8_registry
        .register_message(m8_message_input(owner_did, "message-empty", 1_699_999_000))
        .expect("message should register");
    let empty_projection_error = data_layer_m10_execute_phase6_orchestration_tick(
        &mut empty_m8_registry,
        &mut empty_m10_registry,
        phase6_request(owner_did, BTreeMap::from([(202401, Vec::new())])),
    );
    assert!(matches!(
        empty_projection_error,
        Err(DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed {
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
            ..
        })
    ));
}
