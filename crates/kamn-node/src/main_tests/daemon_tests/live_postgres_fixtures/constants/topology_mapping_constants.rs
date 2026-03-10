pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_IDS_CSV: &str =
    "same_host_parallel,distributed_label_parallel";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_CONTRACT: &str =
    "topology_labels_must_preserve_sorted_lane_reason_taxonomy_fingerprints_under_repeated_runs";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_IDS_CSV: &str =
    "baseline,reverse,rotate_left_1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_PERMUTATION_CONTRACT: &str =
    "deterministic_topology_profile_permutations_must_preserve_sorted_topology_fingerprint_bundles";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV: &str =
    "node_alpha->node_alpha,node_alpha->node_beta";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CONTRACT: &str =
    "host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_SCHEMA_VERSION:
    &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_EXTRACTION_RULE:
    &str = "host_a_to_host_b_arrow_notation_non_commutative";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV:
    &str = "node_beta->node_alpha";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-mapping.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->node_alpha->node_alpha,distributed_label_parallel->node_alpha->node_beta";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_MAPPING_CONTRACT: &str =
    "topology_id_to_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-set-mapping.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->symmetric_parallel,distributed_label_parallel->asymmetric_parallel";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_SET_MAPPING_CONTRACT: &str =
    "topology_id_to_lane_set_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-lane-count-mapping.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->4,distributed_label_parallel->4";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_COUNT_MAPPING_CONTRACT: &str =
    "topology_id_to_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-mapping.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->same_host,distributed_label_parallel->distributed_label";
