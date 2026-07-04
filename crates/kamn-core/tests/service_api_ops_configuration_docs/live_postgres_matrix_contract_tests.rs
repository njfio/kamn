use super::shared_support::*;
use super::*;

#[path = "live_postgres_matrix_contract_tests/distributed_execution_contract_tests.rs"]
mod distributed_execution_contract_tests;
#[path = "live_postgres_matrix_contract_tests/runtime_gate_contract_tests.rs"]
mod runtime_gate_contract_tests;
#[path = "live_postgres_matrix_contract_tests/topology_coherence_contract_tests.rs"]
mod topology_coherence_contract_tests;
#[path = "live_postgres_matrix_contract_tests/topology_mapping_contract_tests.rs"]
mod topology_mapping_contract_tests;
#[path = "live_postgres_matrix_contract_tests/topology_scope_contract_tests.rs"]
mod topology_scope_contract_tests;
