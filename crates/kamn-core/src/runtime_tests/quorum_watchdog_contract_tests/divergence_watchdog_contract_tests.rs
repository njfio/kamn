use super::super::*;

#[test]
fn functional_divergence_watchdog_flags_hash_mismatch_as_critical() {
    let evaluator = StateDivergenceEvaluator;
    let input = StateDivergenceWatchInput::new(
        "kamn:did:agent:validator-a",
        42,
        42,
        "state-hash-expected",
        "state-hash-observed",
        110,
    )
    .expect("valid divergence input");
    let report = evaluator
        .evaluate(input)
        .expect("hash mismatch should emit divergence report");
    assert_eq!(report.status, StateDivergenceStatus::Diverged);
    assert_eq!(report.severity, StateDivergenceSeverity::Critical);
}

#[test]
fn unit_divergence_watchdog_rejects_incomplete_evidence_payload() {
    let error = StateDivergenceWatchInput::new(
        "kamn:did:agent:validator-a",
        42,
        42,
        "state-hash-expected",
        "",
        110,
    )
    .expect_err("empty observed hash must be rejected");
    assert_eq!(
        error,
        StateDivergenceError::IncompleteEvidenceField {
            field: "observed_state_hash"
        }
    );
}

#[test]
fn integration_daemon_divergence_report_includes_deterministic_evidence_fields() {
    let evaluator = StateDivergenceEvaluator;
    let input = StateDivergenceWatchInput::new(
        "kamn:did:agent:validator-a",
        42,
        42,
        "state-hash-expected",
        "state-hash-observed",
        110,
    )
    .expect("valid divergence input");
    let report = evaluate_daemon_state_divergence(&evaluator, input)
        .expect("daemon divergence evaluation should succeed");
    assert_eq!(report.evidence.peer_id, "kamn:did:agent:validator-a");
    assert_eq!(report.evidence.expected_state_version, 42);
    assert_eq!(report.evidence.observed_state_version, 42);
    assert_eq!(report.evidence.expected_state_hash, "state-hash-expected");
    assert_eq!(report.evidence.observed_state_hash, "state-hash-observed");
    assert_eq!(report.evidence.observed_at_tick, 110);
    assert_eq!(
        report.incident_fingerprint,
        "state-divergence:kamn:did:agent:validator-a:42:42:state-hash-expected:state-hash-observed"
    );
}

#[test]
fn regression_state_divergence_false_negative_is_rejected() {
    let evaluator = StateDivergenceEvaluator;
    let input = StateDivergenceWatchInput::new(
        "kamn:did:agent:validator-a",
        99,
        99,
        "state-hash-expected",
        "state-hash-mismatched",
        220,
    )
    .expect("valid divergence input");
    let report = evaluate_daemon_state_divergence(&evaluator, input)
        .expect("mismatch must produce divergence report");
    assert_eq!(report.status, StateDivergenceStatus::Diverged);
    assert_ne!(
        report.evidence.expected_state_hash,
        report.evidence.observed_state_hash
    );
}
