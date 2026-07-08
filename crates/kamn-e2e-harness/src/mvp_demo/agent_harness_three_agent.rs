use super::verify_support::{
    extract_bool, extract_optional_string, extract_string, extract_u64, require_marker, ClaimView,
};

const THREE_AGENT_CLAIM_ID: &str = "three_agent_escrow_verification";

pub(crate) fn validate_agent_three_agent_boundary(
    artifact: &str,
    claims: &[ClaimView<'_>],
) -> Result<(), String> {
    require_marker(
        artifact,
        "\"three_agent_boundary\":{",
        "three_agent_boundary",
    )?;
    match three_agent_claim(claims) {
        Some(claim) => validate_present_boundary(artifact, claim),
        None => validate_absent_boundary(artifact),
    }
}

fn three_agent_claim<'a>(claims: &'a [ClaimView<'a>]) -> Option<&'a ClaimView<'a>> {
    claims.iter().find(|claim| claim.id == THREE_AGENT_CLAIM_ID)
}

fn validate_present_boundary(artifact: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    require_boundary_bool(artifact, "claim_present", true)?;
    require_boundary_string(artifact, "claim_status", claim.status.as_str())?;
    require_boundary_string(artifact, "claim_label", claim.label.as_str())?;
    require_boundary_count(artifact, claim, "agent_a_private_field_count")?;
    require_boundary_count(artifact, claim, "agent_b_private_field_count")?;
    require_boundary_count(artifact, claim, "verifier_private_field_count")?;
    require_boundary_bool(
        artifact,
        "private_payload_redacted",
        extract_bool(claim.raw, "private_payload_redacted")?,
    )?;
    validate_private_counts(artifact)?;
    validate_verifier_private_digest(artifact, claim)
}

fn validate_absent_boundary(artifact: &str) -> Result<(), String> {
    require_boundary_bool(artifact, "claim_present", false)?;
    require_boundary_string(artifact, "claim_status", "NOT_PRESENT")?;
    require_boundary_string(artifact, "claim_label", "NOT_PRESENT")
}

fn require_boundary_string(artifact: &str, field: &str, expected: &str) -> Result<(), String> {
    let actual = extract_string(artifact, field)
        .map_err(|_| format!("missing three_agent_boundary field: {field}"))?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("three_agent_boundary {field} mismatch"))
}

fn require_boundary_bool(artifact: &str, field: &str, expected: bool) -> Result<(), String> {
    let actual = extract_bool(artifact, field)
        .map_err(|_| format!("missing three_agent_boundary field: {field}"))?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("three_agent_boundary {field} mismatch"))
}

fn require_boundary_count(
    artifact: &str,
    claim: &ClaimView<'_>,
    field: &str,
) -> Result<(), String> {
    let actual = boundary_count(artifact, field)?;
    if actual == extract_u64(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three_agent_boundary {field} mismatch"))
}

fn validate_private_counts(artifact: &str) -> Result<(), String> {
    let verifier = boundary_count(artifact, "verifier_private_field_count")?;
    if verifier != 0 {
        return Err("three_agent_boundary verifier_private_field_count mismatch".to_owned());
    }
    validate_participant_count(artifact, "agent_a_private_field_count", verifier)?;
    validate_participant_count(artifact, "agent_b_private_field_count", verifier)
}

fn validate_participant_count(artifact: &str, field: &str, verifier: u64) -> Result<(), String> {
    if boundary_count(artifact, field)? > verifier {
        return Ok(());
    }
    Err(format!("three_agent_boundary {field} mismatch"))
}

fn validate_verifier_private_digest(artifact: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    let expected = extract_optional_string(claim.raw, "verifier_private_view_digest").is_some();
    require_boundary_bool(artifact, "verifier_private_view_digest_present", expected)?;
    if !expected {
        return Ok(());
    }
    Err("three_agent_boundary verifier_private_view_digest_present mismatch".to_owned())
}

fn boundary_count(artifact: &str, field: &str) -> Result<u64, String> {
    extract_u64(artifact, field).map_err(|_| format!("missing three_agent_boundary field: {field}"))
}
