use super::report::CLAIM_LABEL_DEVNET_BACKED;
use super::verify_support::{extract_bool, extract_string, extract_u64, ClaimView};

pub(crate) fn validate_three_agent_escrow_verification(
    claims: &[ClaimView<'_>],
) -> Result<(), String> {
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
    validate_terms_digest(claim)?;
    validate_escrow_id(claim)?;
    validate_settlement_signature(claim)?;
    validate_settlement_commitment(claim)?;
    validate_amount(claim)
}

fn validate_terms_digest(claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_string(
        claim,
        "terms_digest",
        &[
            "agent_a_terms_digest",
            "agent_b_terms_digest",
            "verifier_terms_digest",
        ],
    )
}

fn validate_escrow_id(claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_string(
        claim,
        "escrow_id",
        &[
            "agent_a_escrow_id",
            "agent_b_escrow_id",
            "verifier_escrow_id",
        ],
    )
}

fn validate_settlement_signature(claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_string(
        claim,
        "settlement_tx_signature",
        &[
            "agent_a_settlement_tx_signature",
            "agent_b_settlement_tx_signature",
            "verifier_settlement_tx_signature",
            "persisted_settlement_tx_signature",
        ],
    )
}

fn validate_settlement_commitment(claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_string(
        claim,
        "settlement_commitment",
        &[
            "agent_a_settlement_commitment",
            "agent_b_settlement_commitment",
            "verifier_settlement_commitment",
        ],
    )
}

fn validate_amount(claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_u64(
        claim,
        "amount_lamports",
        &[
            "agent_a_amount_lamports",
            "agent_b_amount_lamports",
            "verifier_amount_lamports",
            "lamports",
        ],
    )
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
