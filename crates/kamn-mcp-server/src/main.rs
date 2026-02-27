use kamn_mcp_server::config::McpServerConfig;
use kamn_mcp_server::process_stdio_input;
use kamn_mcp_server::tools::build_tool_registry;
use std::fs;
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

fn normalize_agent_name_for_did(agent_name: &str) -> Result<String, String> {
    let trimmed = agent_name.trim();
    if trimmed.is_empty() {
        return Err("agent name must not be empty".to_owned());
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err("agent name must use [a-zA-Z0-9_-] only".to_owned());
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn load_identity_from_key_file(
    agent_name: &str,
    key_file: &str,
) -> Result<kamn_agent_lib::AgentIdentity, String> {
    let normalized_agent_name = normalize_agent_name_for_did(agent_name)?;
    let signing_key = fs::read_to_string(key_file)
        .map_err(|error| format!("failed to read key file `{key_file}`: {error}"))?;
    let did = format!("kamn:did:agent:{normalized_agent_name}");
    kamn_agent_lib::AgentIdentity::from_did_and_signing_key(did.as_str(), signing_key.as_str())
        .map_err(|error| format!("failed to parse key-file identity: {error}"))
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
    let identity =
        match load_identity_from_key_file(config.agent_name.as_str(), config.key_file.as_str()) {
            Ok(identity) => identity,
            Err(error) => {
                eprintln!("kamn-mcp-server config error: {error}");
                std::process::exit(2);
            }
        };
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", "http://localhost:3000");
    let handle = match kamn_agent_lib::KamnAgentHandle::with_identity(
        config.endpoint.as_str(),
        kolme_endpoint.as_str(),
        identity,
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
    use super::{
        load_identity_from_key_file, normalize_agent_name_for_did, validate_framed_content_length,
        MAX_FRAMED_CONTENT_LENGTH_BYTES,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_SIGNING_KEY_HEX: &str =
        "094cf4e1f3d974bbf3e72233e2c2937e8fdb094740e0f017e010aa47ac1201ac";

    fn temp_key_file_path(stem: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        path.push(format!("{stem}-{}-{timestamp}.key", std::process::id()));
        path
    }

    #[test]
    fn spec_c01_load_identity_from_key_file_uses_signing_material() {
        let path = temp_key_file_path("mcp-key-file");
        fs::write(&path, TEST_SIGNING_KEY_HEX).expect("temp key file should be writable");
        let identity = load_identity_from_key_file(
            "Alice",
            path.to_str().expect("temp path should render as utf-8"),
        )
        .expect("valid key file should load identity");
        fs::remove_file(&path).expect("temp key file should be removable");

        assert_eq!(identity.did().as_str(), "kamn:did:agent:alice");
        assert_eq!(identity.signing_key(), TEST_SIGNING_KEY_HEX);
    }

    #[test]
    fn spec_c02_load_identity_from_key_file_rejects_unreadable_path() {
        let path = temp_key_file_path("mcp-key-file-missing");
        let error = load_identity_from_key_file(
            "alice",
            path.to_str().expect("temp path should render as utf-8"),
        )
        .expect_err("missing path must fail");
        assert!(
            error.contains("failed to read key file"),
            "error should contain deterministic key-file read marker: {error}",
        );
    }

    #[test]
    fn spec_c03_validate_framed_content_length_accepts_configured_boundary() {
        assert!(
            validate_framed_content_length(MAX_FRAMED_CONTENT_LENGTH_BYTES).is_ok(),
            "boundary content-length should be accepted",
        );
    }

    #[test]
    fn spec_c04_validate_framed_content_length_rejects_values_above_boundary() {
        let error = validate_framed_content_length(MAX_FRAMED_CONTENT_LENGTH_BYTES + 1)
            .expect_err("oversized content-length should be rejected");
        assert!(
            error.contains("content-length exceeds maximum"),
            "error should include deterministic max-size marker: {error}",
        );
    }

    #[test]
    fn unit_normalize_agent_name_for_did_rejects_invalid_characters() {
        let error =
            normalize_agent_name_for_did("alice bad").expect_err("invalid chars should fail");
        assert!(
            error.contains("[a-zA-Z0-9_-]"),
            "normalization error should explain allowed charset: {error}",
        );
    }
}
