use kamn_e2e_harness::drivers::mcp_agent::McpAgentDriver;
use kamn_e2e_harness::drivers::HarnessDriver;
use kamn_e2e_harness::ExecutionMode;

const REQUIRED_ENV_KEYS: &[&str] = &[
    "KAMN_E2E_MCP_AGENT_LIVE",
    "KAMN_E2E_MCP_AGENT_BINARY",
    "KAMN_ENDPOINT",
    "KAMN_AGENT_NAME",
    "KAMN_AGENT_KEY_FILE",
];
const LIVE_EXECUTION_MODE: ExecutionMode = ExecutionMode::McpTau;

#[test]
#[ignore = "requires local Kolme + KAMN runtime with explicit live env"]
fn integration_live_s05_mcp_agent_escrow_settlement_probe_against_local_runtime() {
    require_envs(REQUIRED_ENV_KEYS);

    let driver = live_driver();
    let result = driver.execute("S-05");

    assert_eq!(result.scenario_id, "S-05");
    assert_eq!(
        result.status, "pass",
        "live mcp-agent S-05 failed: {:?}",
        result.detail
    );
}

fn live_driver() -> McpAgentDriver {
    McpAgentDriver::from_env(LIVE_EXECUTION_MODE)
        .expect("mcp agent live S-05 test should build driver")
}

fn require_envs(keys: &[&str]) {
    for key in keys {
        require_env(key);
    }
}

fn require_env(key: &str) {
    let value = std::env::var(key)
        .unwrap_or_else(|_| panic!("required env missing for live MCP S-05 probe: {key}"));
    assert!(
        !value.trim().is_empty(),
        "required env must not be empty for live MCP S-05 probe: {key}"
    );
}
