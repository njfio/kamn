use super::super::models::*;
use super::row_extraction_bundle_rows::*;
use super::row_extraction_host_rows::*;
use super::runner_support::*;
pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_id_host_mode_host_pair_row(fingerprint))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_cardinality_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_cardinality_row(fingerprint)
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut rows = run_parallel_lane_topology_fingerprints(topology_profiles)
        .iter()
        .map(|fingerprint| {
            extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
                fingerprint,
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

pub(crate) fn assert_parallel_lane_topology_rows_are_canonically_sorted(
    rows: &[String],
    context: &str,
) {
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort();
    assert_eq!(
        rows, sorted_rows,
        "{context} should remain lexicographically canonical after order-normalization"
    );
}

pub(crate) fn collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_rows(
            topology_profiles,
        );
    assert_parallel_lane_topology_rows_are_canonically_sorted(
        &rows,
        "topology-id to host-mode-host-pair-lane-set-lane-fingerprint-hash rows",
    );
    rows
}

pub(crate) fn project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> (Vec<String>, String) {
    let rows =
        collect_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalized_rows(
            topology_profiles,
        );
    let digest = deterministic_fnv1a64_hex(&rows.join(","));
    (rows, digest)
}
