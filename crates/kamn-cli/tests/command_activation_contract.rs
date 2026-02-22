use kamn_agent_lib::AgentLibError;
use kamn_cli::{dispatch, CommandKind, OutputFormat, ParsedCliArgs};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String), String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
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

    let deadline = Instant::now() + Duration::from_secs(2);
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
                } else if method == "POST" && path == "/v1/tasks/create" {
                    write_http_response(
                        &mut stream,
                        201,
                        r#"{"task_id":"task-cli","state":"submitted"}"#,
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
    thread::sleep(Duration::from_millis(40));
}

fn parsed(command: CommandKind, endpoint: &str, passthrough: &[&str]) -> ParsedCliArgs {
    ParsedCliArgs {
        command,
        output_format: OutputFormat::Text,
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
        output.contains("status=ok"),
        "health output should include status marker: {output}"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
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
        output.contains("channel_id=channel-cli"),
        "list-messages output should include channel id: {output}"
    );
    assert!(
        output.contains("msg-1,msg-2"),
        "list-messages output should include message ids: {output}"
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
        output.contains("message_id=msg-1"),
        "verify-proof output should include message id: {output}"
    );
    assert!(
        output.contains("verified=true"),
        "verify-proof output should include verified projection: {output}"
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
fn spec_c04_cli_unsupported_command_regression_remains_explicit() {
    let unsupported_error = dispatch(&parsed(
        CommandKind::AcceptTask,
        "http://localhost:18080",
        &["task-1"],
    ))
    .expect_err("accept-task should remain unsupported");
    assert!(matches!(
        unsupported_error,
        AgentLibError::UnsupportedOperation(_)
    ));
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
        register_output.contains("kamn:did:agent"),
        "register output should include did marker: {register_output}"
    );

    let send_output = dispatch(&parsed(
        CommandKind::SendMessage,
        endpoint.as_str(),
        &[r#"{"message":"hello"}"#],
    ))
    .expect("send-message should succeed");
    assert!(
        send_output.contains("message_id=msg-cli"),
        "send-message output should include message id: {send_output}"
    );

    let channel_output = dispatch(&parsed(
        CommandKind::CreateChannel,
        endpoint.as_str(),
        &[r#"{"name":"ops"}"#],
    ))
    .expect("create-channel should succeed");
    assert!(
        channel_output.contains("channel_id=channel-cli"),
        "create-channel output should include channel id: {channel_output}"
    );

    let query_output = dispatch(&parsed(
        CommandKind::QueryMessage,
        endpoint.as_str(),
        &["msg-cli"],
    ))
    .expect("query-message should succeed");
    assert!(
        query_output.contains("status=created"),
        "query-message output should include status marker: {query_output}"
    );

    let task_output = dispatch(&parsed(
        CommandKind::CreateTask,
        endpoint.as_str(),
        &[r#"{"task":"triage"}"#],
    ))
    .expect("create-task should succeed");
    assert!(
        task_output.contains("task_id=task-cli"),
        "create-task output should include task id: {task_output}"
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
