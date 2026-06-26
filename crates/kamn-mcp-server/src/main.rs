use kamn_agent_lib::{AgentIdentity, KamnAgentHandle};
use kamn_mcp_server::config::McpServerConfig;
use kamn_mcp_server::process_stdio_input;
use kamn_mcp_server::tools::build_tool_registry;
use kamn_sdk::{service_public_key_for_private_key, AgentDid};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

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

fn write_stdio_response<W: Write>(writer: &mut W, response: &str) -> io::Result<()> {
    if response.starts_with("Content-Length: ") {
        writer.write_all(response.as_bytes())
    } else {
        writer.write_all(response.as_bytes())?;
        writer.write_all(b"\n")
    }
}

fn validate_framed_content_length(content_length: usize) -> Result<(), String> {
    if content_length > MAX_FRAMED_CONTENT_LENGTH_BYTES {
        return Err(format!(
            "content-length {content_length} exceeds max {MAX_FRAMED_CONTENT_LENGTH_BYTES}"
        ));
    }
    Ok(())
}

fn load_signing_key_from_file(path: &Path) -> Result<String, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read key file {}: {error}", path.display()))?;
    let signing_key = raw.trim();
    if signing_key.is_empty() {
        return Err(format!(
            "key file {} did not contain key material",
            path.display()
        ));
    }
    Ok(signing_key.to_owned())
}

fn normalize_agent_name_for_did(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
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

fn build_identity_from_key_file(agent_name: &str, key_file: &str) -> Result<AgentIdentity, String> {
    let normalized = normalize_agent_name_for_did(agent_name)?;
    let signing_key = load_signing_key_from_file(Path::new(key_file))?;
    let signer_public_key = service_public_key_for_private_key(signing_key.as_str())
        .map_err(|error| format!("failed to derive signer public key: {error}"))?;
    let did =
        AgentDid::with_public_key_hex_binding(normalized.as_str(), signer_public_key.as_str())
            .map_err(|error| format!("failed to bind did to signer public key: {error}"))?;
    AgentIdentity::from_did_and_signing_key(did.as_str(), signing_key.as_str())
        .map_err(|error| format!("failed to construct identity: {error}"))
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
    let kolme_endpoint = env_var_or_default("KAMN_KOLME_ENDPOINT", "http://localhost:3000");
    let identity =
        match build_identity_from_key_file(config.agent_name.as_str(), config.key_file.as_str()) {
            Ok(identity) => identity,
            Err(error) => {
                eprintln!("kamn-mcp-server config error: {error}");
                std::process::exit(2);
            }
        };
    let handle = match KamnAgentHandle::with_identity(
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
                eprintln!("kamn-mcp-server protocol error: {error}");
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
        build_identity_from_key_file, load_signing_key_from_file, validate_framed_content_length,
        MAX_FRAMED_CONTENT_LENGTH_BYTES,
    };
    use kamn_sdk::{service_public_key_for_private_key, AgentDid};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "kamn-mcp-server-{suffix}-{nanos}-{}.key",
            std::process::id()
        ))
    }

    fn write_test_key_file(path: &PathBuf) {
        fs::write(
            path.as_path(),
            "1111111111111111111111111111111111111111111111111111111111111111\n",
        )
        .expect("key file should write");
    }

    fn expected_bound_did(signing_key: &str) -> AgentDid {
        let expected_public_key = service_public_key_for_private_key(signing_key)
            .expect("public key should derive from key file");
        AgentDid::with_public_key_hex_binding("alice", expected_public_key.as_str())
            .expect("expected did should bind to signer key")
    }

    #[test]
    fn regression_issue_6197_load_signing_key_from_file_consumes_key_material() {
        let path = temp_file_path("regression-6197");
        write_test_key_file(&path);

        let loaded =
            load_signing_key_from_file(path.as_path()).expect("key file should load successfully");
        assert_eq!(
            loaded,
            "1111111111111111111111111111111111111111111111111111111111111111"
        );

        let identity = build_identity_from_key_file("Alice", path.to_str().expect("utf8 path"))
            .expect("identity should build from key file");
        let expected_did = expected_bound_did(loaded.as_str());
        assert_eq!(identity.did(), &expected_did);
        assert_eq!(identity.signing_key(), loaded);

        let _ = fs::remove_file(path.as_path());
    }

    #[test]
    fn regression_issue_6197_load_signing_key_from_file_rejects_empty_content() {
        let path = temp_file_path("regression-6197-empty");
        fs::write(path.as_path(), "  \n\t").expect("empty-ish key file should write");
        let error =
            load_signing_key_from_file(path.as_path()).expect_err("empty key file must fail");
        assert!(error.contains("did not contain key material"));
        let _ = fs::remove_file(path.as_path());
    }

    #[test]
    fn regression_issue_6199_validate_content_length_accepts_boundary_values() {
        validate_framed_content_length(MAX_FRAMED_CONTENT_LENGTH_BYTES)
            .expect("max boundary should remain valid");
    }

    #[test]
    fn regression_issue_6199_validate_content_length_rejects_oversize_values() {
        let error = validate_framed_content_length(MAX_FRAMED_CONTENT_LENGTH_BYTES + 1)
            .expect_err("oversize content-length should be rejected");
        assert!(error.contains("exceeds max"));
    }
}
