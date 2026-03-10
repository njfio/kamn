use super::super::support::*;

pub(super) fn run_malformed_message_status_server(bind_addr: String) -> Result<(), String> {
    let listener =
        TcpListener::bind(bind_addr.as_str()).map_err(|error| format!("bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("nonblocking setup failed: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    let mut replay_guard = BTreeSet::new();
    while served < 2 {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        served = served.saturating_add(serve_malformed_once(&listener, &mut replay_guard)?);
    }
    Ok(())
}

fn serve_malformed_once(
    listener: &TcpListener,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<u64, String> {
    match listener.accept() {
        Ok((mut stream, _)) => serve_malformed_connection(&mut stream, replay_guard),
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock) => {
            thread::sleep(Duration::from_millis(5));
            Ok(0)
        }
        Err(error) => Err(format!("accept failed: {error}")),
    }
}

fn serve_malformed_connection(
    stream: &mut TcpStream,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<u64, String> {
    let (method, path, body, headers) = parse_http_request(stream)?;
    if let Err((status, error, reason_code, message)) = validate_auth(
        method.as_str(),
        path.as_str(),
        body.as_str(),
        &headers,
        replay_guard,
        "kamn:did:agent:live-tester",
    ) {
        let payload = format!(
            "{{\"error\":\"{error}\",\"reason_code\":\"{reason_code}\",\"message\":\"{message}\"}}"
        );
        write_http_response(stream, status, payload.as_str())?;
        return Ok(1);
    }
    write_malformed_response(stream, &method, &path)?;
    Ok(1)
}

fn write_malformed_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<(), String> {
    if method == "POST" && path == "/v1/messages/send" {
        return write_http_response(
            stream,
            202,
            r#"{"message_id":"msg-live-contract-001","status":"created","runtime_mode":"api"}"#,
        );
    }
    if method == "GET" && path == "/v1/messages/msg-live-contract-001" {
        return write_http_response(stream, 200, r#"{"message_id":"msg-live-contract-001"}"#);
    }
    Err(format!("unexpected route {method} {path}"))
}
