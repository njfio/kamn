use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/docs_contract_wave3_harness.rs";
const ROOT_CAP: usize = 220;
const MODULE_FILES: &[&str] = &[
    "tests/docs_contract_wave3_harness/invariant_fuzz_strategy_docs.rs",
    "tests/docs_contract_wave3_harness/docs_contract_template_guidance_contract.rs",
    "tests/docs_contract_wave3_harness/onchain_lifecycle_evidence_docs.rs",
    "tests/docs_contract_wave3_harness/discord_bridge_approver_docs.rs",
    "tests/docs_contract_wave3_harness/openclaw_connector_docs.rs",
    "tests/docs_contract_wave3_harness/anti_spam_controls_docs.rs",
    "tests/docs_contract_wave3_harness/ci_caching_parallelism_docs.rs",
    "tests/docs_contract_wave3_harness/message_delivery_guards_docs.rs",
    "tests/docs_contract_wave3_harness/channel_permissions_retention_docs.rs",
    "tests/docs_contract_wave3_harness/watchdog_node_docs.rs",
    "tests/docs_contract_wave3_harness/agent_interop_wave_docs.rs",
    "tests/docs_contract_wave3_harness/token_config_docs.rs",
    "tests/docs_contract_wave3_harness/runtime_module_extraction_roadmap_docs.rs",
    "tests/docs_contract_wave3_harness/upgrade_orchestration_docs.rs",
    "tests/docs_contract_wave3_harness/content_retention_tombstones_docs.rs",
    "tests/docs_contract_wave3_harness/operator_permissioned_actions_docs.rs",
    "tests/docs_contract_wave3_harness/content_replication_repair_docs.rs",
    "tests/docs_contract_wave3_harness/content_retrieval_access_cache_docs.rs",
    "tests/docs_contract_wave3_harness/content_storage_adapter_docs.rs",
    "tests/docs_contract_wave3_harness/task_escrow_suite_modularization_contract.rs",
    "tests/docs_contract_wave3_harness/kolme_runtime_architecture_docs.rs",
    "tests/docs_contract_wave3_harness/failover_runbook_docs.rs",
    "tests/docs_contract_wave3_harness/secure_coding_docs.rs",
    "tests/docs_contract_wave3_harness/channel_models_and_permissions_docs.rs",
    "tests/docs_contract_wave3_harness/kolme_fork_api_contract_inventory_docs.rs",
    "tests/docs_contract_wave3_harness/kolme_live_integration_docs.rs",
    "tests/docs_contract_wave3_harness/task_state_machine_docs.rs",
    "tests/docs_contract_wave3_harness/telegram_bridge_listener_docs.rs",
    "tests/docs_contract_wave3_harness/rehearsal_rollback_governance_docs.rs",
    "tests/docs_contract_wave3_harness/message_proof_anchoring_docs.rs",
    "tests/docs_contract_wave3_harness/channel_models_docs.rs",
    "tests/docs_contract_wave3_harness/invariants_docs.rs",
    "tests/docs_contract_wave3_harness/versioning_compatibility_docs.rs",
    "tests/docs_contract_wave3_harness/cross_chain_bridge_adapters_docs.rs",
    "tests/docs_contract_wave3_harness/redaction_tombstones_docs.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod invariant_fuzz_strategy_docs;",
    "mod docs_contract_template_guidance_contract;",
    "mod onchain_lifecycle_evidence_docs;",
    "mod discord_bridge_approver_docs;",
    "mod openclaw_connector_docs;",
    "mod anti_spam_controls_docs;",
    "mod ci_caching_parallelism_docs;",
    "mod message_delivery_guards_docs;",
    "mod channel_permissions_retention_docs;",
    "mod watchdog_node_docs;",
    "mod agent_interop_wave_docs;",
    "mod token_config_docs;",
    "mod runtime_module_extraction_roadmap_docs;",
    "mod upgrade_orchestration_docs;",
    "mod content_retention_tombstones_docs;",
    "mod operator_permissioned_actions_docs;",
    "mod content_replication_repair_docs;",
    "mod content_retrieval_access_cache_docs;",
    "mod content_storage_adapter_docs;",
    "mod task_escrow_suite_modularization_contract;",
    "mod kolme_runtime_architecture_docs;",
    "mod failover_runbook_docs;",
    "mod secure_coding_docs;",
    "mod channel_models_and_permissions_docs;",
    "mod kolme_fork_api_contract_inventory_docs;",
    "mod kolme_live_integration_docs;",
    "mod task_state_machine_docs;",
    "mod telegram_bridge_listener_docs;",
    "mod rehearsal_rollback_governance_docs;",
    "mod message_proof_anchoring_docs;",
    "mod channel_models_docs;",
    "mod invariants_docs;",
    "mod versioning_compatibility_docs;",
    "mod cross_chain_bridge_adapters_docs;",
    "mod redaction_tombstones_docs;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn invariant_strategy_docs_pin_transition_proptest_suite_sources()",
    "fn conformance_subtask_template_contains_docs_contract_migration_checklist_markers()",
    "fn docs_include_onchain_lifecycle_evidence_bundle_contract_markers()",
    "fn doc_contains_discord_bridge_scope_and_quorum_contracts()",
    "fn doc_contains_connector_contract_and_workflow_steps()",
    "fn doc_contains_enforcement_rules_and_telemetry()",
    "fn doc_contains_selector_scope_and_cache_guidance()",
    "fn doc_contains_delivery_guard_scope_and_validation_rules()",
    "fn doc_contains_channel_permissions_scope_and_models()",
    "fn doc_contains_watchdog_scope_and_detection_rules()",
    "fn doc_contains_did_lifecycle_contract_lane_commands()",
    "fn doc_contains_token_launch_handoff_evidence_contract()",
    "fn roadmap_tracks_runtime_decomposition_tranche4_snapshot_module_extraction()",
    "fn doc_contains_upgrade_orchestrator_scope_and_models()",
    "fn doc_contains_retention_class_and_lifecycle_scope()",
    "fn doc_contains_scope_and_service_contracts()",
    "fn doc_contains_policy_and_repair_scope()",
    "fn doc_contains_retrieval_scope_and_engine_contract()",
    "fn doc_contains_adapter_contract_scope_and_helpers()",
    "fn root_harness_declares_task_and_escrow_domain_modules()",
    "fn architecture_doc_contains_runtime_flow_and_signer_boundaries()",
    "fn runbook_contains_topology_section()",
    "fn doc_contains_panic_path_reachability_and_unsafe_fallback_markers()",
    "fn doc_contains_channel_models_and_permissions_scope()",
    "fn inventory_captures_kolme_fork_base_api_routes_and_kamn_expectations()",
    "fn doc_contains_process_isolation_marker_contracts()",
    "fn doc_contains_task_lifecycle_scope_and_transition_map()",
    "fn doc_contains_telegram_bridge_scope_and_listener_contracts()",
    "fn plan_contains_r27_19_rehearsal_rollback_ci_smoke_closure_markers()",
    "fn doc_contains_anchor_service_contracts()",
    "fn doc_contains_core_channel_models_and_operations()",
    "fn doc_contains_runtime_invariant_harness_coverage_contract()",
    "fn policy_defines_semantic_versions_for_chain_app_and_sdks()",
    "fn doc_covers_adapter_and_receipt_normalization_scopes()",
    "fn doc_contains_redaction_scope_and_validation()",
];

#[test]
fn docs_contract_wave3_harness_root_is_extracted() {
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
