use super::*;

pub(super) fn write_public_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    websocket_payload: &str,
) -> Result<bool, String> {
    if write_health_response(stream, method, path)? {
        return Ok(true);
    }
    if write_metrics_response(stream, method, path)? {
        return Ok(true);
    }
    if write_websocket_response(stream, method, path, headers, websocket_payload)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_health_response(stream: &mut TcpStream, method: &str, path: &str) -> Result<bool, String> {
    if method != "GET" || path != "/healthz" {
        return Ok(false);
    }
    write_http_response(
        stream,
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    )?;
    Ok(true)
}

fn write_metrics_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "GET" || path != "/metrics" {
        return Ok(false);
    }
    let body = "kamn_service_api_health{runtime_mode=\"api\"} 1\n";
    let body_len = body.len();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n{body}"
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("metrics write failed: {error}"))?;
    Ok(true)
}

fn write_websocket_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    websocket_payload: &str,
) -> Result<bool, String> {
    if method != "GET" || path != "/v1/events/ws" {
        return Ok(false);
    }
    if websocket_upgrade_valid(headers) {
        write_websocket_upgrade_response(stream, websocket_payload)?;
    } else {
        write_http_response(
            stream,
            400,
            r#"{"error":"bad-request","reason_code":"service_api_websocket_upgrade_required","message":"websocket upgrade required"}"#,
        )?;
    }
    Ok(true)
}

fn websocket_upgrade_valid(headers: &BTreeMap<String, String>) -> bool {
    let upgrade = headers.get("upgrade").cloned().unwrap_or_default();
    let connection = headers.get("connection").cloned().unwrap_or_default();
    let websocket_key = headers
        .get("sec-websocket-key")
        .cloned()
        .unwrap_or_default();
    let version = headers
        .get("sec-websocket-version")
        .cloned()
        .unwrap_or_default();
    upgrade.eq_ignore_ascii_case("websocket")
        && connection.to_ascii_lowercase().contains("upgrade")
        && !websocket_key.trim().is_empty()
        && version.trim() == "13"
}
