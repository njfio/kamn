use super::*;

const NOW_MONTH_ID: u32 = 202602;
const ARCHIVE_PREFIX: &str = "s3://kamn-archive/messages";
const PARTITION_OLD_PRIMARY: u32 = 202401;
const PARTITION_OLD_SECONDARY: u32 = 202402;
const PARTITION_RECENT: u32 = 202601;
const PARTITION_DUPLICATE: u32 = 202512;
const PARTITION_INVALID: u32 = 202613;

pub(super) fn run_spec_c01_partition_naming_and_future_planning_are_deterministic() {
    let registry = DataLayerM10PartitionLifecycleRegistry::new();
    assert_eq!(
        data_layer_m10_format_partition_name(NOW_MONTH_ID).expect("month should format"),
        "messages_2026_02"
    );

    let planned = registry
        .plan_future_partition_names(NOW_MONTH_ID, 3)
        .expect("future planning should succeed");
    assert_eq!(
        planned,
        vec!["messages_2026_03", "messages_2026_04", "messages_2026_05"]
    );
}

pub(super) fn run_spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness()
{
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(PARTITION_OLD_PRIMARY, true))
        .expect("old shred-complete partition should register");
    registry
        .register_partition(partition_input(PARTITION_OLD_SECONDARY, false))
        .expect("old non-shredded partition should register");
    registry
        .register_partition(partition_input(PARTITION_RECENT, true))
        .expect("recent partition should register");

    let archived = registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: NOW_MONTH_ID,
            active_retention_months: 2,
            object_storage_prefix: ARCHIVE_PREFIX.to_owned(),
        })
        .expect("archive due should succeed");

    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
    assert_eq!(
        archived[0].lifecycle_status,
        DataLayerM10PartitionStatus::Archived
    );
}

pub(super) fn run_spec_c03_archival_index_records_and_reattach_transition_are_deterministic() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(PARTITION_OLD_PRIMARY, true))
        .expect("old shred-complete partition should register");

    let archived = registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: NOW_MONTH_ID,
            active_retention_months: 1,
            object_storage_prefix: ARCHIVE_PREFIX.to_owned(),
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

pub(super) fn run_spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    let invalid = registry.register_partition(partition_input(PARTITION_INVALID, true));
    assert!(matches!(
        invalid,
        Err(DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(PARTITION_INVALID))
    ));

    registry
        .register_partition(partition_input(NOW_MONTH_ID, true))
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

pub(super) fn run_spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(PARTITION_DUPLICATE, true))
        .expect("partition should register");
    let duplicate = registry.register_partition(partition_input(PARTITION_DUPLICATE, true));
    assert!(matches!(
        duplicate,
        Err(DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(PARTITION_DUPLICATE))
    ));

    let planned = registry
        .plan_future_partition_names(PARTITION_DUPLICATE, 1)
        .expect("future planning should succeed");
    assert_eq!(planned.len(), 1);
    assert!(planned[0].starts_with(DATA_LAYER_M10_PARTITION_PREFIX));
}
