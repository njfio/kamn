use kamn_agent_lib::AgentLibError;
use kamn_cli::{dispatch, CommandKind, OutputFormat, ParsedCliArgs};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String), String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    let read_deadline = Instant::now() + Duration::from_secs(5);
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;

    loop {
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if Instant::now() > read_deadline {
            break;
        }
        match stream.read(&mut chunk) {
            Ok(0) => {
                if Instant::now() > read_deadline {
                    break;
                }
                thread::sleep(Duration::from_millis(5));
            }
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                continue;
            }
            Err(error) => return Err(format!("request read failed: {error}")),
        }
    }

    let request_text =
        String::from_utf8(request).map_err(|_| "request was not valid utf-8".to_owned())?;
    let Some((request_head, _)) = request_text.split_once("\r\n\r\n") else {
        return Err("request header terminator missing".to_owned());
    };
    let request_line = request_head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?
        .to_owned();
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?
        .to_owned();
    Ok((method, path))
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let status_text = match status {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        _ => "500 Internal Server Error",
    };
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("service api write failed: {error}"))
}

fn run_cli_contract_server(bind_addr: String, max_requests: usize) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;

    // Workspace-wide test load can delay bind/accept scheduling; keep the fixture budget tolerant.
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut served = 0usize;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path) = parse_http_request(&mut stream)?;
                if method == "GET" && path == "/healthz" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/messages/send" {
                    write_http_response(
                        &mut stream,
                        202,
                        r#"{"message_id":"msg-cli","status":"created","runtime_mode":"api"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/channels/create" {
                    write_http_response(
                        &mut stream,
                        201,
                        r#"{"channel_id":"channel-cli","status":"created"}"#,
                    )?;
                } else if method == "GET" && path == "/v1/channels/channel-cli/messages" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"channel_id":"channel-cli","messages":["msg-1","msg-2"]}"#,
                    )?;
                } else if method == "GET" && path == "/v1/messages/msg-cli" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"message_id":"msg-cli","status":"created"}"#,
                    )?;
                } else if method == "GET" && path == "/v1/tasks/task-cli" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"task_id":"task-cli","state":"submitted"}"#,
                    )?;
                } else if method == "GET" && path == "/v1/agents/kamn:did:agent:alice" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"did":"kamn:did:agent:alice","reputation_score":777}"#,
                    )?;
                } else if method == "POST" && path == "/v1/content/register" {
                    write_http_response(
                        &mut stream,
                        201,
                        r#"{"content_id":"content-cli","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/content/content-cli/expire" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"content_id":"content-cli","lifecycle_state":"expired","redaction_status":"none"}"#,
                    )?;
                } else if (method == "POST" && path == "/v1/content/content-cli/tombstone")
                    || (method == "GET" && path == "/v1/content/content-cli")
                {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"content_id":"content-cli","lifecycle_state":"tombstoned","redaction_status":"redacted"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/bridge/submit" {
                    write_http_response(
                        &mut stream,
                        202,
                        r#"{"bridge_id":"bridge-cli","source_message_id":"msg-bridge-source-cli","bridge_status":"submitted"}"#,
                    )?;
                } else if (method == "POST" && path == "/v1/bridge/bridge-cli/forward")
                    || (method == "GET" && path == "/v1/bridge/bridge-cli")
                {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"bridge_id":"bridge-cli","bridge_status":"forwarded","target_message_id":"msg-bridge-target-cli","forward_tx_hash":"sha256:bridge-forwarded-cli"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/tasks/create" {
                    write_http_response(
                        &mut stream,
                        201,
                        r#"{"task_id":"task-cli","state":"submitted"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/tasks/task-cli/accept" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"task_id":"task-cli","state":"accepted"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/tasks/task-cli/complete" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"task_id":"task-cli","state":"completed"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/escrow/fund" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"escrow_id":"escrow-cli","state":"funded"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/escrow/escrow-cli/release" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"escrow_id":"escrow-cli","state":"released"}"#,
                    )?;
                } else {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"channel_id":"unknown","messages":[]}"#,
                    )?;
                }
                served += 1;
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("server accept failed: {error}")),
        }
    }
    Ok(())
}

fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(120));
}

fn parsed(command: CommandKind, endpoint: &str, passthrough: &[&str]) -> ParsedCliArgs {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    ParsedCliArgs {
        command,
        output_format: OutputFormat::Text,
        endpoint: endpoint.to_owned(),
        passthrough: passthrough.iter().map(|value| value.to_string()).collect(),
    }
}

fn parsed_json(command: CommandKind, endpoint: &str, passthrough: &[&str]) -> ParsedCliArgs {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    ParsedCliArgs {
        command,
        output_format: OutputFormat::Json,
        endpoint: endpoint.to_owned(),
        passthrough: passthrough.iter().map(|value| value.to_string()).collect(),
    }
}

#[test]
fn spec_c01_cli_health_command_executes_supported_path() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 1));
    wait_for_server_ready();

    let output = dispatch(&parsed(
        CommandKind::Health,
        format!("http://{bind_addr}").as_str(),
        &[],
    ))
    .expect("health command should succeed");
    assert!(
        output.text.contains("status=ok"),
        "health output should include status marker: {output:?}"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c11_cli_help_command_renders_usage_surface() {
    let text_output = dispatch(&parsed(CommandKind::Help, "http://localhost:18080", &[]))
        .expect("help command should succeed");
    assert!(
        text_output.text.contains("usage=kamn-cli <command>"),
        "help text should include usage: {text_output:?}"
    );
    assert!(
        text_output.text.contains("commands=register"),
        "help text should include command inventory: {text_output:?}"
    );
    assert!(
        text_output
            .text
            .contains("flags=--help, -h, --format, --endpoint"),
        "help text should include known flags: {text_output:?}"
    );

    let json_output = dispatch(&parsed_json(
        CommandKind::Help,
        "http://localhost:18080",
        &[],
    ))
    .expect("help command should succeed in json mode");
    assert!(
        json_output
            .json
            .contains("\"flags\":[\"--help\",\"-h\",\"--format\",\"--endpoint\"]"),
        "help json should expose deterministic flag set: {json_output:?}"
    );
}

#[test]
fn spec_c02_cli_list_messages_command_executes_and_validates_args() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 1));
    wait_for_server_ready();

    let output = dispatch(&parsed(
        CommandKind::ListMessages,
        format!("http://{bind_addr}").as_str(),
        &["channel-cli"],
    ))
    .expect("list-messages should succeed");
    assert!(
        output.text.contains("channel_id=channel-cli"),
        "list-messages output should include channel id: {output:?}"
    );
    assert!(
        output.text.contains("msg-1,msg-2"),
        "list-messages output should include message ids: {output:?}"
    );

    let missing_args_error = dispatch(&parsed(
        CommandKind::ListMessages,
        format!("http://{bind_addr}").as_str(),
        &[],
    ))
    .expect_err("missing channel id should fail");
    assert!(matches!(
        missing_args_error,
        AgentLibError::InvalidInput { .. }
    ));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c03_cli_verify_proof_command_executes_and_validates_args() {
    let output = dispatch(&parsed(
        CommandKind::VerifyProof,
        "http://localhost:18080",
        &["msg-1", "tx-1", "9", "final"],
    ))
    .expect("verify-proof command should succeed");
    assert!(
        output.text.contains("message_id=msg-1"),
        "verify-proof output should include message id: {output:?}"
    );
    assert!(
        output.text.contains("verified=true"),
        "verify-proof output should include verified projection: {output:?}"
    );

    let invalid_block_height = dispatch(&parsed(
        CommandKind::VerifyProof,
        "http://localhost:18080",
        &["msg-1", "tx-1", "not-a-number", "final"],
    ))
    .expect_err("malformed block-height should fail");
    assert!(matches!(
        invalid_block_height,
        AgentLibError::InvalidInput { .. }
    ));
}

#[test]
fn spec_c04_cli_task_and_escrow_commands_execute_and_validate_args() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 4));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");

    let accept_output = dispatch(&parsed(
        CommandKind::AcceptTask,
        endpoint.as_str(),
        &["task-cli"],
    ))
    .expect("accept-task should succeed");
    assert!(
        accept_output.text.contains("state=accepted"),
        "accept-task output should include accepted state: {accept_output:?}"
    );

    let complete_output = dispatch(&parsed(
        CommandKind::CompleteTask,
        endpoint.as_str(),
        &["task-cli"],
    ))
    .expect("complete-task should succeed");
    assert!(
        complete_output.text.contains("state=completed"),
        "complete-task output should include completed state: {complete_output:?}"
    );

    let fund_output = dispatch(&parsed(
        CommandKind::FundEscrow,
        endpoint.as_str(),
        &[r#"{"task_id":"task-cli","amount":100}"#],
    ))
    .expect("fund-escrow should succeed");
    assert!(
        fund_output.text.contains("escrow_id=escrow-cli"),
        "fund-escrow output should include escrow id: {fund_output:?}"
    );
    assert!(
        fund_output.text.contains("state=funded"),
        "fund-escrow output should include funded state: {fund_output:?}"
    );

    let release_output = dispatch(&parsed(
        CommandKind::ReleaseEscrow,
        endpoint.as_str(),
        &["escrow-cli"],
    ))
    .expect("release-escrow should succeed");
    assert!(
        release_output.text.contains("state=released"),
        "release-escrow output should include released state: {release_output:?}"
    );

    for (command, label) in [
        (CommandKind::AcceptTask, "task_id"),
        (CommandKind::CompleteTask, "task_id"),
        (CommandKind::FundEscrow, "fund_escrow_payload"),
        (CommandKind::ReleaseEscrow, "escrow_id"),
    ] {
        let error = dispatch(&parsed(command, endpoint.as_str(), &[]))
            .expect_err("missing required arg should fail");
        assert!(
            matches!(error, AgentLibError::InvalidInput { .. }),
            "missing required arg for {label} should be invalid input: {error}"
        );
    }

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c05_cli_core_message_and_task_commands_execute_and_validate_args() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 4));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");

    let register_output = dispatch(&parsed(CommandKind::Register, endpoint.as_str(), &[]))
        .expect("register should succeed");
    assert!(
        register_output.text.contains("kamn:did:agent"),
        "register output should include did marker: {register_output:?}"
    );

    let send_output = dispatch(&parsed(
        CommandKind::SendMessage,
        endpoint.as_str(),
        &[r#"{"message":"hello"}"#],
    ))
    .expect("send-message should succeed");
    assert!(
        send_output.text.contains("message_id=msg-cli"),
        "send-message output should include message id: {send_output:?}"
    );

    let channel_output = dispatch(&parsed(
        CommandKind::CreateChannel,
        endpoint.as_str(),
        &[r#"{"name":"ops"}"#],
    ))
    .expect("create-channel should succeed");
    assert!(
        channel_output.text.contains("channel_id=channel-cli"),
        "create-channel output should include channel id: {channel_output:?}"
    );

    let query_output = dispatch(&parsed(
        CommandKind::QueryMessage,
        endpoint.as_str(),
        &["msg-cli"],
    ))
    .expect("query-message should succeed");
    assert!(
        query_output.text.contains("status=created"),
        "query-message output should include status marker: {query_output:?}"
    );

    let task_output = dispatch(&parsed(
        CommandKind::CreateTask,
        endpoint.as_str(),
        &[r#"{"task":"triage"}"#],
    ))
    .expect("create-task should succeed");
    assert!(
        task_output.text.contains("task_id=task-cli"),
        "create-task output should include task id: {task_output:?}"
    );

    for (command, label) in [
        (CommandKind::SendMessage, "send_message_payload"),
        (CommandKind::CreateChannel, "create_channel_payload"),
        (CommandKind::QueryMessage, "query_message_id"),
        (CommandKind::CreateTask, "create_task_payload"),
    ] {
        let error = dispatch(&parsed(command, endpoint.as_str(), &[]))
            .expect_err("missing required arg should fail");
        assert!(
            matches!(error, AgentLibError::InvalidInput { .. }),
            "missing arg for {label} should be invalid input: {error}"
        );
    }

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 2));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");

    let query_task_output = dispatch(&parsed(
        CommandKind::QueryTask,
        endpoint.as_str(),
        &["task-cli"],
    ))
    .expect("query-task should succeed");
    assert!(
        query_task_output.text.contains("task_id=task-cli"),
        "query-task output should include task id: {query_task_output:?}"
    );
    assert!(
        query_task_output.text.contains("state=submitted"),
        "query-task output should include state projection: {query_task_output:?}"
    );

    let query_profile_output = dispatch(&parsed(
        CommandKind::QueryAgentProfile,
        endpoint.as_str(),
        &["kamn:did:agent:alice"],
    ))
    .expect("query-agent-profile should succeed");
    assert!(
        query_profile_output
            .text
            .contains("did=kamn:did:agent:alice"),
        "query-agent-profile output should include did: {query_profile_output:?}"
    );
    assert!(
        query_profile_output.text.contains("reputation_score=777"),
        "query-agent-profile output should include reputation_score: {query_profile_output:?}"
    );

    for (command, label) in [
        (CommandKind::QueryTask, "query_task_id"),
        (CommandKind::QueryAgentProfile, "query_agent_profile_did"),
    ] {
        let error = dispatch(&parsed(command, endpoint.as_str(), &[]))
            .expect_err("missing required arg should fail");
        assert!(
            matches!(error, AgentLibError::InvalidInput { .. }),
            "missing arg for {label} should be invalid input: {error}"
        );
    }

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c08_cli_content_commands_execute_and_validate_args() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 4));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");

    let register_output = dispatch(&parsed(
        CommandKind::RegisterContent,
        endpoint.as_str(),
        &[r#"{"content":"abc","retention_class":"standard"}"#],
    ))
    .expect("register-content should succeed");
    assert!(
        register_output.text.contains("content_id=content-cli"),
        "register-content output should include content id: {register_output:?}"
    );
    assert!(
        register_output.text.contains("retention_class=standard"),
        "register-content output should include retention class: {register_output:?}"
    );

    let expire_output = dispatch(&parsed(
        CommandKind::ExpireContent,
        endpoint.as_str(),
        &["content-cli"],
    ))
    .expect("expire-content should succeed");
    assert!(
        expire_output.text.contains("lifecycle_state=expired"),
        "expire-content output should include lifecycle state: {expire_output:?}"
    );

    let tombstone_output = dispatch(&parsed(
        CommandKind::TombstoneContent,
        endpoint.as_str(),
        &["content-cli"],
    ))
    .expect("tombstone-content should succeed");
    assert!(
        tombstone_output.text.contains("redaction_status=redacted"),
        "tombstone-content output should include redaction status: {tombstone_output:?}"
    );

    let query_output = dispatch(&parsed(
        CommandKind::QueryContent,
        endpoint.as_str(),
        &["content-cli"],
    ))
    .expect("query-content should succeed");
    assert!(
        query_output.text.contains("lifecycle_state=tombstoned"),
        "query-content output should include lifecycle state: {query_output:?}"
    );

    for (command, label) in [
        (CommandKind::RegisterContent, "register_content_payload"),
        (CommandKind::ExpireContent, "expire_content_id"),
        (CommandKind::TombstoneContent, "tombstone_content_id"),
        (CommandKind::QueryContent, "query_content_id"),
    ] {
        let error = dispatch(&parsed(command, endpoint.as_str(), &[]))
            .expect_err("missing required arg should fail");
        assert!(
            matches!(error, AgentLibError::InvalidInput { .. }),
            "missing arg for {label} should be invalid input: {error}"
        );
    }

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c09_cli_bridge_commands_execute_and_validate_args() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 3));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");

    let submit_output = dispatch(&parsed(
        CommandKind::SubmitBridgeMessage,
        endpoint.as_str(),
        &[r#"{"source_message_id":"msg-cli","target_network":"testnet"}"#],
    ))
    .expect("submit-bridge-message should succeed");
    assert!(
        submit_output.text.contains("bridge_id=bridge-cli"),
        "submit-bridge-message output should include bridge id: {submit_output:?}"
    );
    assert!(
        submit_output.text.contains("bridge_status=submitted"),
        "submit-bridge-message output should include bridge status: {submit_output:?}"
    );

    let forward_output = dispatch(&parsed(
        CommandKind::ForwardBridgeMessage,
        endpoint.as_str(),
        &["bridge-cli"],
    ))
    .expect("forward-bridge-message should succeed");
    assert!(
        forward_output.text.contains("bridge_status=forwarded"),
        "forward-bridge-message output should include bridge status: {forward_output:?}"
    );
    assert!(
        forward_output
            .text
            .contains("target_message_id=msg-bridge-target-cli"),
        "forward-bridge-message output should include target id: {forward_output:?}"
    );

    let query_output = dispatch(&parsed(
        CommandKind::QueryBridgeMessage,
        endpoint.as_str(),
        &["bridge-cli"],
    ))
    .expect("query-bridge-message should succeed");
    assert!(
        query_output
            .text
            .contains("forward_tx_hash=sha256:bridge-forwarded-cli"),
        "query-bridge-message output should include forward tx marker: {query_output:?}"
    );

    for (command, label) in [
        (
            CommandKind::SubmitBridgeMessage,
            "submit_bridge_message_payload",
        ),
        (
            CommandKind::ForwardBridgeMessage,
            "forward_bridge_message_id",
        ),
        (CommandKind::QueryBridgeMessage, "query_bridge_message_id"),
    ] {
        let error = dispatch(&parsed(command, endpoint.as_str(), &[]))
            .expect_err("missing required arg should fail");
        assert!(
            matches!(error, AgentLibError::InvalidInput { .. }),
            "missing arg for {label} should be invalid input: {error}"
        );
    }

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c06_cli_json_output_contract_renders_structured_health_projection() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, 1));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");
    let output = dispatch(&parsed_json(CommandKind::Health, endpoint.as_str(), &[]))
        .expect("health command should succeed");
    assert!(
        output.json.starts_with('{'),
        "json output should start with object marker: {}",
        output.json
    );
    assert!(
        output.json.contains("\"status\":\"ok\""),
        "json output should include status projection: {}",
        output.json
    );
    assert!(
        !output.json.contains("\"result\":"),
        "json output should not wrap result as escaped blob: {}",
        output.json
    );
    assert!(
        output.text.contains("status=ok"),
        "text projection should still include key=value markers: {}",
        output.text
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}
