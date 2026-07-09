use super::three_agent_receipt_spec::{agent_a_spec, agent_b_spec, ReceiptSpec};
use super::three_agent_receipt_verify_support::{
    read_receipt, require_receipt_claim_match, require_receipt_string, validate_common_receipt,
    validate_receipt_digest,
};
use super::verify_support::{extract_optional_string, ClaimView};

pub(crate) fn validate_three_agent_receipt_files(
    report_json: &str,
    claim: &ClaimView<'_>,
) -> Result<(), String> {
    validate_participant_receipt(report_json, claim, agent_a_spec())?;
    validate_participant_receipt(report_json, claim, agent_b_spec())?;
    validate_verifier_receipt(report_json, claim)
}

fn validate_participant_receipt(
    report_json: &str,
    claim: &ClaimView<'_>,
    spec: ReceiptSpec,
) -> Result<(), String> {
    let raw = read_receipt(report_json, claim, spec.artifact_entry, spec.artifact_field)?;
    validate_common_receipt(raw.as_str(), claim, spec.context)?;
    require_receipt_string(raw.as_str(), "agent", spec.agent, spec.context)?;
    require_receipt_string(raw.as_str(), "action", spec.action, spec.context)?;
    require_receipt_string(
        raw.as_str(),
        "view_scope",
        "participant-private",
        spec.context,
    )?;
    validate_participant_bindings(raw.as_str(), claim, spec)?;
    validate_receipt_digest(raw.as_str(), claim, spec.digest_field)
}

fn validate_participant_bindings(
    raw: &str,
    claim: &ClaimView<'_>,
    spec: ReceiptSpec,
) -> Result<(), String> {
    require_receipt_claim_match(raw, claim, "view_artifact", spec.view_field, spec.context)?;
    require_receipt_claim_match(
        raw,
        claim,
        "view_digest",
        spec.view_digest_field,
        spec.context,
    )?;
    require_receipt_claim_match(
        raw,
        claim,
        "participant_private_view_digest",
        spec.private_digest_field,
        spec.context,
    )?;
    require_receipt_claim_match(
        raw,
        claim,
        "public_view_digest",
        spec.public_digest_field,
        spec.context,
    )
}

fn validate_verifier_receipt(report_json: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    let context = "agent_c_verifier_observation_receipt";
    let raw = read_receipt(
        report_json,
        claim,
        context,
        "agent_c_verifier_observation_receipt_artifact",
    )?;
    validate_common_receipt(raw.as_str(), claim, context)?;
    require_receipt_string(raw.as_str(), "agent", "agent_c_verifier", context)?;
    require_receipt_string(raw.as_str(), "action", "verify_three_agent_proof", context)?;
    require_receipt_string(raw.as_str(), "view_scope", "restricted-public", context)?;
    validate_verifier_bindings(raw.as_str(), claim, context)?;
    validate_receipt_digest(
        raw.as_str(),
        claim,
        "agent_c_verifier_observation_receipt_digest",
    )
}

fn validate_verifier_bindings(
    raw: &str,
    claim: &ClaimView<'_>,
    context: &str,
) -> Result<(), String> {
    require_receipt_claim_match(
        raw,
        claim,
        "view_artifact",
        "agent_c_verifier_view_artifact",
        context,
    )?;
    require_receipt_claim_match(
        raw,
        claim,
        "view_digest",
        "agent_c_verifier_view_digest",
        context,
    )?;
    require_receipt_claim_match(
        raw,
        claim,
        "public_view_digest",
        "verifier_public_view_digest",
        context,
    )?;
    if extract_optional_string(raw, "participant_private_view_digest").is_some() {
        return Err(format!("{context} exposes participant_private_view_digest"));
    }
    Ok(())
}
