use crate::KamnDid;
use kamn_data_layer::DataLayerM10PartitionMonthPolicyError;

use super::*;

pub(super) fn parse_kamn_did(value: &str) -> Result<KamnDid, DataLayerM10PartitionLifecycleError> {
    KamnDid::parse(value).map_err(|_| {
        DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
            reason_code: DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
            detail: format!("invalid did: {value}"),
        }
    })
}

pub(super) fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<KamnDid, DataLayerM10PartitionLifecycleError> {
    let requester_owner_did = parse_kamn_did(requester_owner_did)?;
    let owner_did = parse_kamn_did(owner_did)?;
    if requester_owner_did.as_str() != owner_did.as_str() {
        return Err(DataLayerM10PartitionLifecycleError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
        });
    }
    Ok(owner_did)
}

pub(super) fn map_partition_month_policy_error_to_m10(
    error: DataLayerM10PartitionMonthPolicyError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10PartitionMonthPolicyError::InvalidPartitionMonthId(value) => {
            DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(value)
        }
    }
}

pub(super) fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if value.trim().is_empty() {
        return Err(DataLayerM10PartitionLifecycleError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn validate_partition_month_id(
    partition_month_id: u32,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    kamn_data_layer::data_layer_m10_validate_partition_month_id(partition_month_id)
        .map_err(map_partition_month_policy_error_to_m10)
}

pub(super) fn add_months(
    partition_month_id: u32,
    months_to_add: u32,
) -> Result<u32, DataLayerM10PartitionLifecycleError> {
    kamn_data_layer::data_layer_m10_add_months(partition_month_id, months_to_add)
        .map_err(map_partition_month_policy_error_to_m10)
}

pub(super) fn month_distance(
    older_partition_month_id: u32,
    newer_partition_month_id: u32,
) -> Result<u32, DataLayerM10PartitionLifecycleError> {
    kamn_data_layer::data_layer_m10_month_distance(
        older_partition_month_id,
        newer_partition_month_id,
    )
    .map_err(map_partition_month_policy_error_to_m10)
}

pub(super) fn deterministic_checksum_marker(
    partition_name: &str,
    partition_month_id: u32,
) -> String {
    kamn_data_layer::data_layer_m10_deterministic_checksum_marker(
        partition_name,
        partition_month_id,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        add_months, deterministic_checksum_marker, month_distance, validate_partition_month_id,
    };
    use crate::DataLayerM10PartitionLifecycleError;

    #[test]
    fn unit_split_month_id_and_validation_reject_invalid_ranges() {
        assert_eq!(
            validate_partition_month_id(196912),
            Err(DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(196912))
        );
        assert_eq!(
            validate_partition_month_id(202513),
            Err(DataLayerM10PartitionLifecycleError::InvalidPartitionMonthId(202513))
        );
        assert_eq!(validate_partition_month_id(202512), Ok(()));
    }

    #[test]
    fn unit_month_arithmetic_handles_year_rollover_deterministically() {
        assert_eq!(add_months(202512, 1), Ok(202601));
        assert_eq!(add_months(202511, 3), Ok(202602));
        assert_eq!(month_distance(202411, 202502), Ok(3));
    }

    #[test]
    fn unit_deterministic_checksum_marker_has_stable_shape() {
        let marker = deterministic_checksum_marker("messages_2025_02", 202502);
        assert_eq!(
            marker,
            "sha256:436a53bb2b45f5bfe769623a94cefdd17c75bc18ceb4c6d985884933ab268a42"
        );
        assert_ne!(marker, "sha256:messages_2025_02:202502");
    }
}
