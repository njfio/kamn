//! Deterministic M10 partition registry lifecycle state machine extracted from `kamn-core`.

mod error;
mod helpers;
mod machine;
mod types;

use helpers::project_partition_recovery_readiness;

pub use error::DataLayerM10PartitionRegistryStateMachineError;
pub use machine::DataLayerM10PartitionRegistryStateMachine;
pub use types::{
    DataLayerM10ArchivalIndexEntry, DataLayerM10ArchiveDueRequest, DataLayerM10PartitionRecord,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus, DataLayerM10RecoveryDecision,
    DataLayerM10RecoveryReadinessReport, DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
    DATA_LAYER_M10_ARCHIVE_REASON_CODE, DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
    DATA_LAYER_M10_REATTACH_REASON_CODE, DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
};

impl DataLayerM10PartitionRegistryStateMachine {
    /// Creates an empty partition lifecycle registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists recoverability readiness for historical partitions in deterministic order.
    pub fn list_historical_recovery_readiness(&self) -> Vec<DataLayerM10RecoveryReadinessReport> {
        let mut reports: Vec<_> = self
            .partitions
            .values()
            .filter(|record| record.lifecycle_status != DataLayerM10PartitionStatus::Active)
            .map(project_partition_recovery_readiness)
            .collect();
        reports.sort_by(|left, right| {
            left.partition_month_id
                .cmp(&right.partition_month_id)
                .then(left.partition_name.cmp(&right.partition_name))
        });
        reports
    }
}
