const DOC: &str = include_str!("../../../docs/foundation/channel-models-and-permissions.md");

#[test]
fn doc_contains_channel_models_and_permissions_scope() {
    assert!(DOC.contains("# Channel Models and Permissions Contract Rules"));
    assert!(DOC.contains("run_channel_policy_contract_lane.sh"));
    assert!(DOC.contains("channel_permissions_retention"));
}

#[test]
fn regression_requires_channel_policy_bypass_marker() {
    // Regression: #929
    assert!(DOC.contains("unauthorized channel policy bypass is rejected (`Regression: #929`)"));
    assert!(DOC.contains("test_run_channel_policy_contract_lane.sh"));
}
