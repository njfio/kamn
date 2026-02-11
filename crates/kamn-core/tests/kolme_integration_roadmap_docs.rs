const ROADMAP: &str = include_str!("../../../docs/planning/kolme-integration-roadmap.md");

#[test]
fn roadmap_contains_version_and_runtime_commit_contract_lane_commands() {
    assert!(ROADMAP.contains("validate_version_compatibility.py"));
    assert!(ROADMAP.contains("generate_fork_compatibility_evidence.py"));
    assert!(ROADMAP.contains("check_fork_compatibility_policy.py"));
    assert!(ROADMAP.contains("run_version_compatibility_contract_lane.sh"));
    assert!(ROADMAP.contains("run_runtime_commit_contract_lane.sh"));
    assert!(ROADMAP.contains("docs/foundation/kolme-runtime-commit-client.md"));
    assert!(ROADMAP.contains("check_runtime_commit_replay_policy.py"));
    assert!(ROADMAP.contains("run_runtime_commit_replay_tamper_matrix.py"));
    assert!(ROADMAP.contains("run_runtime_commit_replay_contract_lane.sh"));
    assert!(ROADMAP.contains("run_runtime_commit_adapter_contract_lane.sh"));
    assert!(ROADMAP.contains("kolme_runtime_commit_fork_finality_resolver"));
    assert!(ROADMAP.contains("run_local_kolme_live_api_conformance_contract_lane.sh"));
    assert!(ROADMAP.contains("unit_runtime_commit_signed_translation_rejects_message_mismatch"));
    assert!(ROADMAP.contains("integration_kolme_fork_signed_envelope_submit_maps_txhash_response"));
    assert!(ROADMAP.contains("check_nonce_broadcast_parity_policy.py"));
    assert!(ROADMAP.contains("run_nonce_broadcast_parity_matrix.py"));
    assert!(ROADMAP.contains("run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(ROADMAP.contains("run_notifications_consumer_contract_lane.sh"));
    assert!(ROADMAP.contains("run_block_fallback_reconciliation_contract_lane.sh"));
    assert!(ROADMAP.contains("kolme_runtime_commit_notifications"));
    assert!(ROADMAP.contains("kolme_runtime_commit_block_fallback"));
    assert!(ROADMAP.contains("fixtures/kolme_compatibility/fork_compatibility_cases.json"));
    assert!(ROADMAP.contains("fixtures/kolme_commit/runtime_commit_request_cases.txt"));
    assert!(ROADMAP.contains("fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json"));
    assert!(ROADMAP.contains("fixtures/kolme_commit/nonce_broadcast_parity_cases.json"));
    assert!(ROADMAP.contains("fixtures/kolme_commit/local_live_api_conformance_matrix.json"));
}

#[test]
fn regression_guards_include_legacy_and_runtime_commit_markers() {
    assert!(ROADMAP.contains("`Regression: #775`"));
    assert!(ROADMAP.contains("`Regression: #825`"));
    assert!(ROADMAP.contains("`Regression: #826`"));
    assert!(ROADMAP.contains("`Regression: #827`"));
    assert!(ROADMAP.contains("`Regression: #979`"));
    assert!(ROADMAP.contains("`Regression: #980`"));
    assert!(ROADMAP.contains("`Regression: #1502`"));
    assert!(ROADMAP.contains("`Regression: #1503`"));
    assert!(ROADMAP.contains("`Regression: #1504`"));
    assert!(ROADMAP.contains("`Regression: #1506`"));
    assert!(ROADMAP.contains("`Regression: #1401`"));
    assert!(ROADMAP.contains("`Regression: #1402`"));
    assert!(ROADMAP.contains("`Regression: #1462`"));
    assert!(ROADMAP.contains("`Regression: #1463`"));
    assert!(ROADMAP.contains("`Regression: #1464`"));
}
