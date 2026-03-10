use super::*;

#[path = "contract_server_support/agent_content_route_support.rs"]
mod agent_content_route_support;
#[path = "contract_server_support/bridge_route_support.rs"]
mod bridge_route_support;
#[path = "contract_server_support/message_task_route_support.rs"]
mod message_task_route_support;
#[path = "contract_server_support/public_route_support.rs"]
mod public_route_support;
#[path = "contract_server_support/route_id_support.rs"]
mod route_id_support;

pub(crate) use route_id_support::strip_suffix_id;

pub(crate) fn run_service_contract_server(
    bind_addr: String,
    max_requests: u64,
) -> Result<(), String> {
    run_service_contract_server_with_websocket_payload(
        bind_addr,
        max_requests,
        DEFAULT_WEBSOCKET_EVENT_PAYLOAD.to_owned(),
    )
}

pub(crate) fn run_service_contract_server_with_websocket_payload(
    bind_addr: String,
    max_requests: u64,
    websocket_payload: String,
) -> Result<(), String> {
    let listener = bind_listener(bind_addr.as_str())?;
    let mut replay_guard: BTreeSet<(String, u64)> = BTreeSet::new();
    serve_requests(
        listener,
        max_requests,
        websocket_payload.as_str(),
        &mut replay_guard,
    )
}

pub(crate) fn wait_for_server_ready(addr: &str) {
    assert!(!addr.trim().is_empty(), "server address must not be empty");
    thread::sleep(Duration::from_millis(40));
}

fn bind_listener(bind_addr: &str) -> Result<TcpListener, String> {
    let listener =
        TcpListener::bind(bind_addr).map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;
    Ok(listener)
}

fn serve_requests(
    listener: TcpListener,
    max_requests: u64,
    websocket_payload: &str,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        served = served.saturating_add(serve_once(&listener, websocket_payload, replay_guard)?);
    }
    Ok(())
}

fn serve_once(
    listener: &TcpListener,
    websocket_payload: &str,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<u64, String> {
    match listener.accept() {
        Ok((mut stream, _)) => serve_connection(&mut stream, websocket_payload, replay_guard),
        Err(error) if error.kind() == ErrorKind::WouldBlock => {
            thread::sleep(Duration::from_millis(5));
            Ok(0)
        }
        Err(error) => Err(format!("server accept failed: {error}")),
    }
}

fn serve_connection(
    stream: &mut TcpStream,
    websocket_payload: &str,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<u64, String> {
    let (method, path, body, headers) = parse_http_request(stream)?;
    if write_public_or_auth_failure(
        stream,
        websocket_payload,
        replay_guard,
        &method,
        &path,
        &body,
        &headers,
    )? {
        return Ok(1);
    }
    write_authenticated_response(stream, &method, &path, &body)
}

fn write_public_or_auth_failure(
    stream: &mut TcpStream,
    websocket_payload: &str,
    replay_guard: &mut BTreeSet<(String, u64)>,
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
) -> Result<bool, String> {
    if write_public_response(stream, method, path, headers, websocket_payload)? {
        return Ok(true);
    }
    if write_failed_auth_response(stream, method, path, body, headers, replay_guard)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_public_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    websocket_payload: &str,
) -> Result<bool, String> {
    public_route_support::write_public_response(stream, method, path, headers, websocket_payload)
}

fn write_failed_auth_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<bool, String> {
    if let Err((status, error, reason_code, message)) =
        validate_auth(method, path, body, headers, replay_guard)
    {
        write_auth_failure(stream, status, error, reason_code, message)?;
        return Ok(true);
    }
    Ok(false)
}

fn write_auth_failure(
    stream: &mut TcpStream,
    status: u16,
    error: &str,
    reason_code: &str,
    message: &str,
) -> Result<u64, String> {
    let payload = format!(
        "{{\"error\":\"{error}\",\"reason_code\":\"{reason_code}\",\"message\":\"{message}\"}}"
    );
    write_http_response(stream, status, payload.as_str())?;
    Ok(1)
}

fn write_authenticated_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<u64, String> {
    if message_task_route_support::write_response(stream, method, path, body)? {
        return Ok(1);
    }
    if agent_content_route_support::write_response(stream, method, path, body)? {
        return Ok(1);
    }
    if bridge_route_support::write_response(stream, method, path, body)? {
        return Ok(1);
    }
    write_http_response(
        stream,
        404,
        r#"{"error":"not-found","reason_code":"service_api_route_not_found","message":"not found"}"#,
    )?;
    Ok(1)
}
