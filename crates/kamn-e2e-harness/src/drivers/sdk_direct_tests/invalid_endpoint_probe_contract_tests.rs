use super::*;

#[test]
fn unit_run_live_s01_discovery_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_agent_updates();
    assert_probe_error_matches_any(
        &updates,
        "service.endpoint",
        "service endpoint",
        run_live_s01_discovery_probe,
    );
}

#[test]
fn unit_run_live_s02_direct_message_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s02_direct_message_probe,
    );
}

#[test]
fn unit_run_live_s03_group_channel_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(&updates, "connect failed", run_live_s03_group_channel_probe);
}

#[test]
fn unit_run_live_s04_task_lifecycle_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_agent_updates();
    assert_probe_error_matches_any(
        &updates,
        "service.endpoint",
        "service endpoint",
        run_live_s04_task_lifecycle_probe,
    );
}

#[test]
fn unit_run_live_s05_escrow_settlement_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s05_escrow_settlement_probe,
    );
}

#[test]
fn unit_run_live_s07_replay_protection_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s07_replay_protection_probe,
    );
}

#[test]
fn unit_run_live_s08_crash_recovery_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s08_crash_recovery_probe,
    );
}

#[test]
fn unit_run_live_s09_transport_failover_probe_rejects_invalid_endpoint() {
    let updates = invalid_failover_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s09_transport_failover_probe,
    );
}

#[test]
fn unit_run_live_s10_topology_coherence_probe_rejects_invalid_endpoint() {
    let updates = invalid_topology_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s10_topology_coherence_probe,
    );
}

#[test]
fn unit_run_live_s11_signer_rotation_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s11_signer_rotation_probe,
    );
}

#[test]
fn unit_run_live_s12_retention_deletion_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s12_retention_deletion_probe,
    );
}

#[test]
fn unit_run_live_s13_bridge_forwarding_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s13_bridge_forwarding_probe,
    );
}

#[test]
fn unit_run_live_s14_batch_merkle_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(&updates, "connect failed", run_live_s14_batch_merkle_probe);
}

#[test]
fn unit_run_live_s15_performance_smoke_probe_rejects_invalid_endpoint() {
    let updates = invalid_endpoint_updates();
    assert_probe_error_contains(
        &updates,
        "connect failed",
        run_live_s15_performance_smoke_probe,
    );
}
