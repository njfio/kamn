use kamn_mcp_server::config::McpServerConfig;
use kamn_mcp_server::tools::build_tool_registry;
use kamn_mcp_server::{dispatch_tool_request_json, invalid_request_response_json};
use std::io::{self, Read};

fn main() {
    let config = match McpServerConfig::from_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("kamn-mcp-server config error: {error}");
            std::process::exit(2);
        }
    };

    let tool_count = build_tool_registry().len();
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("kamn-mcp-server io error: failed to read stdin");
        std::process::exit(2);
    }

    if input.trim().is_empty() {
        println!(
            "{{\"status\":\"ready\",\"endpoint\":\"{}\",\"agent_name\":\"{}\",\"tool_count\":{}}}",
            config.endpoint, config.agent_name, tool_count
        );
        return;
    }

    let _ = config.key_file.as_str();
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| "http://localhost:3000".to_owned());
    let handle = match kamn_agent_lib::KamnAgentHandle::connect(
        config.endpoint.as_str(),
        kolme_endpoint.as_str(),
        config.agent_name.as_str(),
    ) {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("kamn-mcp-server connect error: {error}");
            std::process::exit(2);
        }
    };

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let response = match dispatch_tool_request_json(&handle, line) {
            Ok(response) => response,
            Err(error) => invalid_request_response_json(error.as_str()),
        };
        println!("{response}");
    }
}
