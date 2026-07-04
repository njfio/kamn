use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/release_gonogo_checklist_docs.rs";
const ROOT_CAP: usize = 80;
const MODULE_FILES: &[&str] = &[
    "tests/release_gonogo_checklist_docs/preflight_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/service_api_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/runtime_reconciliation_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/runtime_policy_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/integrity_evidence_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/promotion_lineage_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/durable_compliance_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/compatibility_failover_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/governance_launch_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/regression_release_ops_contract_tests.rs",
    "tests/release_gonogo_checklist_docs/regression_governance_launch_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod preflight_contract_tests;",
    "mod service_api_contract_tests;",
    "mod runtime_reconciliation_contract_tests;",
    "mod runtime_policy_contract_tests;",
    "mod integrity_evidence_contract_tests;",
    "mod promotion_lineage_contract_tests;",
    "mod durable_compliance_contract_tests;",
    "mod compatibility_failover_contract_tests;",
    "mod governance_launch_contract_tests;",
    "mod regression_release_ops_contract_tests;",
    "mod regression_governance_launch_contract_tests;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn checklist_contains_preflight_gates()",
    "fn checklist_contains_service_api_protocol_session_reason_mapping_gate()",
    "fn checklist_contains_block_reconciliation_partition_healing_mismatch_mapping_gate()",
    "fn checklist_contains_panic_replacement_reason_taxonomy_and_runtime_evidence_gate()",
    "fn checklist_contains_tls_evidence_completeness_freshness_gate()",
    "fn checklist_contains_live_gonogo_convergence_boundary_governance_gate()",
    "fn checklist_contains_staging_rehearsal_contract()",
    "fn checklist_contains_kolme_version_compatibility_replay_evidence_contract()",
    "fn checklist_contains_governance_simulation_and_human_veto_evidence_contract()",
    "fn regression_requires_rollback_precheck_in_checklist()",
    "fn regression_requires_governance_simulation_and_veto_guard_marker()",
];

#[test]
fn release_gonogo_checklist_docs_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
    }
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "moved test marker still present: {marker}"
        );
    }
    for path in MODULE_FILES {
        let full = repo_path(path);
        assert!(
            full.exists(),
            "missing extracted module: {}",
            full.display()
        );
        assert!(
            fs::read_to_string(&full)
                .expect("read module")
                .lines()
                .count()
                <= 200,
            "extracted module exceeds 200 lines: {}",
            full.display()
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
