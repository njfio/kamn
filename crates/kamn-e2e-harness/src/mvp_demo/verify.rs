use super::agent_harness::validate_agent_harness_claim_shape;
use super::report::{CLAIM_LABEL_DEVNET_BACKED, MVP_DEMO_REPORT_SCHEMA_VERSION};
use super::three_agent_verify::validate_three_agent_escrow_verification;
use super::verify_claims::{
    validate_authoritative_label, validate_claim_label, validate_required_claims,
    validate_value_movement_label,
};
use super::verify_support::{parse_claims, require_marker, validate_json_delimiters, ClaimView};

/// Verifies a rendered MVP demo proof report JSON payload.
pub fn verify_mvp_demo_report_json(report_json: impl AsRef<str>) -> Result<(), String> {
    let report_json = report_json.as_ref();
    validate_report_shape(report_json)?;
    let claims = parse_claims(report_json)?;
    validate_required_claims(&claims)?;
    validate_agent_transaction_claim_presence(report_json, &claims)?;
    for claim in &claims {
        validate_claim_label(claim.label.as_str())?;
        validate_authoritative_label(claim)?;
        validate_value_movement_label(claim)?;
        validate_devnet_evidence(claim)?;
    }
    validate_agent_harness_claim_shape(&claims)?;
    validate_three_agent_escrow_verification(&claims)?;
    validate_no_go(report_json)
}

fn validate_agent_transaction_claim_presence(
    report: &str,
    claims: &[ClaimView<'_>],
) -> Result<(), String> {
    let has_artifacts = report.contains("\"three_agent_transcript\":\"")
        || report.contains("\"live_task_settlement_binding\":\"")
        || report.contains("\"runtime_agent_a_evidence\":\"");
    let has_claim = claims
        .iter()
        .any(|claim| claim.id == "three_agent_escrow_verification");
    if !has_artifacts || has_claim {
        return Ok(());
    }
    Err("AGENT_TRANSACTION_CLAIM_INVALID".to_owned())
}

fn validate_report_shape(report_json: &str) -> Result<(), String> {
    validate_json_delimiters(report_json)?;
    validate_report_schema(report_json)?;
    validate_artifact_markers(report_json)?;
    require_marker(report_json, "\"claim_matrix\":[", "claim_matrix")
}

fn validate_report_schema(report_json: &str) -> Result<(), String> {
    require_marker(
        report_json,
        MVP_DEMO_REPORT_SCHEMA_VERSION,
        "schema_version",
    )
}

fn validate_artifact_markers(report_json: &str) -> Result<(), String> {
    require_marker(report_json, "\"artifacts\":{", "artifacts")?;
    for (marker, name) in required_artifact_markers() {
        require_marker(report_json, marker, name)?;
    }
    Ok(())
}

fn required_artifact_markers() -> [(&'static str, &'static str); 5] {
    [
        (
            "\"localhost_signed_demo_artifact\":\"",
            "localhost signed demo artifact",
        ),
        (
            "\"localhost_signed_demo_output\":\"",
            "localhost signed demo output",
        ),
        (
            "\"service_api_vertical_slice_output\":\"",
            "service API vertical slice output",
        ),
        (
            "\"service_api_websocket_output\":\"",
            "service API websocket output",
        ),
        (
            "\"devnet_settlement_output\":\"",
            "devnet settlement output",
        ),
    ]
}

fn validate_devnet_evidence(claim: &ClaimView<'_>) -> Result<(), String> {
    if claim.label != CLAIM_LABEL_DEVNET_BACKED {
        return Ok(());
    }
    if claim.status == "NO-GO" {
        return require_marker(claim.raw, "\"no_go_reason\":\"", "devnet no-go reason");
    }
    require_devnet_success_markers(claim.raw)
}

fn require_devnet_success_markers(raw: &str) -> Result<(), String> {
    for marker in devnet_success_markers() {
        require_marker(raw, marker, "devnet-backed settlement evidence")?;
    }
    Ok(())
}

fn validate_no_go(report_json: &str) -> Result<(), String> {
    if report_json.contains("\"devnet_mode\":\"required\"")
        && !report_json.contains("\"no_go\":{\"active\":true")
        && !report_json.contains("\"status\":\"GO\"")
    {
        return Err("devnet-required report must include explicit NO-GO evidence".to_owned());
    }
    Ok(())
}

fn devnet_success_markers() -> [&'static str; 15] {
    [
        "\"network\":\"solana:devnet\"",
        "\"execution_surface\":\"",
        "\"rpc_url\":\"",
        "\"payer_pubkey\":\"",
        "\"recipient_pubkey\":\"",
        "\"lamports\":",
        "\"escrow_id\":\"",
        "\"settlement_tx_signature\":\"",
        "\"settlement_commitment\":\"",
        "\"payer_balance_before\":",
        "\"payer_balance_after\":",
        "\"recipient_balance_before\":",
        "\"recipient_balance_after\":",
        "\"persisted_settlement_tx_signature\":\"",
        "\"status\":\"PASS\"",
    ]
}
