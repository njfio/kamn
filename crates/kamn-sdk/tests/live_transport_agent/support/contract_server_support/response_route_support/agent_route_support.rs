use super::*;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    state: &mut ContractServerState,
) -> Result<bool, String> {
    if write_register_response(stream, method, path, body, state)? {
        return Ok(true);
    }
    if write_search_response(stream, method, path, body)? {
        return Ok(true);
    }
    if write_profile_response(stream, method, path, state)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_register_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    state: &mut ContractServerState,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/agents/register" {
        return Ok(false);
    }
    let metadata = parse_registration_metadata(body)?;
    state.set_registered_metadata(metadata.clone());
    let payload = registration_payload(state.expected_agent_sender_did(), &metadata)?;
    write_http_response(stream, 201, payload.as_str())?;
    Ok(true)
}

fn write_search_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/agents/search" {
        return Ok(false);
    }
    let payload = filtered_search_payload(body)?;
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn write_profile_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    state: &ContractServerState,
) -> Result<bool, String> {
    if method != "GET" || !path.starts_with("/v1/agents/") {
        return Ok(false);
    }
    let did = path.trim_start_matches("/v1/agents/");
    let metadata = state
        .registered_metadata()
        .cloned()
        .unwrap_or_else(default_profile_metadata);
    let payload = registration_payload(did, &metadata)?;
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn parse_registration_metadata(body: &str) -> Result<(String, String, Vec<String>), String> {
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("registration payload should be valid json: {error}"))?;
    let agent_type = required_string(
        &parsed,
        "agent_type",
        "registration payload missing agent_type",
    )?;
    let model_family = required_string(
        &parsed,
        "model_family",
        "registration payload missing model_family",
    )?;
    let capabilities = parse_capabilities(&parsed)?;
    Ok((agent_type, model_family, capabilities))
}

fn required_string(
    parsed: &serde_json::Value,
    field: &str,
    error_message: &str,
) -> Result<String, String> {
    parsed
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| error_message.to_owned())
}

fn parse_capabilities(parsed: &serde_json::Value) -> Result<Vec<String>, String> {
    parsed
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "registration payload missing capabilities".to_owned())?
        .iter()
        .map(parse_capability)
        .collect()
}

fn parse_capability(value: &serde_json::Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "registration capability must be string".to_owned())
}

fn registration_payload(
    did: &str,
    metadata: &(String, String, Vec<String>),
) -> Result<String, String> {
    let (agent_type, model_family, capabilities) = metadata;
    Ok(format!(
        "{{\"did\":\"{}\",\"reputation_score\":777,\"agent_type\":\"{}\",\"model_family\":\"{}\",\"capabilities\":{}}}",
        did,
        agent_type,
        model_family,
        serde_json::to_string(capabilities)
            .map_err(|error| format!("capability serialization failed: {error}"))?
    ))
}

fn filtered_search_payload(body: &str) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("search payload should be valid json: {error}"))?;
    let capability = parsed.get("capability").and_then(serde_json::Value::as_str);
    let model_family = parsed
        .get("model_family")
        .and_then(serde_json::Value::as_str);
    let filtered: Vec<serde_json::Value> = candidate_rows()
        .into_iter()
        .filter(|row| model_family_matches(row, model_family))
        .filter(|row| capability_matches(row, capability))
        .collect();
    serde_json::to_string(&filtered)
        .map_err(|error| format!("search result serialization failed: {error}"))
}

fn candidate_rows() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "did": "kamn:did:agent:alpha",
            "reputation_score": 777,
            "agent_type": "assistant",
            "model_family": "gpt-5",
            "capabilities": ["text", "code"],
        }),
        serde_json::json!({
            "did": "kamn:did:agent:beta",
            "reputation_score": 650,
            "agent_type": "assistant",
            "model_family": "gpt-4.1",
            "capabilities": ["text"],
        }),
    ]
}

fn model_family_matches(row: &serde_json::Value, expected: Option<&str>) -> bool {
    match expected {
        Some(expected) => {
            row.get("model_family").and_then(serde_json::Value::as_str) == Some(expected)
        }
        None => true,
    }
}

fn capability_matches(row: &serde_json::Value, expected: Option<&str>) -> bool {
    match expected {
        Some(expected) => row
            .get("capabilities")
            .and_then(serde_json::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .any(|value| value.as_str().map(str::trim) == Some(expected))
            })
            .unwrap_or(false),
        None => true,
    }
}

fn default_profile_metadata() -> (String, String, Vec<String>) {
    (
        "service-agent".to_owned(),
        "service-api".to_owned(),
        vec!["profile:read".to_owned()],
    )
}
