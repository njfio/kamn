use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn spec_c00_live_disabled_driver_path_fails_closed_without_probe_invocation() {
    let probe_calls = Arc::new(AtomicUsize::new(0));
    let probe_calls_for_closure = Arc::clone(&probe_calls);
    let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, false, move || {
        probe_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .expect("driver should build");
    let result = crate::drivers::HarnessDriver::execute(&driver, "S-01");
    assert_eq!(result.status, "fail");
    assert_eq!(probe_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn spec_c09_live_s04_driver_path_fails_closed_when_task_probe_errors() {
    assert_driver_scenario_fails_closed("S-04", "mcp-agent live s04 task probe failed");
}

#[test]
fn spec_c10_live_s06_driver_path_fails_closed_when_proof_probe_errors() {
    assert_driver_scenario_fails_closed("S-06", "mcp-agent live s06 proof probe failed");
}

#[test]
fn spec_c11_live_s02_driver_path_fails_closed_when_message_probe_errors() {
    assert_driver_scenario_fails_closed("S-02", "mcp-agent live s02 message probe failed");
}

#[test]
fn spec_c12_live_s03_driver_path_fails_closed_when_channel_probe_errors() {
    assert_driver_scenario_fails_closed("S-03", "mcp-agent live s03 channel probe failed");
}

#[test]
fn spec_c13_live_s05_driver_path_fails_closed_when_escrow_probe_errors() {
    assert_driver_scenario_fails_closed("S-05", "mcp-agent live s05 escrow probe failed");
}

#[test]
fn spec_c14_live_s07_driver_path_fails_closed_when_replay_probe_errors() {
    assert_driver_scenario_fails_closed("S-07", "mcp-agent live s07 replay probe failed");
}

#[test]
fn spec_c15_live_s08_driver_path_fails_closed_when_crash_recovery_probe_errors() {
    assert_driver_scenario_fails_closed("S-08", "mcp-agent live s08 crash-recovery probe failed");
}

#[test]
fn spec_c16_live_s09_driver_path_fails_closed_when_transport_failover_probe_errors() {
    assert_driver_scenario_fails_closed(
        "S-09",
        "mcp-agent live s09 transport-failover probe failed",
    );
}

#[test]
fn spec_c17_live_s10_driver_path_fails_closed_when_topology_coherence_probe_errors() {
    assert_driver_scenario_fails_closed(
        "S-10",
        "mcp-agent live s10 topology-coherence probe failed",
    );
}

#[test]
fn spec_c18_live_s11_driver_path_fails_closed_when_signer_rotation_probe_errors() {
    assert_driver_scenario_fails_closed("S-11", "mcp-agent live s11 signer-rotation probe failed");
}

#[test]
fn spec_c19_live_s12_driver_path_fails_closed_when_retention_deletion_probe_errors() {
    assert_driver_scenario_fails_closed(
        "S-12",
        "mcp-agent live s12 retention-deletion probe failed",
    );
}

#[test]
fn spec_c20_live_s13_driver_path_fails_closed_when_bridge_forwarding_probe_errors() {
    assert_driver_scenario_fails_closed(
        "S-13",
        "mcp-agent live s13 bridge-forwarding probe failed",
    );
}

#[test]
fn spec_c21_live_s14_driver_path_fails_closed_when_batch_merkle_probe_errors() {
    assert_driver_scenario_fails_closed("S-14", "mcp-agent live s14 batch-merkle probe failed");
}

#[test]
fn spec_c22_live_s15_driver_path_fails_closed_when_performance_smoke_probe_errors() {
    assert_driver_scenario_fails_closed(
        "S-15",
        "mcp-agent live s15 performance-smoke probe failed",
    );
}

fn assert_driver_scenario_fails_closed(scenario_id: &'static str, message: &str) {
    let message = message.to_owned();
    let driver =
        McpAgentDriver::with_probe(ExecutionMode::McpTau, true, move || Err(message.clone()))
            .expect("driver should build");
    let result = crate::drivers::HarnessDriver::execute(&driver, scenario_id);
    assert_eq!(result.status, "fail");
}
