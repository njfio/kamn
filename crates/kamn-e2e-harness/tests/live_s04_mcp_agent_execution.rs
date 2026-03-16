use kamn_agent_lib::AgentIdentity;
use kamn_e2e_harness::drivers::mcp_agent::McpAgentDriver;
use kamn_e2e_harness::drivers::HarnessDriver;
use kamn_e2e_harness::ExecutionMode;
use std::fs;

const REQUIRED_ENV_KEYS: &[&str] = &[
    "KAMN_E2E_MCP_AGENT_LIVE",
    "KAMN_E2E_MCP_AGENT_BINARY",
    "KAMN_ENDPOINT",
    "KAMN_KOLME_ENDPOINT",
    "KAMN_AGENT_NAME",
    "KAMN_AGENT_KEY_FILE",
];
const LIVE_EXECUTION_MODE: ExecutionMode = ExecutionMode::McpTau;

#[test]
#[ignore = "requires local Kolme + KAMN runtime with explicit live env"]
fn integration_live_s04_mcp_agent_task_lifecycle_probe_against_local_runtime() {
    require_envs(REQUIRED_ENV_KEYS);
    materialize_matching_key_file();

    let driver = live_driver();
    let result = driver.execute("S-04");

    assert_eq!(result.scenario_id, "S-04");
    assert_eq!(
        result.status, "pass",
        "live mcp-agent S-04 failed: {:?}",
        result.detail
    );
}

fn live_driver() -> McpAgentDriver {
    McpAgentDriver::from_env(LIVE_EXECUTION_MODE)
        .expect("mcp agent live S-04 test should build driver")
}

fn require_envs(keys: &[&str]) {
    for key in keys {
        require_env(key);
    }
}

fn materialize_matching_key_file() {
    let agent_name = required_env_value("KAMN_AGENT_NAME");
    let key_file = required_env_value("KAMN_AGENT_KEY_FILE");
    let identity = AgentIdentity::from_agent_name(agent_name.as_str())
        .expect("live MCP S-04 test should derive deterministic signing key");
    fs::write(key_file.as_str(), format!("{}\n", identity.signing_key()))
        .unwrap_or_else(|error| panic!("failed to write live MCP S-04 key file: {error}"));
}

fn require_env(key: &str) {
    let value = required_env_value(key);
    assert!(
        !value.trim().is_empty(),
        "required env must not be empty for live MCP S-04 probe: {key}"
    );
}

fn required_env_value(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("required env missing for live MCP S-04 probe: {key}"))
}
