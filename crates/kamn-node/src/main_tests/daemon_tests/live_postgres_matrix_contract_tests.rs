// daemon live-postgres matrix contracts decomposition: route gate, taxonomy, profile,
// role-pair, and parallel-lane assertions through bounded include modules.
include!("live_postgres_matrix_contract_tests/support.rs");
include!("live_postgres_matrix_contract_tests/env_gate_execution_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/projection_taxonomy_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/load_profile_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/role_profile_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/role_pair_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/parallel_role_pair_lane_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/asymmetric_parallel_lane_contract_tests.rs");
include!("live_postgres_matrix_contract_tests/parallel_lane_invariance_contract_tests.rs");
