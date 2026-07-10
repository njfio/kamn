use std::path::Path;

use super::artifact_digest::{validate_json_digest, ThreeAgentViewDigests};
use super::devnet_settlement::DevnetSettlementEvidence;
use super::live_task_binding::LiveTaskBinding;
use super::report::DemoReportInput;
use super::three_agent_receipts::validate_three_agent_receipt_files;
use super::three_agent_transcript_build::transcript_json;
use super::three_agent_view_artifacts::validate_three_agent_view_files;
use super::verify_support::{
    extract_bool, extract_string, extract_u64, parse_claims, require_marker,
    validate_json_delimiters, ClaimView,
};

const CLAIM_ID: &str = "three_agent_escrow_verification";
const TRANSCRIPT_FILE: &str = "three-agent-transcript.json";

pub(crate) fn three_agent_transcript_path(input: &DemoReportInput<'_>) -> String {
    input
        .output_root
        .join(format!("{}/proof/{TRANSCRIPT_FILE}", input.run_id))
        .display()
        .to_string()
}

pub(crate) fn write_three_agent_transcript(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    binding: &LiveTaskBinding,
    run_dir: &Path,
    view_digests: &ThreeAgentViewDigests,
) -> Result<String, String> {
    let path = run_dir.join("proof").join(TRANSCRIPT_FILE);
    let artifact = transcript_json(run_id, evidence, binding, run_dir, view_digests)?;
    std::fs::write(path.as_path(), artifact.json.as_str()).map_err(|error| {
        format!(
            "failed to write three-agent transcript artifact {}: {error}",
            path.display()
        )
    })?;
    Ok(artifact.digest)
}

pub(crate) fn validate_three_agent_transcript_file(report_json: &str) -> Result<(), String> {
    let claims = parse_claims(report_json)?;
    let Some(claim) = three_agent_claim(claims.as_slice()) else {
        return Ok(());
    };
    let artifact = extract_string(claim.raw, "three_agent_transcript_artifact")?;
    validate_artifact_entry(report_json, artifact.as_str())?;
    let raw = std::fs::read_to_string(artifact.as_str()).map_err(|error| {
        format!("failed to read three-agent transcript artifact {artifact}: {error}")
    })?;
    validate_transcript(raw.as_str(), claim)?;
    validate_three_agent_view_files(report_json, raw.as_str(), claim)?;
    validate_three_agent_receipt_files(report_json, claim)
}

pub(crate) fn validate_three_agent_transcript_claim(claim: &ClaimView<'_>) -> Result<(), String> {
    extract_string(claim.raw, "three_agent_transcript_artifact")?;
    extract_string(claim.raw, "three_agent_transcript_digest")?;
    super::three_agent_view_artifacts::validate_three_agent_view_claim(claim)?;
    Ok(())
}

fn three_agent_claim<'a>(claims: &'a [ClaimView<'a>]) -> Option<&'a ClaimView<'a>> {
    claims.iter().find(|claim| claim.id == CLAIM_ID)
}

fn validate_artifact_entry(report_json: &str, artifact: &str) -> Result<(), String> {
    let entry = extract_string(report_json, "three_agent_transcript")
        .map_err(|_| "missing three_agent_transcript artifact entry".to_owned())?;
    if entry == artifact {
        return Ok(());
    }
    Err("three_agent_transcript artifact entry mismatch".to_owned())
}

fn validate_transcript(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    validate_json_delimiters(raw)?;
    reject_raw_private_payload(raw)?;
    validate_required_markers(raw)?;
    validate_transcript_fields(raw, claim)?;
    validate_digest(raw, claim)
}

fn reject_raw_private_payload(raw: &str) -> Result<(), String> {
    if raw.contains("raw_private_payload") {
        return Err("three-agent transcript contains raw private payload".to_owned());
    }
    Ok(())
}

fn validate_required_markers(raw: &str) -> Result<(), String> {
    for marker in required_markers() {
        require_marker(raw, marker, "three-agent transcript artifact")?;
    }
    Ok(())
}

fn required_markers() -> [&'static str; 13] {
    [
        "\"schema_version\":\"kamn.mvp.three-agent-transcript.v1\"",
        "\"proof_label\":\"local-only\"",
        "\"devnet_settlement_linked\":true",
        "\"agent_a_registered\"",
        "\"agent_b_registered\"",
        "\"agent_a_invoked_transaction\"",
        "\"agent_b_accepted_task\"",
        "\"escrow_funded\"",
        "\"escrow_released\"",
        "\"agent_c_verifier_verified\"",
        "\"agent_a\":\"participant-private\"",
        "\"agent_b\":\"participant-private\"",
        "\"agent_c_verifier\":\"restricted-public\"",
    ]
}

fn validate_transcript_fields(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    require_matching_string(raw, claim, "transaction_id")?;
    require_matching_string(raw, claim, "escrow_id")?;
    if claim.raw.contains("\"task_binding_digest\":") {
        require_matching_string(raw, claim, "task_binding_digest")?;
    }
    require_matching_string(raw, claim, "settlement_tx_signature")?;
    require_matching_u64(raw, claim, "amount_lamports")?;
    require_matching_string(raw, claim, "payer_pubkey")?;
    require_matching_string(raw, claim, "recipient_pubkey")?;
    require_matching_string(raw, claim, "settlement_commitment")?;
    require_matching_u64(raw, claim, "agent_a_private_field_count")?;
    require_matching_u64(raw, claim, "agent_b_private_field_count")?;
    require_matching_u64(raw, claim, "verifier_private_field_count")?;
    require_matching_bool(raw, claim, "private_payload_redacted")?;
    require_matching_string(raw, claim, "agent_a_view_digest")?;
    require_matching_string(raw, claim, "agent_b_view_digest")?;
    require_matching_string(raw, claim, "agent_c_verifier_view_digest")
}

fn validate_digest(raw: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    let artifact_digest = extract_string(raw, "transcript_digest")?;
    let claim_digest = extract_string(claim.raw, "three_agent_transcript_digest")?;
    if artifact_digest != claim_digest {
        return Err("three-agent transcript digest mismatch".to_owned());
    }
    validate_json_digest(
        raw,
        "transcript_digest",
        claim_digest.as_str(),
        "three-agent transcript",
    )
}

fn require_matching_string(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_string(raw, field)? == extract_string(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three-agent transcript {field} mismatch"))
}

fn require_matching_u64(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_u64(raw, field)? == extract_u64(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three-agent transcript {field} mismatch"))
}

fn require_matching_bool(raw: &str, claim: &ClaimView<'_>, field: &str) -> Result<(), String> {
    if extract_bool(raw, field)? == extract_bool(claim.raw, field)? {
        return Ok(());
    }
    Err(format!("three-agent transcript {field} mismatch"))
}
