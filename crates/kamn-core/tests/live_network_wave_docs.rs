const LIVE_NETWORK_WAVE_DOC: &str = include_str!("../../../docs/planning/live-network-wave.md");
const README: &str = include_str!("../../../README.md");
const MAKEFILE: &str = include_str!("../../../Makefile");

#[test]
fn live_network_wave_doc_contains_smoke_commands_and_schema() {
    assert!(LIVE_NETWORK_WAVE_DOC.contains("run_live_network_smoke_lane.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("run_live_network_smoke_contract_lane.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("make smoke-live-network"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("kamn.runtime.live-network-smoke-report.v1"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("run_bridge_replay_redaction_contract_lane.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("run_bridge_replay_redaction_deep_lane.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("check_bridge_replay_redaction_policy.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("run_localhost_bridge_demo_evidence_contract_lane.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("run_localhost_bridge_demo_evidence_deep_lane.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("check_localhost_bridge_demo_policy.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("kamn.bridge.localhost-demo-evidence.v1"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("test_dashboard_package_runtime_compat.sh"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("npx -y node@22"));
}

#[test]
fn regression_budget_guard_marker_is_documented() {
    // Regression: #828
    assert!(LIVE_NETWORK_WAVE_DOC.contains("runtime_budget_exceeded"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("`Regression: #828`"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("120-second upper bound"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("300-second upper bound"));
    // Regression: #868
    assert!(LIVE_NETWORK_WAVE_DOC.contains("`Regression: #868`"));
    assert!(LIVE_NETWORK_WAVE_DOC.contains("node@22"));
}

#[test]
fn readme_and_makefile_expose_live_network_entrypoints() {
    assert!(README.contains("make smoke-live-network"));
    assert!(README.contains("make deep-live-network"));
    assert!(README.contains("make demo-localhost-transport"));
    assert!(README.contains("run_localhost_bridge_demo_evidence_contract_lane.sh"));
    assert!(README.contains("run_localhost_bridge_demo_evidence_deep_lane.sh"));
    assert!(MAKEFILE.contains("smoke-live-network:"));
    assert!(MAKEFILE.contains("deep-live-network:"));
    assert!(MAKEFILE.contains("run_live_network_smoke_lane.sh"));
    assert!(MAKEFILE.contains("run_live_network_pilot_deep_lane.sh"));
}

#[test]
fn regression_localhost_transport_demo_entrypoint_is_documented() {
    // Regression: #877
    assert!(README.contains("run_localhost_signed_demo.sh"));
    assert!(MAKEFILE.contains("demo-localhost-transport:"));
}
