const DOC: &str = include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");
const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn doc_contains_adapter_types_and_validation_commands() {
    assert!(DOC.contains("AdapterBackedKolmeRuntimeCommitClient"));
    assert!(DOC.contains("KolmeRuntimeCommitProvider"));
    assert!(DOC.contains("KolmeRuntimeCommitProviderOutcome"));
    assert!(DOC.contains("KolmeRuntimeCommitProviderReceipt"));
    assert!(DOC.contains("KolmeRuntimeCommitProviderError"));
    assert!(DOC.contains("KolmeRuntimeCommitForkFinalityResolver"));
    assert!(DOC.contains("## Module Ownership Map"));
    assert!(DOC.contains("crates/kamn-kolme/src/runtime_transport_contracts.rs"));
    assert!(DOC.contains("runtime_lifecycle_policy"));
    assert!(DOC.contains("commit_finality_from_receipt_finality"));
    assert!(DOC.contains("parse_commit_receipt_finality"));
    assert!(DOC.contains("lifecycle_state_for_finality"));
    assert!(DOC.contains("lifecycle_state_label"));
    assert!(DOC.contains("commit_finality_label"));
    assert!(DOC.contains("runtime_request_identity_policy"));
    assert!(DOC.contains("deterministic_runtime_commit_idempotency_key"));
    assert!(DOC.contains("deterministic_runtime_commit_id"));
    assert!(DOC.contains("escape_json_string"));
    assert!(DOC.contains("translate_to_signed_broadcast_envelope"));
    assert!(DOC.contains("fetch_next_nonce"));
    assert!(DOC.contains("submit_broadcast_request"));
    assert!(DOC.contains("KolmeRuntimeCommitTransportErrorKind"));
    assert!(DOC.contains("http://` and `https://"));
    assert!(DOC.contains("KAMN_KOLME_TLS_CA_FILE"));
    assert!(DOC.contains("tls certificate verification failed"));
    assert!(DOC.contains("tls handshake failed"));
    assert!(DOC.contains("cargo test -p kamn-core --test kolme_runtime_commit_client"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test kolme_runtime_commit_http_transport integration_http_transport_fetch_next_nonce_query_and_parse -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test kolme_runtime_commit_http_transport integration_http_transport_submit_broadcast_request_put_and_parse_txhash -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test kolme_runtime_commit_http_transport regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test kolme_runtime_commit_http_transport functional_https_transport_submit_with_trusted_ca_succeeds -- --exact"
    ));
    assert!(
        DOC.contains("cargo test -p kamn-kolme --test runtime_commit_module_boundary_contracts")
    );
    assert!(DOC.contains("cargo test -p kamn-core --test kolme_runtime_commit_import_boundary"));
    assert!(DOC.contains("check_local_runtime_commit_live_evidence_policy.py"));
    assert!(DOC.contains("run_local_runtime_commit_live_finality_evidence_contract_lane.sh"));
    assert!(DOC.contains("submit_evidence_marker_present"));
    assert!(DOC.contains("finality_evidence_marker_present"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true"));
    assert!(DOC.contains("production_signer_key_source_env_local_forbidden"));
    assert!(DOC.contains("runtime_commit_managed_external_signer_public_key_marker_missing"));
    assert!(DOC.contains("managed_signer_public_key_marker_missing"));
    assert!(DOC.contains("managed_signer_public_key_marker_invalid"));
    assert!(DOC.contains("run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_runtime_commit_contract_lane.json --phase contract"));
    assert!(DOC.contains("run_local_kamn_live_runtime_integration_lane.sh --mode run"));
    assert!(DOC.contains("--runtime-commit-finality-command"));
    assert!(DOC.contains("--runtime-commit-live-policy-report"));
    assert!(DOC.contains("run_local_kolme_fork_process_lifecycle_lane.sh --mode run"));
    assert!(DOC.contains("--integration-runtime-commit-finality-command"));
    assert!(DOC.contains("run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_real_process_contract_lane.json --phase contract --mode run"));
    assert!(DOC.contains("--lifecycle-mode run"));
    assert!(DOC.contains("runtime_commit_decomposition_parity_matrix.json"));
    assert!(DOC.contains("check_runtime_commit_decomposition_parity_matrix.py"));
    assert!(DOC.contains("test_check_runtime_commit_decomposition_parity_matrix.sh"));
    assert!(DOC.contains("submit_http_round_trip"));
    assert!(DOC.contains("finality_block_fallback_resolution"));
    assert!(DOC.contains("receipt_provider_mismatch"));
    assert!(DOC.contains("receipt_not_final"));
}

#[test]
fn regression_requires_adapter_provider_mismatch_and_non_final_fail_closed_marker() {
    // Regression: #979
    assert!(DOC.contains("`Regression: #979`"));
    assert!(DOC.contains("provider mismatch/non-final receipts remain fail-closed"));
    assert!(DOC.contains("`Regression: #1471`"));
    assert!(DOC.contains("`Regression: #1502`"));
    assert!(DOC.contains("`Regression: #1503`"));
    assert!(DOC.contains("`Regression: #1506`"));
    assert!(DOC.contains("`Regression: #1533`"));
    assert!(DOC.contains("`Regression: #2095`"));
    assert!(DOC.contains("`Regression: #2099`"));
    assert!(DOC.contains("`Regression: #2101`"));
    assert!(DOC.contains("`Regression: #1775`"));
    assert!(DOC.contains("`Regression: #1777`"));
    assert!(DOC.contains("`Regression: #1779`"));
    assert!(DOC.contains("`Regression: #1781`"));
    assert!(DOC.contains("`Regression: #1783`"));
    assert!(DOC.contains("`Regression: #1979`"));
}

#[test]
fn doc_contains_nonce_retry_resilience_contract_and_live_lane_markers() {
    assert!(DOC.contains("## Nonce Retry Resilience Contract (Task #3042)"));
    assert!(DOC.contains("kolme.live.nonce.retry"));
    assert!(DOC.contains("scripts/runtime/validate_nonce_retry_live.sh"));
    assert!(DOC.contains("scripts/runtime/test_validate_nonce_retry_live.sh"));
    assert!(DOC.contains("nonce_retry_contract_status=verified"));
    assert!(DOC.contains("nonce_malformed_fail_closed_status=verified"));
}

#[test]
fn roadmap_tracks_post_roadmap_wave1_nonce_retry_live_validation() {
    assert!(ROADMAP.contains("Task #3042, Subtask #3043"));
    assert!(ROADMAP.contains("scripts/runtime/validate_nonce_retry_live.sh"));
    assert!(ROADMAP.contains("scripts/runtime/test_validate_nonce_retry_live.sh"));
    assert!(ROADMAP.contains("nonce_retry_contract_status=verified"));
    assert!(ROADMAP.contains("fail_closed_reason_code=nonce_response_malformed"));
}

#[test]
fn roadmap_tracks_post_roadmap_wave4_local_live_runtime_profile_matrix() {
    assert!(ROADMAP.contains("Story #3088"));
    assert!(ROADMAP.contains("Task #3102"));
    assert!(ROADMAP.contains("Subtask #3103"));
    assert!(ROADMAP.contains("Task #3104"));
    assert!(ROADMAP.contains("Subtask #3105"));
    assert!(ROADMAP.contains("run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"));
    assert!(ROADMAP.contains("test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"));
    assert!(ROADMAP.contains("test_check_local_kamn_live_runtime_real_node_profile_policy.sh"));
    assert!(ROADMAP.contains("runtime_commit_command_profile_mismatch"));
    assert!(ROADMAP.contains("runtime_signer_rotation_epoch_stale"));
    assert!(ROADMAP.contains("runtime_signer_key_source_profile_pair_disallowed"));
    assert!(ROADMAP.contains("transport.preflight.timeout"));
    assert!(ROADMAP.contains("transport.preflight.failed"));
}
