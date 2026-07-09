use super::verify_support::{extract_string, require_marker, ClaimView};

pub(crate) fn validate_actor_rehearsal(
    artifact: &str,
    claim: &ClaimView<'_>,
) -> Result<(), String> {
    let rehearsal = object_section(artifact, "three_agent_actor_rehearsal")?;
    require_marker(
        rehearsal,
        "\"settlement_claim_label\":\"devnet-backed\"",
        "three_agent_actor_rehearsal settlement label",
    )?;
    require_marker(
        rehearsal,
        "\"settlement_status\":\"PASS\"",
        "three_agent_actor_rehearsal settlement status",
    )?;
    require_marker(
        rehearsal,
        "\"private_payload_redacted\":true",
        "three_agent_actor_rehearsal redaction",
    )?;
    reject_private_payload(rehearsal)?;
    validate_participant(rehearsal, claim, "agent_a", "invoke_transaction")?;
    validate_participant(rehearsal, claim, "agent_b", "accept_task")?;
    validate_verifier(rehearsal, claim)
}

fn validate_participant(
    rehearsal: &str,
    claim: &ClaimView<'_>,
    agent: &str,
    action: &str,
) -> Result<(), String> {
    let actor = object_section(rehearsal, agent)?;
    require_marker(actor, format!("\"agent\":\"{agent}\"").as_str(), agent)?;
    require_marker(actor, "\"register\"", agent)?;
    require_marker(actor, action, agent)?;
    require_marker(actor, "\"view_scope\":\"participant-private\"", agent)?;
    require_actor_match(
        actor,
        claim,
        "view_artifact",
        format!("{agent}_view_artifact").as_str(),
    )?;
    require_actor_match(
        actor,
        claim,
        format!("{agent}_view_digest").as_str(),
        format!("{agent}_view_digest").as_str(),
    )
}

fn validate_verifier(rehearsal: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    let actor = object_section(rehearsal, "agent_c_verifier")?;
    require_marker(actor, "\"agent\":\"agent_c_verifier\"", "agent_c_verifier")?;
    require_marker(actor, "\"verify_proof\"", "agent_c_verifier")?;
    require_marker(
        actor,
        "\"view_scope\":\"restricted-public\"",
        "agent_c_verifier",
    )?;
    reject_verifier_private(actor)?;
    require_actor_match(
        actor,
        claim,
        "view_artifact",
        "agent_c_verifier_view_artifact",
    )?;
    require_actor_match(
        actor,
        claim,
        "agent_c_verifier_view_digest",
        "agent_c_verifier_view_digest",
    )
}

fn require_actor_match(
    actor: &str,
    claim: &ClaimView<'_>,
    actor_field: &str,
    claim_field: &str,
) -> Result<(), String> {
    if extract_string(actor, actor_field)? == extract_string(claim.raw, claim_field)? {
        return Ok(());
    }
    Err(format!(
        "three_agent_actor_rehearsal {claim_field} mismatch"
    ))
}

fn object_section<'a>(raw: &'a str, field: &str) -> Result<&'a str, String> {
    let marker = format!("\"{field}\":{{");
    let start = raw
        .find(marker.as_str())
        .ok_or_else(|| format!("missing three_agent_actor_rehearsal field: {field}"))?
        + marker.len()
        - 1;
    matching_object(raw, start)
}

fn matching_object(raw: &str, start: usize) -> Result<&str, String> {
    let mut depth = 0_u64;
    for (offset, byte) in raw[start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' if depth == 0 => break,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(&raw[start..start + offset + 1]);
                }
            }
            _ => {}
        }
    }
    Err("malformed three_agent_actor_rehearsal object".to_owned())
}

fn reject_private_payload(raw: &str) -> Result<(), String> {
    if raw.contains("raw_private_payload") {
        return Err("three_agent_actor_rehearsal contains raw private payload".to_owned());
    }
    Ok(())
}

fn reject_verifier_private(actor: &str) -> Result<(), String> {
    if actor.contains("\"participant_private_view_digest\"") {
        return Err("three_agent_actor_rehearsal agent_c_verifier private digest".to_owned());
    }
    if actor.contains("\"view_scope\":\"participant-private\"") {
        return Err("three_agent_actor_rehearsal agent_c_verifier scope".to_owned());
    }
    Ok(())
}
