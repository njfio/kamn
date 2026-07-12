use std::path::Path;

use super::artifact_digest::validate_json_digest;
use super::runtime_receipt_chain::build_runtime_receipt_chain_from_actor_paths;
use super::verify_support::{extract_string, extract_u64, ClaimView};

pub(super) fn write_runtime_chain(
    path: &Path,
    actor_paths: &[String; 3],
) -> Result<String, String> {
    let raw = build_runtime_receipt_chain_from_actor_paths(actor_paths)?;
    let digest = extract_string(raw.as_str(), "chain_digest")?;
    std::fs::write(path, raw).map_err(|error| {
        format!(
            "failed to write runtime receipt chain artifact {}: {error}",
            path.display()
        )
    })?;
    Ok(digest)
}

pub(super) fn validate_runtime_chain(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    if raw.contains("raw_private_payload") || raw.contains("private_receipt_digest") {
        return Err("RUNTIME_RECEIPT_CHAIN_VERIFIER_PRIVATE_LEAK".to_owned());
    }
    for field in [
        "transaction_id",
        "escrow_id",
        "settlement_tx_signature",
        "settlement_commitment",
    ] {
        require_matching_string(raw, claim, field)?;
    }
    require_matching_u64(raw, claim, "amount_lamports")?;
    validate_chain_digest(raw, claim)
}

fn validate_chain_digest(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    let digest = extract_string(raw, "chain_digest")?;
    let claimed = extract_string(claim.raw, "three_agent_transcript_digest")?;
    if digest != claimed {
        return Err("RUNTIME_RECEIPT_CHAIN_ARTIFACT_MISMATCH".to_owned());
    }
    validate_json_digest(
        raw,
        "chain_digest",
        digest.as_str(),
        "runtime receipt chain",
    )
}

fn require_matching_string(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_string(raw, field)? == extract_string(claim.raw, field)? {
        return Ok(());
    }
    Err("RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH".to_owned())
}

fn require_matching_u64(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_u64(raw, field)? == extract_u64(claim.raw, field)? {
        return Ok(());
    }
    Err("RUNTIME_RECEIPT_CHAIN_FACT_MISMATCH".to_owned())
}
