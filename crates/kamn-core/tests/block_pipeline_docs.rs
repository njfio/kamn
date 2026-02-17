const DOC: &str = include_str!("../../../docs/architecture/block-pipeline.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn architecture_doc_contains_block_pipeline_core_components() {
    assert!(DOC.contains("MempoolBlockPipeline"));
    assert!(DOC.contains("BlockConsensusRoundInput"));
    assert!(DOC.contains("BlockPipelineCommitReport"));
    assert!(DOC.contains("BlockPipelineError"));
    assert!(DOC.contains("SqliteCanonicalCommitStore"));
    assert!(DOC.contains("build_transport_convergence_evidence_bundle(...)"));
    assert!(DOC.contains("UdpPeerLifecycleTransport"));
}

#[test]
fn architecture_doc_contains_consensus_and_runtime_wiring_contracts() {
    assert!(DOC.contains("ListenerQuorumEvaluator"));
    assert!(DOC.contains("ApproverQuorumEvaluator"));
    assert!(DOC.contains("RoleSmokeNetwork::produce_block"));
    assert!(DOC.contains("consensus-validator"));
}

#[test]
fn roadmap_references_phase_32_initial_block_pipeline_slice() {
    assert!(ROADMAP.contains("Phase 3.2 initial slice delivered"));
    assert!(ROADMAP.contains("Task #2926, Subtask #2927"));
    assert!(ROADMAP.contains("docs/architecture/block-pipeline.md"));
}

#[test]
fn docs_reference_phase_32_live_validation_lane_commands() {
    assert!(DOC.contains("scripts/runtime/validate_block_pipeline_live.sh"));
    assert!(DOC.contains("scripts/runtime/test_validate_block_pipeline_live.sh"));
    assert!(ROADMAP.contains("Phase 3.2 live validation delivered"));
    assert!(ROADMAP.contains("Task #2928, Subtask #2929"));
}

#[test]
fn roadmap_tracks_block_pipeline_live_validation_markers() {
    assert!(ROADMAP.contains("block_pipeline_contract_status=verified"));
    assert!(ROADMAP.contains("docs_contract_status=verified"));
    assert!(ROADMAP.contains("fail_closed_status=verified"));
    assert!(ROADMAP.contains("performance_budget_status=verified"));
}

#[test]
fn regression_doc_tracks_digest_mismatch_fail_closed_guard() {
    // Regression: #2927
    assert!(DOC.contains("Regression: #2927"));
    assert!(DOC.contains("fail_closed_reason_code=block_pipeline_payload_digest_mismatch"));
}

#[test]
fn regression_doc_tracks_sqlite_canonical_commit_store_fail_closed_markers() {
    // Regression: #3580
    assert!(DOC.contains("canonical_commit_store_sqlite_schema_mismatch"));
    assert!(DOC.contains("canonical_commit_store_sqlite_payload_not_utf8"));
    assert!(DOC.contains("canonical_commit_store_sqlite_key_height_mismatch"));
}

#[test]
fn regression_doc_tracks_transport_convergence_fault_matrix_markers() {
    // Regression: #3579
    assert!(DOC.contains("transport_convergence_case_id_missing"));
    assert!(DOC.contains("transport_convergence_commit_height_regression"));
    assert!(DOC.contains("block_pipeline_transport_convergence_faults"));
    assert!(DOC.contains("block_pipeline_transport_convergence_live_sockets"));
    assert!(DOC.contains("Regression: #3652"));
    assert!(DOC.contains("Regression: #3670"));
    assert!(DOC.contains("Regression: #4257"));
    assert!(DOC.contains("Regression: #4258"));
    assert!(DOC.contains("Regression: #4259"));
    assert!(DOC.contains("Regression: #4260"));
    assert!(DOC.contains("p2p_transport_live_socket_send_failed"));
    assert!(DOC.contains("validate_libp2p_convergence_process_isolated_live_contract_lane.sh"));
    assert!(DOC.contains("check_libp2p_convergence_process_isolated_live_evidence_convergence.sh"));
    assert!(DOC.contains("convergence_reason_codes=fork_choice_stale_block_height"));
    assert!(DOC.contains("finality_taxonomy_mapping_status=verified"));
    assert!(DOC.contains("runbook_marker_parity_status=verified"));
    assert!(DOC.contains(
        "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(DOC.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation"
    ));
    assert!(DOC.contains("libp2p_finality_evidence_convergence_status=verified"));
    assert!(DOC.contains(
        "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch"
    ));
}
