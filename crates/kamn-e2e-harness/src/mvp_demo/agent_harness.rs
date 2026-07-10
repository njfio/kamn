use super::report::CLAIM_LABEL_LOCAL_ONLY;
use super::{
    agent_harness_three_agent::validate_agent_three_agent_boundary,
    verify_support::{extract_optional_string, parse_claims, require_marker, ClaimView},
};

const AGENT_HARNESS_CLAIM_ID: &str = "mcp_agent_harness_verification";
const AGENT_HARNESS_ARTIFACT_FIELD: &str = "agent_harness_evidence";
const MCP_TOOL_SURFACE: &str = "mcp-tools";
const PI_EXTENSION_TOOL_SURFACE: &str = "pi-extension-tools";

pub(crate) fn agent_harness_claim_json() -> String {
    format!(
        "{{\"id\":\"{AGENT_HARNESS_CLAIM_ID}\",\"label\":\"{CLAIM_LABEL_LOCAL_ONLY}\",\"required\":false,\"status\":\"PASS\",\"summary\":\"MCP agent harness verified report boundaries\",\"harness\":\"mcp-agent\"}}"
    )
}

pub(crate) fn validate_agent_harness_claim_shape(claims: &[ClaimView<'_>]) -> Result<(), String> {
    if let Some(claim) = agent_harness_claim(claims) {
        validate_agent_harness_claim(claim)?;
    }
    Ok(())
}

pub(crate) fn validate_agent_harness_evidence_file(
    report_json: &str,
    report_path: &str,
) -> Result<(), String> {
    if !report_json.contains(AGENT_HARNESS_CLAIM_ID) {
        return Ok(());
    }
    let artifact_path = extract_optional_string(report_json, AGENT_HARNESS_ARTIFACT_FIELD)
        .ok_or_else(|| "missing agent harness evidence artifact path".to_owned())?;
    validate_agent_harness_evidence_path(report_json, report_path, artifact_path.as_str())
}

pub(crate) fn validate_agent_harness_evidence_path(
    report_json: &str,
    report_path: &str,
    artifact_path: &str,
) -> Result<(), String> {
    let artifact = std::fs::read_to_string(artifact_path).map_err(|error| {
        format!(
            "failed to read agent harness evidence {}: {error}",
            artifact_path
        )
    })?;
    validate_agent_harness_evidence(artifact.as_str(), report_path, report_json)
}

pub(crate) fn agent_harness_execution_surface(path: &str) -> Result<String, String> {
    let artifact = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read agent harness evidence {path}: {error}"))?;
    extract_optional_string(artifact.as_str(), "execution_surface")
        .ok_or_else(|| "missing agent harness execution_surface".to_owned())
}

fn agent_harness_claim<'a>(claims: &'a [ClaimView<'a>]) -> Option<&'a ClaimView<'a>> {
    claims
        .iter()
        .find(|claim| claim.id == AGENT_HARNESS_CLAIM_ID)
}

fn validate_agent_harness_claim(claim: &ClaimView<'_>) -> Result<(), String> {
    if claim.label != CLAIM_LABEL_LOCAL_ONLY {
        return Err("agent harness claim must be local-only".to_owned());
    }
    if claim.status != "PASS" {
        return Err("agent harness claim must be PASS when present".to_owned());
    }
    require_marker(claim.raw, "\"harness\":\"mcp-agent\"", "agent harness kind")
}

fn validate_agent_harness_evidence(
    artifact: &str,
    report_path: &str,
    report_json: &str,
) -> Result<(), String> {
    validate_evidence_markers(artifact)?;
    validate_execution_surface(artifact)?;
    validate_report_path(artifact, report_path)?;
    validate_private_boundary(artifact)?;
    validate_settlement_boundary(artifact)?;
    validate_agent_three_agent_boundary(artifact, parse_claims(report_json)?.as_slice())
}

fn validate_evidence_markers(artifact: &str) -> Result<(), String> {
    for (marker, context) in required_evidence_markers() {
        require_marker(artifact, marker, context)?;
    }
    Ok(())
}

fn required_evidence_markers() -> [(&'static str, &'static str); 11] {
    [
        (
            "\"schema_version\":\"kamn.mvp.agent-harness-evidence.v1\"",
            "agent harness schema",
        ),
        ("\"harness\":\"mcp-agent\"", "agent harness kind"),
        (
            "\"verifier_status\":\"PASS\"",
            "agent harness verifier status",
        ),
        ("\"agent_a\"", "agent A role"),
        ("\"agent_b\"", "agent B role"),
        ("\"agent_c_verifier\"", "agent C verifier role"),
        ("\"register\"", "register tool marker"),
        ("\"create_task\"", "create_task tool marker"),
        ("\"fund_escrow\"", "fund_escrow tool marker"),
        ("\"release_escrow\"", "release_escrow tool marker"),
        ("\"verify_proof\"", "verify_proof tool marker"),
    ]
}

fn validate_execution_surface(artifact: &str) -> Result<(), String> {
    let surface = extract_optional_string(artifact, "execution_surface")
        .ok_or_else(|| "missing agent harness execution_surface".to_owned())?;
    if surface == MCP_TOOL_SURFACE || surface == PI_EXTENSION_TOOL_SURFACE {
        return Ok(());
    }
    Err(format!(
        "unsupported agent harness execution_surface: {surface}"
    ))
}

fn validate_report_path(artifact: &str, report_path: &str) -> Result<(), String> {
    let artifact_report_path = extract_optional_string(artifact, "report_path")
        .ok_or_else(|| "missing agent harness report_path".to_owned())?;
    if artifact_report_path == report_path {
        return Ok(());
    }
    Err("agent harness report_path does not match verified report".to_owned())
}

fn validate_private_boundary(artifact: &str) -> Result<(), String> {
    if artifact.contains("\"verifier_private_view_visible\":false") {
        return Ok(());
    }
    Err("agent harness verifier_private_view_visible must be false".to_owned())
}

fn validate_settlement_boundary(artifact: &str) -> Result<(), String> {
    if artifact.contains("\"settlement_claim_label\":\"devnet-backed\"") {
        return Ok(());
    }
    Err("agent harness settlement_claim_label must be devnet-backed".to_owned())
}
