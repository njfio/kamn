use super::super::super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if write_channel_response(stream, method, path, body)? {
        return Ok(true);
    }
    if write_channel_messages_response(stream, method, path)? {
        return Ok(true);
    }
    if write_create_task_response(stream, method, path, body)? {
        return Ok(true);
    }
    if write_get_task_response(stream, method, path)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_channel_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/channels/create" {
        return Ok(false);
    }
    let channel_id = format!("channel-local-{:016x}", deterministic_tag(body.as_bytes()));
    let payload = format!("{{\"channel_id\":\"{channel_id}\",\"status\":\"created\"}}");
    write_http_response(stream, 201, payload.as_str())?;
    Ok(true)
}

fn write_channel_messages_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "GET" || !path.starts_with("/v1/channels/") || !path.ends_with("/messages") {
        return Ok(false);
    }
    let channel_id = path
        .trim_start_matches("/v1/channels/")
        .trim_end_matches("/messages")
        .trim_end_matches('/');
    let payload = format!(
        "{{\"channel_id\":\"{channel_id}\",\"messages\":[\"msg-local-a\",\"msg-local-b\"]}}"
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn write_create_task_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/tasks/create" {
        return Ok(false);
    }
    let task_id = format!("task-local-{:016x}", deterministic_tag(body.as_bytes()));
    let payload = format!(
        "{{\"task_id\":\"{task_id}\",\"state\":\"submitted\",\"receipt_id\":\"task-transition-receipt-create\",\"receipt_digest\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"action\":\"task:create\"}}"
    );
    write_http_response(stream, 201, payload.as_str())?;
    Ok(true)
}

fn write_get_task_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "GET" || !path.starts_with("/v1/tasks/") {
        return Ok(false);
    }
    let task_id = path.trim_start_matches("/v1/tasks/");
    let payload = format!("{{\"task_id\":\"{task_id}\",\"state\":\"submitted\"}}");
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}
