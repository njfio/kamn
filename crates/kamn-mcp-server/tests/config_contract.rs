use kamn_mcp_server::config::McpServerConfig;

#[test]
fn spec_c04_mcp_config_parses_required_fields() {
    let config = McpServerConfig::from_args([
        "--endpoint",
        "http://localhost:8080",
        "--agent-name",
        "alice",
        "--key-file",
        "/tmp/alice.key",
    ])
    .expect("config");

    assert_eq!(config.endpoint, "http://localhost:8080");
    assert_eq!(config.agent_name, "alice");
    assert_eq!(config.key_file, "/tmp/alice.key");
}
