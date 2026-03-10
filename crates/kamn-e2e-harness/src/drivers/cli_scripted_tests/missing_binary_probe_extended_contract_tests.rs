use super::*;

#[test]
fn unit_run_live_s09_cli_transport_failover_probe_rejects_missing_binary() {
    let updates = failover_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s09_cli_transport_failover_probe);
}

#[test]
fn unit_run_live_s10_cli_topology_coherence_probe_rejects_missing_binary() {
    let updates = topology_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s10_cli_topology_coherence_probe);
}

#[test]
fn unit_run_live_s11_cli_signer_rotation_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s11_cli_signer_rotation_probe);
}

#[test]
fn unit_run_live_s12_cli_retention_deletion_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s12_cli_retention_deletion_probe);
}

#[test]
fn unit_run_live_s13_cli_bridge_forwarding_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s13_cli_bridge_forwarding_probe);
}

#[test]
fn unit_run_live_s14_cli_batch_merkle_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s14_cli_batch_merkle_probe);
}

#[test]
fn unit_run_live_s15_cli_performance_smoke_probe_rejects_missing_binary() {
    let updates = endpoint_updates();
    assert_missing_binary_probe_failure(&updates, run_live_s15_cli_performance_smoke_probe);
}
