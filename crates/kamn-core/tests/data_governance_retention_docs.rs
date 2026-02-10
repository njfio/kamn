const DOC: &str = include_str!("../../../docs/foundation/data-governance-retention.md");

#[test]
fn doc_contains_retention_redaction_contract_scope() {
    assert!(DOC.contains("# Data Governance Retention and Redaction Contracts"));
    assert!(DOC.contains("run_channel_retention_redaction_contract_lane.sh"));
    assert!(DOC.contains("channel_retention_redaction_contract.py"));
    assert!(DOC.contains("kamn.channel.retention-redaction-evidence.v1"));
}

#[test]
fn regression_requires_replay_safe_reason_code_marker() {
    // Regression: #930
    assert!(DOC.contains("replay-safe reason-code drift is rejected (`Regression: #930`)"));
    assert!(DOC.contains("check_channel_retention_redaction_policy.sh"));
}
