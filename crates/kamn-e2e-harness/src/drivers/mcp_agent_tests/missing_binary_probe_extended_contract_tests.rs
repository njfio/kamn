use super::*;

#[test]
fn unit_run_live_s09_mcp_transport_failover_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&s09_updates(), run_live_s09_mcp_transport_failover_probe);
}

#[test]
fn unit_run_live_s10_mcp_topology_coherence_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&s10_updates(), run_live_s10_mcp_topology_coherence_probe);
}

#[test]
fn unit_run_live_s11_mcp_signer_rotation_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&single_endpoint_key_updates(), run_live_s11_mcp_signer_rotation_probe);
}

#[test]
fn unit_run_live_s12_mcp_retention_deletion_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&single_endpoint_key_updates(), run_live_s12_mcp_retention_deletion_probe);
}

#[test]
fn unit_run_live_s13_mcp_bridge_forwarding_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&single_endpoint_key_updates(), run_live_s13_mcp_bridge_forwarding_probe);
}

#[test]
fn unit_run_live_s14_mcp_batch_merkle_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&single_endpoint_key_updates(), run_live_s14_mcp_batch_merkle_probe);
}

#[test]
fn unit_run_live_s15_mcp_performance_smoke_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(&single_endpoint_key_updates(), run_live_s15_mcp_performance_smoke_probe);
}

fn single_endpoint_key_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}

fn s09_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_E2E_S09_FAILOVER_ENDPOINT", Some("http://localhost:8081")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}

fn s10_updates() -> [(&'static str, Option<&'static str>); 4] {
    [
        ("KAMN_E2E_S10_PRIMARY_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_E2E_S10_SECONDARY_ENDPOINT", Some("http://localhost:8081")),
        ("KAMN_E2E_S10_TERTIARY_ENDPOINT", Some("http://localhost:8082")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}
