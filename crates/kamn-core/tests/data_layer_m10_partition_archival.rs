use kamn_core::{
    data_layer_m10_format_partition_name, DataLayerM10ArchiveDueRequest,
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus,
    DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD, DATA_LAYER_M10_PARTITION_PREFIX,
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
