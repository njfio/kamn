const DOC: &str = include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");

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
    assert!(DOC.contains("managed_signer_public_key_marker_missing"));
    assert!(DOC.contains("managed_signer_public_key_marker_invalid"));
    assert!(DOC.contains("scripts/kolme/run_runtime_commit_contract_lane.sh"));
    assert!(DOC.contains("run_local_kamn_live_runtime_integration_lane.sh --mode run"));
    assert!(DOC.contains("--runtime-commit-finality-command"));
    assert!(DOC.contains("--runtime-commit-live-policy-report"));
    assert!(DOC.contains("run_local_kolme_fork_process_lifecycle_lane.sh --mode run"));
    assert!(DOC.contains("--integration-runtime-commit-finality-command"));
    assert!(DOC.contains("run_local_kolme_fork_real_process_contract_lane.sh --mode run"));
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
