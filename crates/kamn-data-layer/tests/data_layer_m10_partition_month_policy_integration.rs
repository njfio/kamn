use kamn_data_layer::{
    data_layer_m10_add_months, data_layer_m10_format_partition_name, data_layer_m10_month_distance,
    data_layer_m10_split_month_id, data_layer_m10_validate_partition_month_id,
    DataLayerM10PartitionMonthPolicyError,
};

#[test]
fn integration_partition_month_policy_formats_and_projects_months() {
    assert_eq!(data_layer_m10_split_month_id(202512), Ok((2025, 12)));
    assert_eq!(
        data_layer_m10_format_partition_name(202602),
        Ok("messages_2026_02".to_owned())
    );
    assert_eq!(data_layer_m10_add_months(202512, 1), Ok(202601));
    assert_eq!(data_layer_m10_month_distance(202411, 202502), Ok(3));
}

#[test]
fn integration_partition_month_policy_rejects_invalid_month_ranges() {
    assert_eq!(
        data_layer_m10_validate_partition_month_id(196912),
        Err(DataLayerM10PartitionMonthPolicyError::InvalidPartitionMonthId(196912))
    );
    assert_eq!(
        data_layer_m10_split_month_id(202513),
        Err(DataLayerM10PartitionMonthPolicyError::InvalidPartitionMonthId(202513))
    );
}
