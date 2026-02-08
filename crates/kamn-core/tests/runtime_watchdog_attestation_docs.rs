const DOC: &str = include_str!("../../../docs/foundation/runtime-watchdog-attestation.md");

#[test]
fn doc_contains_watchdog_attestation_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("## Runtime Watchdog Attestation Models"));
    assert!(DOC.contains("RuntimeWatchdogAttestation"));
    assert!(DOC.contains("WatchdogSeverity"));
    assert!(DOC.contains("WatchdogIncidentEvidence"));
    assert!(DOC.contains("WatchdogIncidentRecord"));
}

#[test]
fn doc_contains_incident_response_mapping_and_fast_lane() {
    assert!(DOC.contains("## Incident Response Mapping"));
    assert!(DOC.contains("`docs/foundation/upgrade-rollback-runbook.md`"));
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_watchdog_attestation_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test watchdog_node_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test upgrade_rollback_runbook_docs"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_divergence_and_censorship_guard_rules() {
    // Regression: #383
    assert!(DOC
        .contains("state-hash divergence attestation must include expected and observed hashes."));
    assert!(
        DOC.contains("single-recipient deliveries are excluded from censorship classification.")
    );
    assert!(DOC.contains(
        "attestation replay for the same incident fingerprint is rejected (`Regression: #383`)."
    ));
}
