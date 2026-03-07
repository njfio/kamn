//! Deterministic M10 partition month-id parsing and naming policy.

use std::fmt;

use crate::data_layer_hashing::tagged_sha256;

/// Partition prefix for monthly message partitions.
pub const DATA_LAYER_M10_PARTITION_PREFIX: &str = "messages_";

/// Error surface for M10 partition month-id policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10PartitionMonthPolicyError {
    /// Partition month id failed `YYYYMM` validation.
    InvalidPartitionMonthId(u32),
}

impl fmt::Display for DataLayerM10PartitionMonthPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPartitionMonthId(value) => {
                write!(f, "invalid partition month id: {value}")
            }
        }
    }
}

impl std::error::Error for DataLayerM10PartitionMonthPolicyError {}

/// Validates an M10 partition month id encoded as `YYYYMM`.
pub fn data_layer_m10_validate_partition_month_id(
    partition_month_id: u32,
) -> Result<(), DataLayerM10PartitionMonthPolicyError> {
    let _ = data_layer_m10_split_month_id(partition_month_id)?;
    Ok(())
}

/// Splits an M10 partition month id encoded as `YYYYMM`.
pub fn data_layer_m10_split_month_id(
    partition_month_id: u32,
) -> Result<(u32, u32), DataLayerM10PartitionMonthPolicyError> {
    let year = partition_month_id / 100;
    let month = partition_month_id % 100;
    if year < 1970 || !(1..=12).contains(&month) {
        return Err(
            DataLayerM10PartitionMonthPolicyError::InvalidPartitionMonthId(partition_month_id),
        );
    }
    Ok((year, month))
}

/// Adds whole months to an M10 partition month id with deterministic rollover.
pub fn data_layer_m10_add_months(
    partition_month_id: u32,
    months_to_add: u32,
) -> Result<u32, DataLayerM10PartitionMonthPolicyError> {
    let (year, month) = data_layer_m10_split_month_id(partition_month_id)?;
    let base = year * 12 + (month - 1);
    let future = base + months_to_add;
    let future_year = future / 12;
    let future_month = (future % 12) + 1;
    Ok(future_year * 100 + future_month)
}

/// Computes saturated month distance between two M10 partition month ids.
pub fn data_layer_m10_month_distance(
    older_partition_month_id: u32,
    newer_partition_month_id: u32,
) -> Result<u32, DataLayerM10PartitionMonthPolicyError> {
    let (older_year, older_month) = data_layer_m10_split_month_id(older_partition_month_id)?;
    let (newer_year, newer_month) = data_layer_m10_split_month_id(newer_partition_month_id)?;
    let older = older_year * 12 + (older_month - 1);
    let newer = newer_year * 12 + (newer_month - 1);
    Ok(newer.saturating_sub(older))
}

/// Formats partition month id (`YYYYMM`) as `messages_YYYY_MM`.
pub fn data_layer_m10_format_partition_name(
    partition_month_id: u32,
) -> Result<String, DataLayerM10PartitionMonthPolicyError> {
    let (year, month) = data_layer_m10_split_month_id(partition_month_id)?;
    Ok(format!(
        "{DATA_LAYER_M10_PARTITION_PREFIX}{year:04}_{month:02}"
    ))
}

/// Projects a deterministic checksum marker for one partition archival record.
pub fn data_layer_m10_deterministic_checksum_marker(
    partition_name: &str,
    partition_month_id: u32,
) -> String {
    let canonical_payload = format!("{partition_name}:{partition_month_id}");
    tagged_sha256(canonical_payload.as_str(), "sha256")
}
