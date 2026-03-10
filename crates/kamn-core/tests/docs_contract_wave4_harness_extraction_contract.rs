use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/docs_contract_wave4_harness.rs";
const ROOT_CAP: usize = 180;
const MODULE_FILES: &[&str] = &[
    "tests/docs_contract_wave4_harness/transaction_guards_docs.rs",
    "tests/docs_contract_wave4_harness/instruction_verification_docs.rs",
    "tests/docs_contract_wave4_harness/agent_upgrade_workflow_docs.rs",
    "tests/docs_contract_wave4_harness/bridge_quorum_runtime_docs.rs",
    "tests/docs_contract_wave4_harness/performance_target_benchmarking_docs.rs",
    "tests/docs_contract_wave4_harness/task_swarm_dag_docs.rs",
    "tests/docs_contract_wave4_harness/bridge_adapter_docs.rs",
    "tests/docs_contract_wave4_harness/sdk_parity_fixture_docs.rs",
    "tests/docs_contract_wave4_harness/zk_message_proofs_docs.rs",
    "tests/docs_contract_wave4_harness/kolme_runtime_commit_extraction_plan_docs.rs",
    "tests/docs_contract_wave4_harness/block_pipeline_docs.rs",
    "tests/docs_contract_wave4_harness/a2a_mcp_interop_docs.rs",
    "tests/docs_contract_wave4_harness/key_lifecycle_audit_trails_docs.rs",
    "tests/docs_contract_wave4_harness/didcomm_compatibility_profile_docs.rs",
    "tests/docs_contract_wave4_harness/task_escrow_suite_discovery_parallel_contract.rs",
    "tests/docs_contract_wave4_harness/task_payment_workflow_docs.rs",
    "tests/docs_contract_wave4_harness/task_operations_docs.rs",
    "tests/docs_contract_wave4_harness/did_registry_transactions_docs.rs",
    "tests/docs_contract_wave4_harness/kolme_integration_roadmap_docs.rs",
    "tests/docs_contract_wave4_harness/escrow_lifecycle_docs.rs",
    "tests/docs_contract_wave4_harness/reputation_signal_routing_docs.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod transaction_guards_docs;",
    "mod instruction_verification_docs;",
    "mod agent_upgrade_workflow_docs;",
    "mod bridge_quorum_runtime_docs;",
    "mod performance_target_benchmarking_docs;",
    "mod task_swarm_dag_docs;",
    "mod bridge_adapter_docs;",
    "mod sdk_parity_fixture_docs;",
    "mod zk_message_proofs_docs;",
    "mod kolme_runtime_commit_extraction_plan_docs;",
    "mod block_pipeline_docs;",
    "mod a2a_mcp_interop_docs;",
    "mod key_lifecycle_audit_trails_docs;",
    "mod didcomm_compatibility_profile_docs;",
    "mod task_escrow_suite_discovery_parallel_contract;",
    "mod task_payment_workflow_docs;",
    "mod task_operations_docs;",
    "mod did_registry_transactions_docs;",
    "mod kolme_integration_roadmap_docs;",
    "mod escrow_lifecycle_docs;",
    "mod reputation_signal_routing_docs;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_transaction_guard_scope_and_components()",
    "fn doc_contains_instruction_verification_scope_and_checks()",
    "fn doc_contains_agent_upgrade_scope_and_models()",
    "fn doc_contains_bridge_quorum_scope_and_models()",
    "fn doc_contains_prd_13_2_thresholds()",
    "fn doc_contains_swarm_executor_and_graph_scope()",
    "fn doc_contains_bridge_adapter_scope_and_models()",
    "fn doc_contains_fixture_harness_scope_and_commands()",
    "fn doc_contains_zk_message_scope_and_invariants()",
    "fn doc_contains_runtime_commit_scope_and_phases()",
    "fn doc_contains_block_pipeline_scope_and_guards()",
    "fn doc_contains_scope_and_transport_models()",
    "fn doc_contains_key_lifecycle_scope_and_lane()",
    "fn doc_contains_profile_scope_and_commands()",
    "fn doc_contains_parallel_discovery_scope_and_inputs()",
    "fn doc_contains_scope_and_commands()",
    "fn doc_contains_task_ops_scope_and_rules()",
    "fn doc_contains_did_registry_scope_and_rules()",
    "fn doc_contains_roadmap_scope_and_prd_alignment()",
    "fn doc_contains_escrow_scope_and_state_machine()",
    "fn doc_contains_signal_integration_model()",
];

#[test]
fn docs_contract_wave4_harness_root_is_extracted() {
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
