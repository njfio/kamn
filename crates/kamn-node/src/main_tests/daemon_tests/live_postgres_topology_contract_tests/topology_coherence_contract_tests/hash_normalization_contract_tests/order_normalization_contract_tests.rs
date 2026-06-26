use super::super::*;

#[test]
fn functional_live_postgres_topology_order_normalized_hash_rows_are_canonical() {
    assert_topology_metadata(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_SCHEMA_VERSION,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_CONTRACT,
    );
    assert_topology_rows_match(
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
            project_live_postgres_parallel_lane_topology_profiles(),
        ),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_ROWS_CSV,
    );
}
