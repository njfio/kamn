use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use kamn_e2e_harness::drivers::mcp_agent::McpAgentDriver;
use kamn_e2e_harness::drivers::HarnessDriver;
use kamn_e2e_harness::ExecutionMode;

#[test]
fn spec_c01_mcp_agent_live_toggle_disabled_keeps_deterministic_pass_without_probe_invocation() {
    let probe_calls = Arc::new(AtomicUsize::new(0));
    let probe_calls_for_closure = Arc::clone(&probe_calls);
    let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, false, move || {
        probe_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Err("probe should not run when live toggle is disabled".to_owned())
    })
    .expect("driver should build");

    let result = driver.execute("S-01");
    assert_eq!(result.status, "pass");
    assert_eq!(
        probe_calls.load(Ordering::SeqCst),
        0,
        "probe must not be invoked when live toggle is disabled",
    );
}

#[test]
fn spec_c02_mcp_agent_live_s01_success_maps_to_pass() {
    let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || Ok(()))
        .expect("driver should build");
    let result = driver.execute("S-01");
    assert_eq!(result.status, "pass");
}

#[test]
fn spec_c03_mcp_agent_live_s01_failure_maps_to_fail() {
    let driver = McpAgentDriver::with_probe(ExecutionMode::McpAny, true, || {
        Err("probe failure".to_owned())
    })
    .expect("driver should build");
    let result = driver.execute("S-01");
    assert_eq!(result.status, "fail");
}

#[test]
fn spec_c04_mcp_agent_live_non_s01_remains_pass_without_probe_invocation() {
    let probe_calls = Arc::new(AtomicUsize::new(0));
    let probe_calls_for_closure = Arc::clone(&probe_calls);
    let driver = McpAgentDriver::with_probe(ExecutionMode::McpAny, true, move || {
        probe_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .expect("driver should build");

    let result = driver.execute("S-10");
    assert_eq!(result.status, "pass");
    assert_eq!(
        probe_calls.load(Ordering::SeqCst),
        0,
        "non-S-01 scenarios should not invoke probe in this slice",
    );
}
