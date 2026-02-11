const DOC: &str = include_str!("../../../docs/testing/invariant-and-fuzz-strategy.md");

#[test]
fn doc_contains_live_transport_replay_tamper_contract_commands() {
    assert!(DOC.contains("run_live_transport_replay_tamper_contract_lane.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_fast_lane.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_deep_lane.sh"));
    assert!(DOC.contains("check_live_transport_replay_tamper_policy.sh"));
    assert!(DOC.contains("kamn.sdk.live-transport-replay-tamper-evidence.v1"));
}

#[test]
fn regression_requires_live_transport_replay_tamper_contract_markers() {
    // Regression: #1380
    assert!(DOC.contains("/tmp/live-transport-replay-tamper-contract-report.json"));
    assert!(DOC.contains("bundle-file /tmp/live-transport-replay-tamper-contract-report.json"));
}

#[test]
fn regression_requires_lifecycle_property_replay_metadata_markers() {
    // Regression: #1605
    assert!(DOC.contains("kamn.runtime.lifecycle-property-replay-metadata.v1"));
    assert!(DOC.contains("generated_sequence_bounds"));
    assert!(DOC.contains("executed_cases"));
}
