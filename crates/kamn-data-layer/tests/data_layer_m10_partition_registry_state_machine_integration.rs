use kamn_data_layer::{
    DataLayerM10ArchiveDueRequest, DataLayerM10PartitionRecordInput,
    DataLayerM10PartitionRegistryStateMachine, DataLayerM10PartitionRegistryStateMachineError,
    DataLayerM10PartitionStatus, DataLayerM10RecoveryDecision,
    DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD, DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
    DATA_LAYER_M10_REATTACH_REASON_CODE, DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
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
fn integration_registry_state_machine_projects_deterministic_lifecycle_outputs() {
    let mut registry = DataLayerM10PartitionRegistryStateMachine::new();
    registry
        .register_partition(partition_input(202401, true))
        .expect("partition should register");
    registry
        .register_partition(partition_input(202402, true))
        .expect("partition should register");

    assert_eq!(
        registry
            .plan_future_partition_names(202412, 2)
            .expect("planning should succeed"),
        vec!["messages_2025_01", "messages_2025_02"]
    );

    let archived = registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .expect("archive due should succeed");
    assert_eq!(archived.len(), 2);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
    assert_eq!(
        archived[0].lifecycle_status,
        DataLayerM10PartitionStatus::Archived
    );
    assert_eq!(
        archived[0].archive_format_marker,
        DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD
    );

    let reattached = registry
        .reattach_partition("messages_2024_01")
        .expect("reattach should succeed");
    assert_eq!(
        reattached.last_reason_code,
        Some(DATA_LAYER_M10_REATTACH_REASON_CODE)
    );

    let readiness = registry
        .evaluate_partition_recovery_readiness("messages_2024_01")
        .expect("recovery readiness should evaluate");
    assert_eq!(readiness.decision, DataLayerM10RecoveryDecision::Ready);
    assert_eq!(
        readiness.reason_code,
        DATA_LAYER_M10_RECOVERY_READY_REASON_CODE
    );
}

#[test]
fn integration_registry_state_machine_lists_historical_readiness_in_sorted_order() {
    let mut registry = DataLayerM10PartitionRegistryStateMachine::new();
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
}

#[test]
fn integration_registry_state_machine_fails_closed_on_invalid_transitions_and_unknown_names() {
    let mut registry = DataLayerM10PartitionRegistryStateMachine::new();
    registry
        .register_partition(partition_input(202602, true))
        .expect("partition should register");

    let active = registry
        .evaluate_partition_recovery_readiness("messages_2026_02")
        .expect("active partition should evaluate");
    assert_eq!(active.decision, DataLayerM10RecoveryDecision::Blocked);
    assert_eq!(
        active.reason_code,
        DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE
    );

    assert_eq!(
        registry.reattach_partition("messages_2026_02"),
        Err(
            DataLayerM10PartitionRegistryStateMachineError::InvalidLifecycleTransition {
                partition_name: "messages_2026_02".to_owned(),
                from_status: DataLayerM10PartitionStatus::Active,
                to_status: DataLayerM10PartitionStatus::Reattached,
                reason_code: DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
            }
        )
    );

    assert_eq!(
        registry.evaluate_partition_recovery_readiness("messages_2024_99"),
        Err(
            DataLayerM10PartitionRegistryStateMachineError::PartitionNotFound(
                "messages_2024_99".to_owned()
            )
        )
    );
}
