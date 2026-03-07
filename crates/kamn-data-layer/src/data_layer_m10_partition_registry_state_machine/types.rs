/// Archive format marker for exported partition artifacts.
pub const DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD: &str = "parquet-zstd";
/// Stable reason marker for archived lifecycle transitions.
pub const DATA_LAYER_M10_ARCHIVE_REASON_CODE: &str = "m10_partition_archived";
/// Stable reason marker for archived -> reattached transitions.
pub const DATA_LAYER_M10_REATTACH_REASON_CODE: &str = "m10_partition_reattached";
/// Stable reason marker for invalid lifecycle transitions.
pub const DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE: &str = "m10_partition_transition_invalid";
/// Stable reason marker when partition recoverability is ready for historical replay.
pub const DATA_LAYER_M10_RECOVERY_READY_REASON_CODE: &str = "m10_partition_recovery_ready";
/// Stable reason marker when partition status is not eligible for historical recovery.
pub const DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE: &str =
    "m10_partition_recovery_status_ineligible";
/// Stable reason marker when historical partition metadata is incomplete.
pub const DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE: &str =
    "m10_partition_recovery_metadata_incomplete";

/// Partition lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10PartitionStatus {
    /// Active partition in primary query path.
    Active,
    /// Archived partition with export metadata in archival index.
    Archived,
    /// Archived partition reattached for historical query access.
    Reattached,
}

/// Recoverability decision for one partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM10RecoveryDecision {
    /// Partition has complete archival metadata and can be recovered.
    Ready,
    /// Partition cannot be recovered under current state/metadata.
    Blocked,
}

/// Partition registration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10PartitionRecordInput {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// True when all rows in partition are shred-complete and eligible for archival export.
    pub all_messages_shredded: bool,
}

/// Partition lifecycle record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10PartitionRecord {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Shred-complete marker for archival eligibility checks.
    pub all_messages_shredded: bool,
    /// Current lifecycle status.
    pub lifecycle_status: DataLayerM10PartitionStatus,
    /// Archived object URI when partition is archived.
    pub archived_object_uri: Option<String>,
    /// Archive format marker when partition is archived.
    pub archive_format_marker: Option<&'static str>,
    /// Deterministic checksum marker when partition is archived.
    pub checksum_marker: Option<String>,
    /// Last lifecycle transition reason marker.
    pub last_reason_code: Option<&'static str>,
}

/// Archive evaluation request envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ArchiveDueRequest {
    /// Current month identifier as `YYYYMM`.
    pub now_month_id: u32,
    /// Active retention window in months.
    pub active_retention_months: u16,
    /// Object-storage prefix used for archived artifacts.
    pub object_storage_prefix: String,
}

/// Archival index projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ArchivalIndexEntry {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Archived object URI.
    pub archived_object_uri: String,
    /// Archive format marker.
    pub archive_format_marker: &'static str,
    /// Deterministic checksum marker.
    pub checksum_marker: String,
    /// Lifecycle status after archive transition.
    pub lifecycle_status: DataLayerM10PartitionStatus,
}

/// Recoverability readiness report for one partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10RecoveryReadinessReport {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Recoverability decision.
    pub decision: DataLayerM10RecoveryDecision,
    /// Stable reason marker for the decision.
    pub reason_code: &'static str,
    /// Current lifecycle status.
    pub lifecycle_status: DataLayerM10PartitionStatus,
    /// Archived object URI.
    pub archived_object_uri: Option<String>,
    /// Archive format marker.
    pub archive_format_marker: Option<&'static str>,
    /// Deterministic checksum marker.
    pub checksum_marker: Option<String>,
}
