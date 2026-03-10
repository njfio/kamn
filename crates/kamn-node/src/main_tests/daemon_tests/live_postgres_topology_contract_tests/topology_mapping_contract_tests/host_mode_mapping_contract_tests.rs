use super::support::*;
use super::*;

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_mapping_contract_is_canonical(
) {
    assert_host_mode_mapping_contract_metadata();
    assert_host_mode_mapping_sample_row();
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_mapping_is_stable(
) {
    with_live_postgres_validation_database_url(|database_url| {
        apply_live_postgres_validation_migrations(database_url);
        let baseline_rows = host_mode_mapping_rows("baseline");
        assert_host_mode_mapping_baseline(&baseline_rows);
        assert_rows_stable_across_permutations(
            &baseline_rows,
            host_mode_mapping_rows,
            "topology-id to host-mode rows should remain stable",
        );
    });
}

fn assert_host_mode_mapping_contract_metadata() {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-mode-mapping.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_ROWS_CSV,
        "same_host_parallel->same_host,distributed_label_parallel->distributed_label"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_CONTRACT,
        "topology_id_to_host_mode_rows_must_remain_stable_under_repeated_runs_and_permutations"
    );
}

fn assert_host_mode_mapping_sample_row() {
    let topology_fingerprint = sample_topology_fingerprint(
        "same_host_parallel",
        "node_alpha",
        "node_alpha",
        "processor_listener_parallel_applied",
    );
    assert_eq!(
        extract_parallel_lane_topology_id_host_mode_row(&topology_fingerprint),
        "same_host_parallel->same_host"
    );
}

fn host_mode_mapping_rows(permutation: &str) -> Vec<String> {
    collect_parallel_lane_topology_id_host_mode_rows(permuted_topology_profiles(permutation))
}

fn assert_host_mode_mapping_baseline(baseline_rows: &[String]) {
    assert_eq!(
        baseline_rows,
        [
            "distributed_label_parallel->distributed_label".to_owned(),
            "same_host_parallel->same_host".to_owned(),
        ]
    );
    assert_eq!(
        baseline_rows.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_MODE_MAPPING_ROWS_CSV
    );
}
