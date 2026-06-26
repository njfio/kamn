const CHECKLIST: &str = include_str!("../../../docs/foundation/release-gonogo-checklist.md");

#[path = "release_gonogo_checklist_docs/compatibility_failover_contract_tests.rs"]
mod compatibility_failover_contract_tests;
#[path = "release_gonogo_checklist_docs/durable_compliance_contract_tests.rs"]
mod durable_compliance_contract_tests;
#[path = "release_gonogo_checklist_docs/governance_launch_contract_tests.rs"]
mod governance_launch_contract_tests;
#[path = "release_gonogo_checklist_docs/integrity_evidence_contract_tests.rs"]
mod integrity_evidence_contract_tests;
#[path = "release_gonogo_checklist_docs/preflight_contract_tests.rs"]
mod preflight_contract_tests;
#[path = "release_gonogo_checklist_docs/promotion_lineage_contract_tests.rs"]
mod promotion_lineage_contract_tests;
#[path = "release_gonogo_checklist_docs/regression_governance_launch_contract_tests.rs"]
mod regression_governance_launch_contract_tests;
#[path = "release_gonogo_checklist_docs/regression_release_ops_contract_tests.rs"]
mod regression_release_ops_contract_tests;
#[path = "release_gonogo_checklist_docs/runtime_policy_contract_tests.rs"]
mod runtime_policy_contract_tests;
#[path = "release_gonogo_checklist_docs/runtime_reconciliation_contract_tests.rs"]
mod runtime_reconciliation_contract_tests;
#[path = "release_gonogo_checklist_docs/service_api_contract_tests.rs"]
mod service_api_contract_tests;
#[path = "release_gonogo_checklist_docs/support.rs"]
mod support;
