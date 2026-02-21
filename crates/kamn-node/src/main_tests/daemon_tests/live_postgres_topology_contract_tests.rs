// daemon live-postgres topology contracts decomposition: route topology/fingerprint/matrix/regression
// suites through bounded include modules while preserving deterministic test names and ordering.
include!("live_postgres_topology_contract_tests/fingerprint_and_topology_scope_tests.rs");
include!("live_postgres_topology_contract_tests/topology_mapping_contract_tests.rs");
include!("live_postgres_topology_contract_tests/topology_coherence_contract_tests.rs");
include!("live_postgres_topology_contract_tests/matrix_and_regression_contract_tests.rs");
