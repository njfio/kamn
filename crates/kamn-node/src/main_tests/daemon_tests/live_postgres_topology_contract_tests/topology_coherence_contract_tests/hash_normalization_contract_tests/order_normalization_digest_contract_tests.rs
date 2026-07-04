use super::super::*;

#[test]
fn functional_live_postgres_topology_order_normalized_digest_is_canonical() {
    assert_topology_metadata(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_SCHEMA_VERSION,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_CONTRACT,
    );
    let (rows, digest) =
        project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
            project_live_postgres_parallel_lane_topology_profiles(),
        );
    assert_topology_rows_match(
        rows,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_CSV,
    );
    assert_eq!(
        digest,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_FINGERPRINT_HASH_ORDER_NORMALIZATION_DIGEST_ROWS_FNV1A64_HEX,
    );
}
