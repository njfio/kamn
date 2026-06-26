// include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_cardinality_contract_tests.rs");
#[path = "topology_coherence_contract_tests/host_mode_cardinality_contract_tests.rs"]
mod host_mode_cardinality_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_pair_cardinality_contract_tests.rs");
#[path = "topology_coherence_contract_tests/host_pair_cardinality_contract_tests.rs"]
mod host_pair_cardinality_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_host_pair_contract_tests.rs");
#[path = "topology_coherence_contract_tests/host_mode_host_pair_contract_tests.rs"]
mod host_mode_host_pair_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests/lane_set_bundle_contract_tests.rs");
#[path = "topology_coherence_contract_tests/lane_set_bundle_contract_tests.rs"]
mod lane_set_bundle_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests/fingerprint_hash_coherence_contract_tests.rs");
#[path = "topology_coherence_contract_tests/fingerprint_hash_coherence_contract_tests.rs"]
mod fingerprint_hash_coherence_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests/hash_normalization_contract_tests.rs");
#[path = "topology_coherence_contract_tests/hash_normalization_contract_tests.rs"]
mod hash_normalization_contract_tests;

fn assert_topology_metadata(schema_version: &str, contract: &str) {
    assert!(schema_version.starts_with("kamn.runtime.daemon.phase6-live-postgres."));
    assert!(!contract.trim().is_empty());
}

fn assert_topology_rows_match(mut rows: Vec<String>, rows_csv: &str) {
    let mut expected = rows_csv
        .split(',')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    rows.sort();
    expected.sort();
    assert_eq!(rows, expected);
}
