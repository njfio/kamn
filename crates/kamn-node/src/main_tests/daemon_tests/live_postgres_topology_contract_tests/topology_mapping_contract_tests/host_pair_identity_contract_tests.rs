use super::support::*;
use super::*;

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_contract_is_canonical(
) {
    assert_host_pair_contract_metadata();
    assert_host_pair_sample_row();
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pairs_are_stable(
) {
    with_live_postgres_validation_database_url(|database_url| {
        apply_live_postgres_validation_migrations(database_url);
        let baseline_host_pair_ids = host_pair_ids("baseline");
        assert_host_pair_baseline(&baseline_host_pair_ids);
        assert_rows_stable_across_permutations(
            &baseline_host_pair_ids,
            host_pair_ids,
            "topology host-pair ids should remain stable",
        );
    });
}

fn assert_host_pair_contract_metadata() {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV,
        ["node_alpha->node_alpha", "node_alpha->node_beta"].join(",")
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_CONTRACT,
        "host_pair_ids_must_remain_stable_under_repeated_runs_and_topology_permutations"
    );
}

fn assert_host_pair_sample_row() {
    let topology_fingerprint = sample_topology_fingerprint(
        "same_host_parallel",
        "node_alpha",
        "node_alpha",
        "processor_listener_parallel_applied",
    );
    assert_eq!(
        extract_parallel_lane_topology_host_pair_id(&topology_fingerprint),
        "node_alpha->node_alpha"
    );
}

fn host_pair_ids(permutation: &str) -> Vec<String> {
    collect_parallel_lane_topology_host_pair_ids(permuted_topology_profiles(permutation))
}

fn assert_host_pair_baseline(baseline_host_pair_ids: &[String]) {
    assert_eq!(
        baseline_host_pair_ids,
        [
            "node_alpha->node_alpha".to_owned(),
            "node_alpha->node_beta".to_owned()
        ]
    );
    assert_eq!(
        baseline_host_pair_ids.join(","),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_REQUIRED_HOST_PAIR_IDS_CSV
    );
}
