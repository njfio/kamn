const DOC: &str = include_str!("../../../docs/ops/configuration.md");
#[path = "service_api_ops_configuration_docs/shared_support.rs"]
mod shared_support;

#[path = "service_api_ops_configuration_docs/dependency_supply_chain_contract_tests.rs"]
mod dependency_supply_chain_contract_tests;
#[path = "service_api_ops_configuration_docs/compatibility_resilience_contract_tests.rs"]
mod compatibility_resilience_contract_tests;
#[path = "service_api_ops_configuration_docs/phase6_runtime_contract_tests.rs"]
mod phase6_runtime_contract_tests;
#[path = "service_api_ops_configuration_docs/live_postgres_matrix_contract_tests.rs"]
mod live_postgres_matrix_contract_tests;
#[path = "service_api_ops_configuration_docs/guardrail_signer_contract_tests.rs"]
mod guardrail_signer_contract_tests;
#[path = "service_api_ops_configuration_docs/reconciliation_upgrade_contract_tests.rs"]
mod reconciliation_upgrade_contract_tests;
