use super::artifact_digest::validate_json_digest;
use super::verify_support::{
    extract_bool, extract_string, extract_u64, require_marker, validate_json_delimiters, ClaimView,
};

pub(super) fn read_receipt(
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

pub(super) fn validate_common_receipt(
    raw: &str,
    claim: &ClaimView<'_>,
    context: &str,
) -> Result<(), String> {
    validate_json_delimiters(raw)?;
    reject_raw_private_payload(raw, context)?;
    require_marker(
        raw,
        "\"schema_version\":\"kamn.mvp.three-agent-observation-receipt.v1\"",
        context,
    )?;
    require_matching_string(raw, claim, "transaction_id", context)?;
    require_matching_string(raw, claim, "escrow_id", context)?;
    require_matching_string(raw, claim, "settlement_tx_signature", context)?;
    require_matching_u64(raw, claim, "amount_lamports", context)?;
    require_matching_string(raw, claim, "payer_pubkey", context)?;
    require_matching_string(raw, claim, "recipient_pubkey", context)?;
    require_matching_string(raw, claim, "settlement_commitment", context)?;
    require_matching_bool(raw, claim, "private_payload_redacted", context)
}

pub(super) fn validate_receipt_digest(
    raw: &str,
    claim: &ClaimView<'_>,
    claim_field: &str,
) -> Result<(), String> {
    let artifact_digest = extract_string(raw, "receipt_digest")?;
    let claim_digest = extract_string(claim.raw, claim_field)?;
    if artifact_digest != claim_digest {
        return Err(format!("{claim_field} mismatch"));
    }
    validate_json_digest(raw, "receipt_digest", claim_digest.as_str(), claim_field)
}

pub(super) fn require_receipt_string(
    raw: &str,
    field: &str,
    expected: &str,
    context: &str,
) -> Result<(), String> {
    if extract_string(raw, field)? == expected {
        return Ok(());
    }
    Err(format!("{context} {field} mismatch"))
}

pub(super) fn require_receipt_claim_match(
    raw: &str,
    claim: &ClaimView<'_>,
    receipt_field: &str,
    claim_field: &str,
    context: &str,
) -> Result<(), String> {
    if extract_string(raw, receipt_field)? == extract_string(claim.raw, claim_field)? {
        return Ok(());
    }
    Err(format!("{context} {receipt_field} mismatch"))
}

fn reject_raw_private_payload(raw: &str, context: &str) -> Result<(), String> {
    if raw.contains("raw_private_payload") {
        return Err(format!("{context} contains raw private payload"));
    }
    Ok(())
}

fn require_matching_string(
    raw: &str,
    claim: &ClaimView<'_>,
    field: &str,
    context: &str,
) -> Result<(), String> {
    require_receipt_claim_match(raw, claim, field, field, context)
}

fn require_matching_u64(
    raw: &str,
    claim: &ClaimView<'_>,
    field: &str,
    context: &str,
) -> Result<(), String> {
    if extract_u64(raw, field)? == extract_u64(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("{context} {field} mismatch"))
}

fn require_matching_bool(
    raw: &str,
    claim: &ClaimView<'_>,
    field: &str,
    context: &str,
) -> Result<(), String> {
    if extract_bool(raw, field)? == extract_bool(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("{context} {field} mismatch"))
}
