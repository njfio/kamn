use kamn_core::{
    data_layer_m10_format_partition_name, DataLayerM10ArchiveDueRequest,
    DataLayerM10ComplianceShredProjectionReport, DataLayerM10ComplianceShredProjectionRequest,
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus, DataLayerM8ComplianceRegistry,
    DataLayerM8CryptoShredRequest, DataLayerM8MessageRecordInput, DataLayerM8RetentionClass,
    DataLayerM8WrappedCekInput, DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE, DATA_LAYER_M10_PARTITION_PREFIX,
    DATA_LAYER_M10_REATTACH_REASON_CODE,
};

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
