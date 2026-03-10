use super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if write_submit_response(stream, method, path, body)? {
        return Ok(true);
    }
    if write_forward_response(stream, method, path)? {
        return Ok(true);
    }
    if write_query_response(stream, method, path)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_submit_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/bridge/submit" {
        return Ok(false);
    }
    let bridge_tag = deterministic_tag(body.as_bytes());
    let payload = format!(
        "{{\"bridge_id\":\"bridge-local-{bridge_tag:016x}\",\"source_message_id\":\"msg-bridge-source-{bridge_tag:016x}\",\"bridge_status\":\"submitted\"}}"
    );
    write_http_response(stream, 202, payload.as_str())?;
    Ok(true)
}

fn write_forward_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "POST" || !path.starts_with("/v1/bridge/") || !path.ends_with("/forward") {
        return Ok(false);
    }
    let bridge_id = strip_suffix_id(path, "/v1/bridge/", "/forward");
    write_http_response(stream, 200, forwarded_payload(bridge_id).as_str())?;
    Ok(true)
}

fn write_query_response(stream: &mut TcpStream, method: &str, path: &str) -> Result<bool, String> {
    if method != "GET" || !path.starts_with("/v1/bridge/") || path == "/v1/bridge/submit" {
        return Ok(false);
    }
    let bridge_id = path.trim_start_matches("/v1/bridge/");
    write_http_response(stream, 200, forwarded_payload(bridge_id).as_str())?;
    Ok(true)
}

fn forwarded_payload(bridge_id: &str) -> String {
    format!(
        "{{\"bridge_id\":\"{}\",\"bridge_status\":\"forwarded\",\"target_message_id\":\"msg-bridge-target-{}\",\"forward_tx_hash\":\"sha256:bridge-forwarded-{}\"}}",
        bridge_id, bridge_id, bridge_id
    )
}

fn strip_suffix_id<'a>(path: &'a str, prefix: &str, suffix: &str) -> &'a str {
    path.trim_start_matches(prefix)
        .trim_end_matches(suffix)
        .trim_end_matches('/')
}
