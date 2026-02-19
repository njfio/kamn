const DOC: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");

#[test]
fn doc_contains_transport_retry_validation_contract_markers() {
    assert!(DOC.contains("transport_retry_validation_contract_version=v1"));
    assert!(DOC.contains("kolme.live.submit.retry"));
    assert!(DOC.contains("kolme.live.finality.retry"));
    assert!(DOC.contains("kolme.live.submit.retry.terminal"));
    assert!(DOC.contains("kolme.live.finality.retry.terminal"));
    assert!(DOC.contains("terminal_decision=attempt_ceiling_reached"));
    assert!(DOC.contains("terminal_decision=malformed_response_fail_fast"));
}

#[test]
fn doc_contains_transport_retry_validation_commands() {
    assert!(DOC.contains(
        "main_tests::runtime_tests::functional_kolme_live_retry_emits_structured_retry_markers"
    ));
    assert!(DOC.contains(
        "main_tests::runtime_tests::regression_runtime_kolme_live_submit_retry_exhaustion_emits_terminal_decision_marker"
    ));
    assert!(DOC.contains(
        "main_tests::runtime_tests::functional_kolme_live_finality_retry_exhaustion_emits_terminal_decision_marker"
    ));
}
