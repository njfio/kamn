use super::*;

#[test]
fn unit_run_live_s09_mcp_transport_failover_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &transport_failover_updates(),
        run_live_s09_mcp_transport_failover_probe,
    );
}

#[test]
fn unit_run_live_s10_mcp_topology_coherence_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &topology_endpoint_updates(),
        run_live_s10_mcp_topology_coherence_probe,
    );
}

#[test]
fn unit_run_live_s11_mcp_signer_rotation_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &default_endpoint_key_updates(),
        run_live_s11_mcp_signer_rotation_probe,
    );
}

#[test]
fn unit_run_live_s12_mcp_retention_deletion_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &default_endpoint_key_updates(),
        run_live_s12_mcp_retention_deletion_probe,
    );
}

#[test]
fn unit_run_live_s13_mcp_bridge_forwarding_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &default_endpoint_key_updates(),
        run_live_s13_mcp_bridge_forwarding_probe,
    );
}

#[test]
fn unit_run_live_s14_mcp_batch_merkle_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &default_endpoint_key_updates(),
        run_live_s14_mcp_batch_merkle_probe,
    );
}

#[test]
fn unit_run_live_s15_mcp_performance_smoke_probe_rejects_missing_binary() {
    assert_missing_binary_probe_failure(
        &default_endpoint_key_updates(),
        run_live_s15_mcp_performance_smoke_probe,
    );
}
