use super::verify_support::{
    extract_bool, extract_optional_string, extract_string, extract_u64, ClaimView,
};

pub(crate) fn validate_three_agent_view_disclosure(claim: &ClaimView<'_>) -> Result<(), String> {
    validate_private_view_visibility(claim)?;
    validate_view_scopes(claim)?;
    validate_private_field_counts(claim)?;
    validate_private_digest_boundaries(claim)?;
    validate_public_view_digests(claim)?;
    validate_private_payload_redaction(claim)
}

fn validate_private_view_visibility(claim: &ClaimView<'_>) -> Result<(), String> {
    if !extract_bool(claim.raw, "agent_a_private_view_visible")?
        || !extract_bool(claim.raw, "agent_b_private_view_visible")?
    {
        return Err("participant private views must be visible to participants".to_owned());
    }
    if extract_bool(claim.raw, "verifier_private_view_visible")? {
        return Err("verifier view must not expose private fields".to_owned());
    }
    Ok(())
}

fn validate_view_scopes(claim: &ClaimView<'_>) -> Result<(), String> {
    require_scope(claim, "agent_a_view_scope", "participant-private")?;
    require_scope(claim, "agent_b_view_scope", "participant-private")?;
    require_scope(claim, "verifier_view_scope", "restricted-public")
}

fn require_scope(claim: &ClaimView<'_>, field: &str, expected: &str) -> Result<(), String> {
    if extract_string(claim.raw, field)? == expected {
        return Ok(());
    }
    Err(format!("three-agent view scope mismatch: {field}"))
}

fn validate_private_field_counts(claim: &ClaimView<'_>) -> Result<(), String> {
    let verifier = extract_u64(claim.raw, "verifier_private_field_count")?;
    if verifier != 0 {
        return Err("verifier view must not expose private fields".to_owned());
    }
    validate_participant_private_count(claim, "agent_a_private_field_count", verifier)?;
    validate_participant_private_count(claim, "agent_b_private_field_count", verifier)
}

fn validate_participant_private_count(
    claim: &ClaimView<'_>,
    field: &str,
    verifier_count: u64,
) -> Result<(), String> {
    if extract_u64(claim.raw, field)? > verifier_count {
        return Ok(());
    }
    Err("participant views must include private evidence".to_owned())
}

fn validate_private_digest_boundaries(claim: &ClaimView<'_>) -> Result<(), String> {
    extract_string(claim.raw, "agent_a_private_view_digest")?;
    extract_string(claim.raw, "agent_b_private_view_digest")?;
    if extract_optional_string(claim.raw, "verifier_private_view_digest").is_none() {
        return Ok(());
    }
    Err("verifier view must not expose private digest".to_owned())
}

fn validate_public_view_digests(claim: &ClaimView<'_>) -> Result<(), String> {
    let expected = extract_string(claim.raw, "agent_a_public_view_digest")?;
    if extract_string(claim.raw, "agent_b_public_view_digest")? != expected {
        return Err("three-agent public view digest mismatch".to_owned());
    }
    if extract_string(claim.raw, "verifier_public_view_digest")? == expected {
        return Ok(());
    }
    Err("three-agent public view digest mismatch".to_owned())
}

fn validate_private_payload_redaction(claim: &ClaimView<'_>) -> Result<(), String> {
    if extract_bool(claim.raw, "private_payload_redacted")? {
        return Ok(());
    }
    Err("private payloads must be redacted".to_owned())
}
