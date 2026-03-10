use super::super::*;

pub(crate) fn run_live_s01_mcp_probe() -> Result<(), String> {
    let settings = s01_settings();
    run_named_mcp_tool_call(
        "mcp live probe",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-health",
        "health",
        "{}",
    )
    .map(|_| ())
}

struct S01Settings {
    binary: String,
    endpoint: String,
    agent_name: String,
    key_file: String,
}

fn s01_settings() -> S01Settings {
    S01Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        agent_name: env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
    }
}
