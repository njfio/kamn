use super::super::models::*;
use super::row_extraction_host_rows::*;
use super::runner_support::*;
pub(crate) fn collect_parallel_lane_topology_host_pair_ids(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut host_pair_ids = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_host_pair_id(fingerprint))
        .collect::<Vec<_>>();
    host_pair_ids.sort();
    host_pair_ids
}

pub(crate) fn collect_parallel_lane_topology_id_host_pair_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_pair_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_lane_set_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_lane_set_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_lane_count_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_lane_count_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_mode_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_cardinality_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_mode_cardinality_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_pair_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_pair_cardinality_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}
