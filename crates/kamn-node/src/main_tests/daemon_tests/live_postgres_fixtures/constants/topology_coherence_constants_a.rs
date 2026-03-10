pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_CONTRACT: &str =
    "topology_id_to_host_mode_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_SCHEMA_VERSION:
    &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-cardinality-mapping.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_ROWS_CSV: &str =
    "same_host_parallel->1,distributed_label_parallel->2";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_CARDINALITY_MAPPING_CONTRACT: &str =
    "topology_id_to_unique_host_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-cardinality-coherence.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_ROWS_CSV:
    &str = "same_host_parallel->same_host->1,distributed_label_parallel->distributed_label->2";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_CARDINALITY_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-cardinality-coherence.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->node_alpha->node_alpha->1,distributed_label_parallel->node_alpha->node_beta->2";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CARDINALITY_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-coherence.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha,distributed_label_parallel->distributed_label->node_alpha->node_beta";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-cardinality-coherence.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->1,distributed_label_parallel->distributed_label->node_alpha->node_beta->2";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_CARDINALITY_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_cardinality_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-count-coherence.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->4,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->4";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_COUNT_COHERENCE_CONTRACT: &str =
    "topology_id_to_host_mode_host_pair_lane_set_lane_count_rows_must_remain_stable_under_repeated_runs_and_permutations";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-host-pair-lane-set-lane-id-bundle-coherence.v1";
pub(crate) const LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_HOST_PAIR_LANE_SET_LANE_ID_BUNDLE_COHERENCE_ROWS_CSV: &str =
    "same_host_parallel->same_host->node_alpha->node_alpha->symmetric_parallel->listener_approver_parallel_applied+listener_approver_parallel_deferred+processor_listener_parallel_applied+processor_listener_parallel_deferred,distributed_label_parallel->distributed_label->node_alpha->node_beta->asymmetric_parallel->listener_approver_asymmetric_parallel_applied+listener_approver_asymmetric_parallel_deferred+processor_listener_asymmetric_parallel_applied+processor_listener_asymmetric_parallel_deferred";
