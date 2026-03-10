// daemon live-postgres topology mapping decomposition: keep contract-expected include markers
// as comments while compiling bounded sibling modules via `#[path = ...]`.
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_pair_identity_contract_tests.rs");
#[path = "topology_mapping_contract_tests/host_pair_identity_contract_tests.rs"]
mod host_pair_identity_contract_tests;
#[path = "topology_mapping_contract_tests/support.rs"]
mod support;
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_pair_directionality_contract_tests.rs");
#[path = "topology_mapping_contract_tests/host_pair_directionality_contract_tests.rs"]
mod host_pair_directionality_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_pair_mapping_contract_tests.rs");
#[path = "topology_mapping_contract_tests/host_pair_mapping_contract_tests.rs"]
mod host_pair_mapping_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/lane_set_mapping_contract_tests.rs");
#[path = "topology_mapping_contract_tests/lane_set_mapping_contract_tests.rs"]
mod lane_set_mapping_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/lane_count_mapping_contract_tests.rs");
#[path = "topology_mapping_contract_tests/lane_count_mapping_contract_tests.rs"]
mod lane_count_mapping_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_mode_mapping_contract_tests.rs");
#[path = "topology_mapping_contract_tests/host_mode_mapping_contract_tests.rs"]
mod host_mode_mapping_contract_tests;
// include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests/host_cardinality_mapping_contract_tests.rs");
#[path = "topology_mapping_contract_tests/host_cardinality_mapping_contract_tests.rs"]
mod host_cardinality_mapping_contract_tests;
