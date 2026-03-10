use super::super::super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if write_send_message_response(stream, method, path, body)? {
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
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/messages/send" {
        return Ok(false);
    }
    let message_id = format!("msg-local-{:016x}", deterministic_tag(body.as_bytes()));
    let payload = format!(
        "{{\"message_id\":\"{}\",\"status\":\"created\",\"runtime_mode\":\"api\"}}",
        message_id
    );
    write_http_response(stream, 202, payload.as_str())?;
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
