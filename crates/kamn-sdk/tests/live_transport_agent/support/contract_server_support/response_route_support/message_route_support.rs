use super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    state: &ContractServerState,
) -> Result<bool, String> {
    if write_send_message_response(stream, method, path, body, state)? {
        return Ok(true);
    }
    if write_get_message_response(stream, method, path)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_send_message_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    state: &ContractServerState,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/messages/send" {
        return Ok(false);
    }
    if let Some(expected_body) = state.expected_message_body() {
        verify_message_body(body, expected_body)?;
    }
    write_http_response(
        stream,
        202,
        r#"{"message_id":"msg-live-contract-001","status":"created","runtime_mode":"api"}"#,
    )?;
    Ok(true)
}

fn write_get_message_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "GET" || !path.starts_with("/v1/messages/") {
        return Ok(false);
    }
    let message_id = path.trim_start_matches("/v1/messages/");
    let payload = format!(
        "{{\"message_id\":\"{}\",\"status\":\"created\"}}",
        message_id
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn verify_message_body(body: &str, expected_body: &str) -> Result<(), String> {
    if body == expected_body {
        return Ok(());
    }
    Err(format!(
        "message payload mismatch, expected `{expected_body}` got `{body}`"
    ))
}
