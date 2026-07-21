use serde_json::{json, Value};

pub(crate) const INVALID: &str = "MCP_AUTHORITY_RECEIPT_INVALID";
pub(crate) const MISSING: &str = "MCP_AUTHORITY_RECEIPT_MISSING";

pub(crate) fn wrap(
    tool: &str,
    payload: &str,
    request: &str,
    expected_actor: &str,
) -> Result<Option<String>, &'static str> {
    if tool == "register" {
        return registration(payload, tool, expected_actor).map(Some);
    }
    let Some(resource_key) = mutation_resource_key(tool) else {
        return Ok(None);
    };
    mutation(payload, request, tool, resource_key, expected_actor).map(Some)
}

fn registration(payload: &str, tool: &str, expected_actor: &str) -> Result<String, &'static str> {
    let value = parse(payload)?;
    let did = required(&value, "did")?;
    let commitment = required(&value, "profile_commitment")?;
    validate_digest(commitment)?;
    validate_equal(did, expected_actor)?;
    Ok(json!({
        "schema_version": "kamn.mcp.authority-receipt.v1",
        "authority_kind": "service-profile-commitment",
        "source": "kamn-service",
        "actor_did": did,
        "tool": tool,
        "resource_id": did,
        "profile_commitment": commitment,
        "service_result": value,
    })
    .to_string())
}

fn mutation(
    payload: &str,
    request: &str,
    tool: &str,
    resource_key: &str,
    expected_actor: &str,
) -> Result<String, &'static str> {
    let value = parse(payload)?;
    let actor = required(&value, "actor_did")?;
    let resource = required(&value, resource_key)?;
    let state = required(&value, "state")?;
    let receipt_id = required(&value, "receipt_id")?;
    let digest = required(&value, "receipt_digest")?;
    let action = required(&value, "action")?;
    validate_digest(digest)?;
    validate_equal(actor, expected_actor)?;
    validate_equal(action, expected_action(tool)?)?;
    validate_state(tool, state)?;
    validate_requested_resource(request, resource_key, resource)?;
    Ok(json!({
        "schema_version": "kamn.mcp.authority-receipt.v1",
        "authority_kind": "service-receipt",
        "source": "kamn-service",
        "actor_did": actor,
        "tool": tool,
        "resource_id": resource,
        "resulting_state": state,
        "service_receipt_id": receipt_id,
        "service_receipt_digest": digest,
        "service_result": value,
    })
    .to_string())
}

fn parse(payload: &str) -> Result<Value, &'static str> {
    serde_json::from_str(payload).map_err(|_| INVALID)
}

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str, &'static str> {
    value
        .get(key)
        .ok_or(MISSING)?
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or(INVALID)
}

fn validate_digest(value: &str) -> Result<(), &'static str> {
    let hex = value.strip_prefix("sha256:").ok_or(INVALID)?;
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(INVALID)
}

fn validate_requested_resource(
    request: &str,
    resource_key: &str,
    resource: &str,
) -> Result<(), &'static str> {
    let request = parse(request)?;
    match request.get(resource_key).and_then(Value::as_str) {
        Some(expected) if expected != resource => Err(INVALID),
        _ => Ok(()),
    }
}

fn validate_equal(actual: &str, expected: &str) -> Result<(), &'static str> {
    if actual == expected {
        return Ok(());
    }
    Err(INVALID)
}

fn expected_action(tool: &str) -> Result<&'static str, &'static str> {
    match tool {
        "create_task" => Ok("task:create"),
        "accept_task" => Ok("task:accept"),
        "complete_task" => Ok("task:complete"),
        "fund_escrow" => Ok("escrow:fund"),
        "release_escrow" => Ok("escrow:release-authorize"),
        _ => Err(INVALID),
    }
}

fn validate_state(tool: &str, state: &str) -> Result<(), &'static str> {
    let valid = match tool {
        "create_task" => state == "submitted",
        "accept_task" => state == "accepted",
        "complete_task" => state == "completed",
        "fund_escrow" => state == "funded",
        "release_escrow" => matches!(state, "release-authorized" | "released"),
        _ => false,
    };
    if valid {
        return Ok(());
    }
    Err(INVALID)
}

fn mutation_resource_key(tool: &str) -> Option<&'static str> {
    match tool {
        "create_task" | "accept_task" | "complete_task" => Some("task_id"),
        "fund_escrow" | "release_escrow" => Some("escrow_id"),
        _ => None,
    }
}
