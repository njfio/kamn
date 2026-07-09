use super::artifact_digest::validate_json_digest;
use super::verify_support::{
    extract_bool, extract_optional_string, extract_string, extract_u64, require_marker,
    validate_json_delimiters, ClaimView,
};

pub(crate) fn validate_three_agent_view_claim(claim: &ClaimView<'_>) -> Result<(), String> {
    for field in view_claim_fields() {
        extract_string(claim.raw, field)?;
    }
    Ok(())
}

pub(crate) fn validate_three_agent_view_files(
    report_json: &str,
    transcript: &str,
    claim: &ClaimView<'_>,
) -> Result<(), String> {
    let agent_a = read_view(report_json, claim, "agent_a_view", "agent_a_view_artifact")?;
    let agent_b = read_view(report_json, claim, "agent_b_view", "agent_b_view_artifact")?;
    let agent_c = read_view(
        report_json,
        claim,
        "agent_c_verifier_view",
        "agent_c_verifier_view_artifact",
    )?;
    validate_transcript_bindings(transcript, claim)?;
    validate_participant_view(agent_a.as_str(), claim, "agent_a", "agent_a_view")?;
    validate_participant_view(agent_b.as_str(), claim, "agent_b", "agent_b_view")?;
    validate_verifier_view(agent_c.as_str(), claim)
}

fn view_claim_fields() -> [&'static str; 6] {
    [
        "agent_a_view_artifact",
        "agent_b_view_artifact",
        "agent_c_verifier_view_artifact",
        "agent_a_view_digest",
        "agent_b_view_digest",
        "agent_c_verifier_view_digest",
    ]
}

fn read_view(
    report_json: &str,
    claim: &ClaimView<'_>,
    artifact_entry: &str,
    claim_field: &str,
) -> Result<String, String> {
    let report_path = extract_string(report_json, artifact_entry)?;
    let claim_path = extract_string(claim.raw, claim_field)?;
    if report_path != claim_path {
        return Err(format!("{artifact_entry} artifact entry mismatch"));
    }
    std::fs::read_to_string(claim_path.as_str())
        .map_err(|error| format!("failed to read {artifact_entry} {claim_path}: {error}"))
}

fn validate_transcript_bindings(transcript: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    for field in view_claim_fields() {
        let transcript_value = extract_string(transcript, field)?;
        if transcript_value != extract_string(claim.raw, field)? {
            return Err(format!("three-agent transcript {field} mismatch"));
        }
    }
    Ok(())
}

fn validate_participant_view(
    raw: &str,
    claim: &ClaimView<'_>,
    agent: &str,
    artifact: &str,
) -> Result<(), String> {
    validate_common_view(raw, claim)?;
    require_agent_identity(raw, agent, artifact)?;
    require_marker(raw, "\"view_scope\":\"participant-private\"", agent)?;
    require_marker(raw, "\"participant_private_view_digest\":\"", agent)?;
    if extract_u64(raw, "private_field_count")? == 0 {
        return Err(format!("{agent} private_field_count mismatch"));
    }
    validate_view_digest(raw, claim, format!("{agent}_view_digest").as_str())?;
    validate_artifact_digest(
        raw,
        claim,
        "participant_private_view_digest",
        format!("{agent}_private_view_digest").as_str(),
    )?;
    validate_artifact_digest(
        raw,
        claim,
        "public_view_digest",
        format!("{agent}_public_view_digest").as_str(),
    )
}

fn validate_verifier_view(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    validate_common_view(raw, claim)?;
    require_agent_identity(raw, "agent_c_verifier", "agent_c_verifier_view")?;
    require_marker(
        raw,
        "\"view_scope\":\"restricted-public\"",
        "agent_c_verifier_view",
    )?;
    if extract_u64(raw, "private_field_count")? != 0 {
        return Err("agent_c_verifier_view private_field_count mismatch".to_owned());
    }
    if extract_optional_string(raw, "participant_private_view_digest").is_some() {
        return Err("agent_c_verifier_view exposes participant private digest".to_owned());
    }
    validate_view_digest(raw, claim, "agent_c_verifier_view_digest")?;
    validate_artifact_digest(
        raw,
        claim,
        "public_view_digest",
        "verifier_public_view_digest",
    )
}

fn validate_common_view(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    validate_json_delimiters(raw)?;
    reject_private_payload(raw)?;
    require_marker(
        raw,
        "\"schema_version\":\"kamn.mvp.three-agent-view.v1\"",
        "view",
    )?;
    require_matching_string(raw, claim, "transaction_id")?;
    require_matching_string(raw, claim, "escrow_id")?;
    require_matching_string(raw, claim, "settlement_tx_signature")?;
    require_matching_u64(raw, claim, "amount_lamports")?;
    require_matching_string(raw, claim, "payer_pubkey")?;
    require_matching_string(raw, claim, "recipient_pubkey")?;
    require_matching_string(raw, claim, "settlement_commitment")?;
    require_matching_bool(raw, claim, "private_payload_redacted")
}

fn require_agent_identity(raw: &str, expected: &str, artifact: &str) -> Result<(), String> {
    let actual = extract_string(raw, "agent")?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("{artifact} agent identity mismatch"))
}

fn reject_private_payload(raw: &str) -> Result<(), String> {
    if raw.contains("raw_private_payload") {
        return Err("three-agent view artifact contains raw private payload".to_owned());
    }
    Ok(())
}

fn validate_view_digest(raw: &str, claim: &ClaimView<'_>, claim_field: &str) -> Result<(), String> {
    validate_artifact_digest(raw, claim, "view_digest", claim_field)?;
    validate_json_digest(
        raw,
        "view_digest",
        extract_string(claim.raw, claim_field)?.as_str(),
        format!("three-agent view {claim_field}").as_str(),
    )
}

fn validate_artifact_digest(
    raw: &str,
    claim: &ClaimView<'_>,
    artifact_field: &str,
    claim_field: &str,
) -> Result<(), String> {
    if extract_string(raw, artifact_field)? == extract_string(claim.raw, claim_field)? {
        return Ok(());
    }
    Err(format!("three-agent view {claim_field} mismatch"))
}

fn require_matching_string(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_string(raw, field)? == extract_string(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three-agent view {field} mismatch"))
}

fn require_matching_u64(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_u64(raw, field)? == extract_u64(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three-agent view {field} mismatch"))
}

fn require_matching_bool(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_bool(raw, field)? == extract_bool(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three-agent view {field} mismatch"))
}
