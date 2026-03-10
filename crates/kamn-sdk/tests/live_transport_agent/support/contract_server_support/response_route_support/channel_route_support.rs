use super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/channels/create" {
        return Ok(false);
    }
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("channel payload should be valid json: {error}"))?;
    let name = parsed
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "channel payload missing name".to_owned())?;
    let payload = if name == "empty-channel" {
        r#"{"channel_id":"","status":"created"}"#.to_owned()
    } else {
        format!(
            "{{\"channel_id\":\"channel-live-{}\",\"status\":\"created\"}}",
            name
        )
    };
    write_http_response(stream, 201, payload.as_str())?;
    Ok(true)
}
