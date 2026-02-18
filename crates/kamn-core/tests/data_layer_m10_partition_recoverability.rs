use kamn_core::{
    DataLayerM10ArchiveDueRequest, DataLayerM10PartitionLifecycleError,
    DataLayerM10PartitionLifecycleRegistry, DataLayerM10PartitionRecordInput,
    DataLayerM10PartitionStatus, DataLayerM10RecoveryDecision,
    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
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
fn spec_c01_archived_partition_recovery_readiness_is_ready() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202401, true))
        .expect("partition should register");
    registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");

    let report = registry
        .evaluate_partition_recovery_readiness("messages_2024_01")
        .expect("recovery readiness should evaluate");
    assert_eq!(report.decision, DataLayerM10RecoveryDecision::Ready);
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_RECOVERY_READY_REASON_CODE
    );
    assert_eq!(
        report.lifecycle_status,
        DataLayerM10PartitionStatus::Archived
    );
}

#[test]
fn spec_c02_reattached_partition_recovery_readiness_remains_ready() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202401, true))
        .expect("partition should register");
    registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    registry
        .reattach_partition("messages_2024_01")
        .expect("reattach should succeed");

    let report = registry
        .evaluate_partition_recovery_readiness("messages_2024_01")
        .expect("recovery readiness should evaluate");
    assert_eq!(report.decision, DataLayerM10RecoveryDecision::Ready);
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_RECOVERY_READY_REASON_CODE
    );
    assert_eq!(
        report.lifecycle_status,
        DataLayerM10PartitionStatus::Reattached
    );
}

#[test]
fn spec_c03_active_partition_is_blocked_for_recoverability() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202602, true))
        .expect("partition should register");

    let report = registry
        .evaluate_partition_recovery_readiness("messages_2026_02")
        .expect("recovery readiness should evaluate");
    assert_eq!(report.decision, DataLayerM10RecoveryDecision::Blocked);
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE
    );
    assert_eq!(report.lifecycle_status, DataLayerM10PartitionStatus::Active);
}

#[test]
fn spec_c04_historical_recovery_readiness_catalog_is_deterministic() {
    let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
    registry
        .register_partition(partition_input(202401, true))
        .expect("partition should register");
    registry
        .register_partition(partition_input(202402, true))
        .expect("partition should register");
    registry
        .register_partition(partition_input(202602, true))
        .expect("active partition should register");
    registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    registry
        .reattach_partition("messages_2024_02")
        .expect("reattach should succeed");

    let reports = registry.list_historical_recovery_readiness();
    let names: Vec<&str> = reports
        .iter()
        .map(|report| report.partition_name.as_str())
        .collect();
    assert_eq!(names, vec!["messages_2024_01", "messages_2024_02"]);
    assert!(reports
        .iter()
        .all(|report| report.decision == DataLayerM10RecoveryDecision::Ready));
}

#[test]
fn spec_c05_unknown_partition_lookup_fails_closed() {
    let registry = DataLayerM10PartitionLifecycleRegistry::new();
    let missing = registry.evaluate_partition_recovery_readiness("messages_2024_99");
    assert!(matches!(
        missing,
        Err(DataLayerM10PartitionLifecycleError::PartitionNotFound(name))
        if name == "messages_2024_99"
    ));
}
