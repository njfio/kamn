const DOC: &str = include_str!("../../../docs/foundation/runtime-watchdog-attestation.md");

#[test]
fn doc_contains_watchdog_attestation_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("## Runtime Watchdog Attestation Models"));
    assert!(DOC.contains("RuntimeWatchdogAttestation"));
    assert!(DOC.contains("WatchdogSeverity"));
    assert!(DOC.contains("WatchdogIncidentEvidence"));
    assert!(DOC.contains("WatchdogIncidentRecord"));
    assert!(DOC.contains("StateDivergenceWatchInput"));
    assert!(DOC.contains("StateDivergenceEvaluator"));
    assert!(DOC.contains("StateDivergenceReport"));
    assert!(DOC.contains("StateDivergenceError"));
    assert!(DOC.contains("evaluate_daemon_state_divergence"));
    assert!(DOC.contains("WatchdogAnomalyWatchInput"));
    assert!(DOC.contains("WatchdogAnomalyEvaluator"));
    assert!(DOC.contains("WatchdogAnomalyReport"));
    assert!(DOC.contains("WatchdogAnomalyError"));
    assert!(DOC.contains("evaluate_daemon_watchdog_anomaly"));
    assert!(DOC.contains("ValidatorProofConsensusEvaluator"));
    assert!(DOC.contains("ValidatorProofConsensusDecision"));
    assert!(DOC.contains("ProofWatchdogProjector"));
    assert!(DOC.contains("ProofWatchdogProjection"));
}

#[test]
fn doc_contains_incident_response_mapping_and_fast_lane() {
    assert!(DOC.contains("## Incident Response Mapping"));
    assert!(DOC.contains("`docs/foundation/upgrade-rollback-runbook.md`"));
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_watchdog_attestation_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test watchdog_node_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test upgrade_rollback_runbook_docs"));
    assert!(DOC.contains("cargo test -p kamn-core divergence_watchdog"));
    assert!(DOC.contains("cargo test -p kamn-core watchdog_anomaly"));
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
    assert!(DOC.contains("hash mismatch false-negative is rejected (`Regression: #381`)."));
    assert!(DOC.contains(
        "censorship edge-signal remains critical when targeted peers are at least two and delivery ratio is 500 per-mille or lower (`Regression: #382`)."
    ));
    assert!(DOC.contains("proof consensus alignment (`ConsensusValid`) projects `info` severity."));
    assert!(DOC.contains(
        "proof consensus invalid/replay/mismatch projects `critical` severity with deterministic fingerprint fields."
    ));
}
