use kamn_mcp_server::config::McpServerConfig;
use kamn_mcp_server::tools::build_tool_registry;

fn main() {
    let config = match McpServerConfig::from_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("kamn-mcp-server config error: {error}");
            std::process::exit(2);
        }
    };

    let tool_count = build_tool_registry().len();
    println!(
        "{{\"status\":\"ready\",\"endpoint\":\"{}\",\"agent_name\":\"{}\",\"tool_count\":{}}}",
        config.endpoint, config.agent_name, tool_count
    );
}
