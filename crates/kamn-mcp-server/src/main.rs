use kamn_mcp_server::config::McpServerConfig;
use kamn_mcp_server::process_stdio_input;
use kamn_mcp_server::tools::build_tool_registry;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

const MAX_FRAMED_CONTENT_LENGTH_BYTES: usize = 1_048_576;

fn env_var_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => default.to_owned(),
    }
}

fn parse_content_length_header_line(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    let prefix = "content-length:";
    if !trimmed.to_ascii_lowercase().starts_with(prefix) {
        return None;
    }
    trimmed
        .split_once(':')
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
}

fn validate_framed_content_length(content_length: usize) -> Result<(), String> {
    if content_length > MAX_FRAMED_CONTENT_LENGTH_BYTES {
        return Err(format!(
            "content-length exceeds maximum: {content_length} > {MAX_FRAMED_CONTENT_LENGTH_BYTES}"
        ));
    }
    Ok(())
}

fn write_stdio_response<W: Write>(writer: &mut W, response: &str) -> io::Result<()> {
    if response.starts_with("Content-Length: ") {
        writer.write_all(response.as_bytes())
    } else {
        writer.write_all(response.as_bytes())?;
        writer.write_all(b"\n")
    }
}

fn main() {
    let config = match McpServerConfig::from_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("kamn-mcp-server config error: {error}");
            std::process::exit(2);
        }
    };

    let tool_count = build_tool_registry().len();
    let _ = config.key_file.as_str();
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", "http://localhost:3000");
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

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let mut saw_request = false;

    loop {
        let mut first_line = String::new();
        let read_count = match reader.read_line(&mut first_line) {
            Ok(count) => count,
            Err(_) => {
                eprintln!("kamn-mcp-server io error: failed to read stdin");
                std::process::exit(2);
            }
        };
        if read_count == 0 {
            break;
        }
        if first_line.trim().is_empty() {
            continue;
        }

        let request_input = if let Some(initial_content_length) =
            parse_content_length_header_line(first_line.as_str())
        {
            let mut content_length = initial_content_length;
            loop {
                let mut header_line = String::new();
                let header_read_count = match reader.read_line(&mut header_line) {
                    Ok(count) => count,
                    Err(_) => {
                        eprintln!("kamn-mcp-server io error: failed to read framed header");
                        std::process::exit(2);
                    }
                };
                if header_read_count == 0 {
                    eprintln!("kamn-mcp-server io error: truncated framed request header");
                    std::process::exit(2);
                }
                if let Some(found) = parse_content_length_header_line(header_line.as_str()) {
                    content_length = found;
                }
                if header_line.trim().is_empty() {
                    break;
                }
            }
            if let Err(error) = validate_framed_content_length(content_length) {
                eprintln!("kamn-mcp-server io error: {error}");
                std::process::exit(2);
            }

            let mut payload_bytes = vec![0_u8; content_length];
            if reader.read_exact(payload_bytes.as_mut_slice()).is_err() {
                eprintln!("kamn-mcp-server io error: failed to read framed payload bytes");
                std::process::exit(2);
            }
            let payload = match String::from_utf8(payload_bytes) {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("kamn-mcp-server io error: framed payload was not utf-8");
                    std::process::exit(2);
                }
            };
            format!("Content-Length: {content_length}\r\n\r\n{payload}")
        } else {
            first_line
        };

        saw_request = true;
        let responses = match process_stdio_input(&handle, request_input.as_str()) {
            Ok(responses) => responses,
            Err(error) => {
                eprintln!("kamn-mcp-server protocol error: {error}");
                std::process::exit(2);
            }
        };
        for response in responses {
            if write_stdio_response(&mut writer, response.as_str()).is_err() {
                eprintln!("kamn-mcp-server io error: failed to write stdout");
                std::process::exit(2);
            }
        }
        if writer.flush().is_err() {
            eprintln!("kamn-mcp-server io error: failed to flush stdout");
            std::process::exit(2);
        }
    }

    if !saw_request {
        let ready_payload = format!(
            "{{\"status\":\"ready\",\"endpoint\":\"{}\",\"agent_name\":\"{}\",\"tool_count\":{}}}",
            config.endpoint, config.agent_name, tool_count
        );
        if write_stdio_response(&mut writer, ready_payload.as_str()).is_err()
            || writer.flush().is_err()
        {
            eprintln!("kamn-mcp-server io error: failed to write ready status");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_framed_content_length, MAX_FRAMED_CONTENT_LENGTH_BYTES};

    #[test]
    fn spec_c01_validate_framed_content_length_accepts_configured_boundary() {
        assert!(
            validate_framed_content_length(MAX_FRAMED_CONTENT_LENGTH_BYTES).is_ok(),
            "boundary content-length should be accepted",
        );
    }

    #[test]
    fn spec_c02_validate_framed_content_length_rejects_values_above_boundary() {
        let error = validate_framed_content_length(MAX_FRAMED_CONTENT_LENGTH_BYTES + 1)
            .expect_err("oversized content-length should be rejected");
        assert!(
            error.contains("content-length exceeds maximum"),
            "error should include deterministic max-size marker: {error}",
        );
    }
}
