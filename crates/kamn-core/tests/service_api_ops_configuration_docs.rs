const DOC: &str = include_str!("../../../docs/ops/configuration.md");

fn assert_doc_contains_all(markers: &[&str]) {
    for marker in markers {
        assert!(DOC.contains(marker), "missing doc marker: {marker}");
    }
}

fn assert_doc_contains_prefixed_entries(prefix: &str, codes: &[&str]) {
    for code in codes {
        let marker = format!("{prefix}.{code}=");
        assert!(DOC.contains(&marker), "missing doc marker: {marker}");
    }
}

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
