use super::artifact_digest::validate_json_digest;
use super::command_config::LiveTaskEvidencePaths;
use super::live_task_funding_verify::validate_funding_request;
use super::live_task_sources::{validate_live_task_evidence, ValidatedSources};
use super::verify_support::{extract_string, extract_u64, parse_claims, ClaimView};

pub(crate) fn validate_live_task_binding_file(report: &str) -> Result<(), String> {
    let claims = parse_claims(report)?;
    let Some(claim) = claims
        .iter()
        .find(|claim| claim.id == "three_agent_escrow_verification")
    else {
        return Ok(());
    };
    let artifact = binding_artifact_path(report, claim)?;
    let raw = std::fs::read_to_string(artifact.as_str())
        .map_err(|error| format!("failed to read live task settlement binding: {error}"))?;
    let digest = extract_string(claim.raw, "live_task_settlement_binding_digest")?;
    validate_json_digest(
        raw.as_str(),
        "binding_digest",
        digest.as_str(),
        "live task settlement binding",
    )?;
    let sources = validate_live_task_evidence(&source_paths(raw.as_str())?)?;
    validate_binding_fields(raw.as_str(), &sources, digest.as_str())?;
    validate_claim_fields(report, claim, &sources, digest.as_str())
}

fn binding_artifact_path(report: &str, claim: &ClaimView<'_>) -> Result<String, String> {
    let claim_path = extract_string(claim.raw, "live_task_settlement_binding_artifact")?;
    let report_path = extract_string(report, "live_task_settlement_binding")?;
    if claim_path == report_path {
        Ok(claim_path)
    } else {
        Err("live task settlement binding artifact path mismatch".to_owned())
    }
}

fn source_paths(raw: &str) -> Result<LiveTaskEvidencePaths, String> {
    Ok(LiveTaskEvidencePaths {
        handoff: extract_string(raw, "handoff_artifact")?,
        agent_a_receipt: extract_string(raw, "agent_a_receipt_artifact")?,
        agent_b_receipt: extract_string(raw, "agent_b_receipt_artifact")?,
        agent_c_observation: extract_string(raw, "agent_c_observation_artifact")?,
    })
}

fn validate_binding_fields(
    raw: &str,
    sources: &ValidatedSources,
    digest: &str,
) -> Result<(), String> {
    validate_binding_identity(raw, sources, digest)?;
    validate_binding_source_digests(raw, sources)
}

fn validate_binding_identity(
    raw: &str,
    sources: &ValidatedSources,
    digest: &str,
) -> Result<(), String> {
    require_string(
        raw,
        "schema_version",
        "kamn.mvp.live-task-settlement-binding.v1",
    )?;
    require_string(raw, "binding_digest", digest)?;
    require_string(raw, "task_id", sources.task_id.as_str())?;
    require_string(raw, "state", "accepted")?;
    require_u64(raw, "agent_a_pi_process_id", sources.agent_a_pid)?;
    require_u64(raw, "agent_b_pi_process_id", sources.agent_b_pid)?;
    require_u64(raw, "agent_c_pi_process_id", sources.agent_c_pid)
}

fn validate_binding_source_digests(raw: &str, sources: &ValidatedSources) -> Result<(), String> {
    require_string(
        raw,
        "source_handoff_digest",
        sources.handoff_digest.as_str(),
    )?;
    require_string(
        raw,
        "source_agent_a_receipt_digest",
        sources.agent_a_digest.as_str(),
    )?;
    require_string(
        raw,
        "source_agent_b_receipt_digest",
        sources.agent_b_digest.as_str(),
    )?;
    require_string(
        raw,
        "source_agent_c_observation_digest",
        sources.agent_c_digest.as_str(),
    )?;
    require_string(
        raw,
        "agent_c_public_commitment",
        sources.public_commitment.as_str(),
    )
}

fn validate_claim_fields(
    report: &str,
    claim: &ClaimView<'_>,
    sources: &ValidatedSources,
    digest: &str,
) -> Result<(), String> {
    require_string(claim.raw, "transaction_id", sources.task_id.as_str())?;
    require_string(claim.raw, "terms_digest", digest)?;
    require_string(claim.raw, "task_binding_digest", digest)?;
    let devnet = devnet_claim(report)?;
    require_string(devnet.raw, "task_id", sources.task_id.as_str())?;
    require_string(devnet.raw, "task_binding_digest", digest)?;
    let escrow_id = extract_string(devnet.raw, "escrow_id")?;
    require_string(claim.raw, "escrow_id", escrow_id.as_str())?;
    validate_funding_request(
        report,
        &devnet,
        sources.task_id.as_str(),
        digest,
        escrow_id.as_str(),
    )
}

fn devnet_claim(report: &str) -> Result<ClaimView<'_>, String> {
    parse_claims(report)?
        .into_iter()
        .find(|item| item.id == "devnet_settlement_asset_movement")
        .ok_or_else(|| "missing devnet settlement claim for live task binding".to_owned())
}

fn require_string(raw: &str, field: &str, expected: &str) -> Result<(), String> {
    if extract_string(raw, field)? == expected {
        Ok(())
    } else {
        Err(format!("live task settlement binding {field} mismatch"))
    }
}

fn require_u64(raw: &str, field: &str, expected: u64) -> Result<(), String> {
    if extract_u64(raw, field)? == expected {
        Ok(())
    } else {
        Err(format!("live task settlement binding {field} mismatch"))
    }
}
