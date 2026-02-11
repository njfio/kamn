const DOC: &str = include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");

#[test]
fn doc_contains_adapter_types_and_validation_commands() {
    assert!(DOC.contains("AdapterBackedKolmeRuntimeCommitClient"));
    assert!(DOC.contains("KolmeRuntimeCommitProvider"));
    assert!(DOC.contains("KolmeRuntimeCommitProviderOutcome"));
    assert!(DOC.contains("KolmeRuntimeCommitProviderReceipt"));
    assert!(DOC.contains("KolmeRuntimeCommitProviderError"));
    assert!(DOC.contains("KolmeRuntimeCommitForkFinalityResolver"));
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
    assert!(DOC.contains("scripts/kolme/run_runtime_commit_contract_lane.sh"));
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
}
