use kamn_mcp_server::config::McpServerConfig;
use kamn_mcp_server::process_stdio_input;
use kamn_mcp_server::tools::build_tool_registry;
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

    let responses = match process_stdio_input(&handle, input.as_str()) {
        Ok(responses) => responses,
        Err(error) => {
            eprintln!("kamn-mcp-server protocol error: {error}");
            std::process::exit(2);
        }
    };

    for response in responses {
        if response.starts_with("Content-Length: ") {
            print!("{response}");
        } else {
            println!("{response}");
        }
    }
}
