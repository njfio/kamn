use super::report::{
    CLAIM_LABEL_DEVNET_BACKED, CLAIM_LABEL_DRY_RUN, CLAIM_LABEL_LOCAL_ONLY,
    CLAIM_LABEL_PLACEHOLDER, CLAIM_LABEL_REAL, CLAIM_LABEL_ROADMAP, MVP_DEMO_REPORT_SCHEMA_VERSION,
};
use super::verify_support::{
    extract_bool, extract_string, extract_u64, parse_claims, require_marker,
    validate_json_delimiters, ClaimView,
};

const REQUIRED_CLAIMS: &[&str] = &[
    "local_runtime_startup",
    "authenticated_agent_identities",
    "signed_message_or_task_flow",
    "durable_state_written",
    "relay_projection_visible",
    "websocket_event_visibility",
    "audit_proof_export",
];

const VALUE_TERMS: &[&str] = &[
    "exchange",
    "escrow",
    "settlement",
    "transfer",
    "lamports",
    "asset",
    "value movement",
];

/// Verifies a rendered MVP demo proof report JSON payload.
pub fn verify_mvp_demo_report_json(report_json: impl AsRef<str>) -> Result<(), String> {
    let report_json = report_json.as_ref();
    validate_report_shape(report_json)?;
    let claims = parse_claims(report_json)?;
    validate_required_claims(&claims)?;
    for claim in &claims {
        validate_claim_label(claim.label.as_str())?;
        validate_required_label(claim)?;
        validate_value_movement_label(claim)?;
        validate_devnet_evidence(claim)?;
    }
    validate_three_agent_escrow_verification(&claims)?;
    validate_no_go(report_json)
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

fn validate_required_claims(claims: &[ClaimView<'_>]) -> Result<(), String> {
    for required in REQUIRED_CLAIMS {
        if !claims.iter().any(|claim| claim.id == *required) {
            return Err(format!("missing required MVP claim: {required}"));
        }
    }
    Ok(())
}

fn validate_claim_label(label: &str) -> Result<(), String> {
    if allowed_labels().contains(&label) {
        return Ok(());
    }
    Err(format!("unknown MVP claim label: {label}"))
}

fn validate_required_label(claim: &ClaimView<'_>) -> Result<(), String> {
    if !claim.required {
        return Ok(());
    }
    match claim.label.as_str() {
        CLAIM_LABEL_DRY_RUN => Err("required MVP claim cannot be dry-run".to_owned()),
        CLAIM_LABEL_PLACEHOLDER => Err("required MVP claim cannot be placeholder".to_owned()),
        _ => Ok(()),
    }
}

fn validate_value_movement_label(claim: &ClaimView<'_>) -> Result<(), String> {
    if !mentions_value_movement(claim.raw) || claim.label == CLAIM_LABEL_DEVNET_BACKED {
        return Ok(());
    }
    Err("value movement claim must be devnet-backed".to_owned())
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

fn validate_three_agent_escrow_verification(claims: &[ClaimView<'_>]) -> Result<(), String> {
    if !has_devnet_settlement_success(claims) {
        return Ok(());
    }
    let claim = three_agent_claim(claims)?;
    validate_three_agent_status(claim)?;
    validate_three_agent_privacy(claim)?;
    validate_three_agent_commitments(claim)
}

fn has_devnet_settlement_success(claims: &[ClaimView<'_>]) -> bool {
    claims
        .iter()
        .any(|claim| claim.id == "devnet_settlement_asset_movement" && claim.status == "PASS")
}

fn three_agent_claim<'a>(claims: &'a [ClaimView<'a>]) -> Result<&'a ClaimView<'a>, String> {
    claims
        .iter()
        .find(|claim| claim.id == "three_agent_escrow_verification")
        .ok_or_else(|| "missing three-agent escrow verification claim".to_owned())
}

fn validate_three_agent_status(claim: &ClaimView<'_>) -> Result<(), String> {
    if claim.label != CLAIM_LABEL_DEVNET_BACKED || claim.status != "PASS" || !claim.required {
        return Err("three-agent escrow verification claim must be devnet-backed PASS".to_owned());
    }
    Ok(())
}

fn validate_three_agent_privacy(claim: &ClaimView<'_>) -> Result<(), String> {
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

fn validate_three_agent_commitments(claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_string(claim, "terms_digest", &[
        "agent_a_terms_digest",
        "agent_b_terms_digest",
        "verifier_terms_digest",
    ])?;
    require_matching_string(claim, "escrow_id", &[
        "agent_a_escrow_id",
        "agent_b_escrow_id",
        "verifier_escrow_id",
    ])?;
    require_matching_string(claim, "settlement_tx_signature", &[
        "agent_a_settlement_tx_signature",
        "agent_b_settlement_tx_signature",
        "verifier_settlement_tx_signature",
        "persisted_settlement_tx_signature",
    ])?;
    require_matching_string(claim, "settlement_commitment", &[
        "agent_a_settlement_commitment",
        "agent_b_settlement_commitment",
        "verifier_settlement_commitment",
    ])?;
    require_matching_u64(claim, "amount_lamports", &[
        "agent_a_amount_lamports",
        "agent_b_amount_lamports",
        "verifier_amount_lamports",
        "lamports",
    ])
}

fn require_matching_string(
    claim: &ClaimView<'_>,
    canonical: &str,
    peers: &[&str],
) -> Result<(), String> {
    let expected = extract_string(claim.raw, canonical)?;
    for peer in peers {
        if extract_string(claim.raw, peer)? != expected {
            return Err(format!("three-agent shared commitment mismatch: {peer}"));
        }
    }
    Ok(())
}

fn require_matching_u64(
    claim: &ClaimView<'_>,
    canonical: &str,
    peers: &[&str],
) -> Result<(), String> {
    let expected = extract_u64(claim.raw, canonical)?;
    for peer in peers {
        if extract_u64(claim.raw, peer)? != expected {
            return Err(format!("three-agent shared commitment mismatch: {peer}"));
        }
    }
    Ok(())
}

fn allowed_labels() -> [&'static str; 6] {
    [
        CLAIM_LABEL_REAL,
        CLAIM_LABEL_DEVNET_BACKED,
        CLAIM_LABEL_LOCAL_ONLY,
        CLAIM_LABEL_DRY_RUN,
        CLAIM_LABEL_PLACEHOLDER,
        CLAIM_LABEL_ROADMAP,
    ]
}

fn mentions_value_movement(raw: &str) -> bool {
    let lowercase = raw.to_ascii_lowercase();
    VALUE_TERMS.iter().any(|term| lowercase.contains(term))
}

fn devnet_success_markers() -> [&'static str; 13] {
    [
        "\"network\":\"solana:devnet\"",
        "\"rpc_url\":\"",
        "\"payer_pubkey\":\"",
        "\"recipient_pubkey\":\"",
        "\"lamports\":",
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
