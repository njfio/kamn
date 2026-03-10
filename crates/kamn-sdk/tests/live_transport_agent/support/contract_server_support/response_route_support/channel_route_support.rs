use super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if !is_create_channel_route(method, path) {
        return Ok(false);
    }
    let name = parse_channel_name(body)?;
    let payload = channel_payload(name);
    write_http_response(stream, 201, payload.as_str())?;
    Ok(true)
}

fn is_create_channel_route(method: &str, path: &str) -> bool {
    method == "POST" && path == "/v1/channels/create"
}

fn parse_channel_name(body: &str) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("channel payload should be valid json: {error}"))?;
    parsed
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "channel payload missing name".to_owned())
}

fn channel_payload(name: String) -> String {
    if name == "empty-channel" {
        return r#"{"channel_id":"","status":"created"}"#.to_owned();
    }
    format!(
        "{{\"channel_id\":\"channel-live-{}\",\"status\":\"created\"}}",
        name
    )
}
