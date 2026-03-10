use super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if write_agent_response(stream, method, path)? {
        return Ok(true);
    }
    write_content_response(stream, method, path, body)
}

fn write_agent_response(stream: &mut TcpStream, method: &str, path: &str) -> Result<bool, String> {
    if method == "POST" && path == "/v1/agents/register" {
        let payload = r#"{"did":"kamn:did:agent:sdk-register","reputation_score":500,"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}"#;
        write_http_response(stream, 201, payload)?;
        return Ok(true);
    }
    if method == "POST" && path == "/v1/agents/search" {
        let payload = r#"[{"did":"kamn:did:agent:sdk-register","reputation_score":500,"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}]"#;
        write_http_response(stream, 200, payload)?;
        return Ok(true);
    }
    if method == "GET" && path.starts_with("/v1/agents/") {
        let did = path.trim_start_matches("/v1/agents/");
        let payload = format!(
            "{{\"did\":\"{}\",\"reputation_score\":500,\"agent_type\":\"service-agent\",\"model_family\":\"service-api\",\"capabilities\":[\"profile:read\"]}}",
            did
        );
        write_http_response(stream, 200, payload.as_str())?;
        return Ok(true);
    }
    Ok(false)
}

fn write_content_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method == "POST" && path == "/v1/content/register" {
        let payload = r#"{"content_id":"content-local-sdk","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}"#;
        write_http_response(stream, 201, payload)?;
        return Ok(true);
    }
    if method == "GET" && path.starts_with("/v1/content/") {
        let content_id = path.trim_start_matches("/v1/content/");
        let payload = format!(
            "{{\"content_id\":\"{}\",\"lifecycle_state\":\"retained\",\"redaction_status\":\"none\"}}",
            content_id
        );
        write_http_response(stream, 200, payload.as_str())?;
        return Ok(true);
    }
    write_content_mutation_response(stream, method, path, body)
}

fn write_content_mutation_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    _body: &str,
) -> Result<bool, String> {
    if write_expire_response(stream, method, path)? {
        return Ok(true);
    }
    if write_tombstone_response(stream, method, path)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_expire_response(stream: &mut TcpStream, method: &str, path: &str) -> Result<bool, String> {
    if method != "POST" || !path.starts_with("/v1/content/") || !path.ends_with("/expire") {
        return Ok(false);
    }
    let content_id = strip_suffix_id(path, "/v1/content/", "/expire");
    let payload = format!(
        "{{\"content_id\":\"{}\",\"lifecycle_state\":\"expired\",\"redaction_status\":\"none\"}}",
        content_id
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn write_tombstone_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "POST" || !path.starts_with("/v1/content/") || !path.ends_with("/tombstone") {
        return Ok(false);
    }
    let content_id = strip_suffix_id(path, "/v1/content/", "/tombstone");
    let payload = format!(
        "{{\"content_id\":\"{}\",\"lifecycle_state\":\"tombstoned\",\"redaction_status\":\"redacted\"}}",
        content_id
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn strip_suffix_id<'a>(path: &'a str, prefix: &str, suffix: &str) -> &'a str {
    path.trim_start_matches(prefix)
        .trim_end_matches(suffix)
        .trim_end_matches('/')
}
