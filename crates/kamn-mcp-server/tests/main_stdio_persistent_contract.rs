use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

fn frame_request(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn read_framed_response(reader: &mut BufReader<impl Read>) -> String {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let read_count = reader
            .read_line(&mut line)
            .expect("response header line should be readable");
        assert!(
            read_count > 0,
            "response stream ended before header completed"
        );
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.to_ascii_lowercase().starts_with("content-length:") {
            if let Some((_, value)) = trimmed.split_once(':') {
                let parsed = value
                    .trim()
                    .parse::<usize>()
                    .expect("content-length header should be numeric");
                content_length = Some(parsed);
            }
        }
    }

    let body_len = content_length.expect("framed response should include content-length");
    let mut body_bytes = vec![0_u8; body_len];
    reader
        .read_exact(body_bytes.as_mut_slice())
        .expect("response body bytes should be readable");
    String::from_utf8(body_bytes).expect("response body should be utf-8")
}

#[test]
fn spec_c10_main_stdio_session_processes_multiple_framed_requests_without_eof() {
    let binary = env!("CARGO_BIN_EXE_kamn-mcp-server");
    let mut child = Command::new(binary)
        .args([
            "--endpoint",
            "http://127.0.0.1:18080",
            "--agent-name",
            "mcp-persistent-test",
            "--key-file",
            "/tmp/mcp-persistent-test.key",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp server process should spawn");

    let mut stdin = child
        .stdin
        .take()
        .expect("child stdin should be piped for framed requests");
    let stdout = child
        .stdout
        .take()
        .expect("child stdout should be piped for framed responses");
    let mut reader = BufReader::new(stdout);

    let initialize =
        frame_request(r#"{"jsonrpc":"2.0","id":"req-1","method":"initialize","params":{}}"#);
    stdin
        .write_all(initialize.as_bytes())
        .expect("initialize frame should write");
    stdin.flush().expect("initialize frame should flush");
    let initialize_response = read_framed_response(&mut reader);
    assert!(
        initialize_response.contains(r#""id":"req-1""#),
        "initialize response should preserve request id: {initialize_response}"
    );
    assert!(
        initialize_response.contains(r#""serverInfo""#),
        "initialize response should include server info: {initialize_response}"
    );

    let tools_list = frame_request(r#"{"jsonrpc":"2.0","id":"req-2","method":"tools/list"}"#);
    stdin
        .write_all(tools_list.as_bytes())
        .expect("tools/list frame should write");
    stdin.flush().expect("tools/list frame should flush");
    let tools_list_response = read_framed_response(&mut reader);
    assert!(
        tools_list_response.contains(r#""id":"req-2""#),
        "tools/list response should preserve request id: {tools_list_response}"
    );
    assert!(
        tools_list_response.contains(r#""tools""#),
        "tools/list response should include tool inventory: {tools_list_response}"
    );

    drop(stdin);
    let output = child
        .wait_with_output()
        .expect("mcp server should terminate after stdin closes");
    assert!(
        output.status.success(),
        "mcp server should exit cleanly after persistent session; stderr={}",
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}

#[test]
fn spec_c03_main_stdio_rejects_oversized_framed_content_length() {
    let binary = env!("CARGO_BIN_EXE_kamn-mcp-server");
    let mut child = Command::new(binary)
        .args([
            "--endpoint",
            "http://127.0.0.1:18080",
            "--agent-name",
            "mcp-content-length-cap-test",
            "--key-file",
            "/tmp/mcp-content-length-cap-test.key",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp server process should spawn");

    let mut stdin = child
        .stdin
        .take()
        .expect("child stdin should be piped for framed requests");

    // Regression: #6120
    let oversized_frame = "Content-Length: 1048577\r\n\r\n";
    stdin
        .write_all(oversized_frame.as_bytes())
        .expect("oversized frame header should write");
    stdin.flush().expect("oversized frame header should flush");
    drop(stdin);

    let output = child
        .wait_with_output()
        .expect("mcp server should terminate after oversized frame");
    assert_eq!(
        output.status.code(),
        Some(2),
        "oversized framed content-length should fail closed with parse/io exit status",
    );

    let stderr = String::from_utf8_lossy(output.stderr.as_slice());
    assert!(
        stderr.contains("content-length exceeds maximum"),
        "stderr should include oversized content-length marker: {stderr}",
    );
}
