use super::support::*;
use super::*;

#[test]
fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical(
) {
    assert_host_pair_directionality_contract_metadata();
    assert_host_pair_directionality_sample_row();
}

#[test]
fn integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_is_stable(
) {
    with_live_postgres_validation_database_url(|database_url| {
        apply_live_postgres_validation_migrations(database_url);
        let baseline_canonical_ids = sorted_canonical_host_pair_ids("baseline");
        assert_host_pair_directionality_baseline(&baseline_canonical_ids);
        let forbidden_reverse_pairs = forbidden_reverse_pairs();
        assert_host_pair_directionality_permutations(
            &baseline_canonical_ids,
            &forbidden_reverse_pairs,
        );
    });
}

fn assert_host_pair_directionality_contract_metadata() {
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_SCHEMA_VERSION,
        "kamn.runtime.daemon.phase6-live-postgres.parallel-lane-topology-host-pair-directionality.v1"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_EXTRACTION_RULE,
        "host_a_to_host_b_arrow_notation_non_commutative"
    );
    assert_eq!(
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV,
        "node_beta->node_alpha"
    );
}

fn assert_host_pair_directionality_sample_row() {
    let topology_fingerprint = sample_topology_fingerprint(
        "distributed_label_parallel",
        "node_alpha",
        "node_beta",
        "listener_approver_asymmetric_parallel_applied",
    );
    let canonical = extract_parallel_lane_topology_host_pair_id(&topology_fingerprint);
    let reverse = extract_parallel_lane_topology_host_pair_reverse_id(&topology_fingerprint);
    assert_eq!(canonical, "node_alpha->node_beta");
    assert_eq!(reverse, "node_beta->node_alpha");
    assert_ne!(
        canonical, reverse,
        "host-pair extraction should remain non-commutative"
    );
}

fn sorted_canonical_host_pair_ids(permutation: &str) -> Vec<String> {
    let mut canonical_ids = topology_fingerprints(permutation)
        .iter()
        .map(|fingerprint| extract_parallel_lane_topology_host_pair_id(fingerprint))
        .collect::<Vec<_>>();
    canonical_ids.sort();
    canonical_ids
}

fn topology_fingerprints(permutation: &str) -> Vec<String> {
    run_parallel_lane_topology_fingerprints(permuted_topology_profiles(permutation))
}

fn forbidden_reverse_pairs() -> Vec<&'static str> {
    LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_HOST_PAIR_DIRECTIONALITY_FORBIDDEN_REVERSE_PAIRS_CSV
        .split(',')
        .collect()
}

fn assert_host_pair_directionality_baseline(baseline_canonical_ids: &[String]) {
    assert_eq!(
        baseline_canonical_ids,
        [
            "node_alpha->node_alpha".to_owned(),
            "node_alpha->node_beta".to_owned()
        ]
    );
}

fn assert_host_pair_directionality_permutations(
    baseline_canonical_ids: &[String],
    forbidden_reverse_pairs: &[&str],
) {
    for permutation in ["reverse", "rotate_left_1"] {
        let permuted_topology_fingerprints = topology_fingerprints(permutation);
        assert_eq!(
            baseline_canonical_ids,
            sorted_canonical_host_pair_ids(permutation),
            "canonical host-pair ids should remain stable under permutation {permutation}"
        );
        assert_reverse_pairs_forbidden(&permuted_topology_fingerprints, forbidden_reverse_pairs);
    }
}

fn assert_reverse_pairs_forbidden(
    topology_fingerprints: &[String],
    forbidden_reverse_pairs: &[&str],
) {
    for topology_fingerprint in topology_fingerprints {
        let reverse_id = extract_parallel_lane_topology_host_pair_reverse_id(topology_fingerprint);
        assert!(
            !forbidden_reverse_pairs.contains(&reverse_id.as_str()),
            "reverse host-pair id {reverse_id} must remain forbidden under directionality contract"
        );
    }
}
