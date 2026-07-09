use super::agent_harness_json::matching_object;
use super::verify_support::{extract_string, require_marker, ClaimView};

pub(crate) fn validate_observation_receipts(
    artifact: &str,
    claim: &ClaimView<'_>,
) -> Result<(), String> {
    require_marker(
        artifact,
        "\"three_agent_actor_observation_receipts\":{",
        "three_agent_actor_observation_receipts",
    )?;
    for spec in receipt_specs() {
        validate_receipt(artifact, claim, &spec)?;
    }
    Ok(())
}

fn validate_receipt(
    artifact: &str,
    claim: &ClaimView<'_>,
    spec: &ReceiptSpec,
) -> Result<(), String> {
    let receipt = receipt_section(artifact, spec.agent)?;
    require_receipt_string(receipt, "agent", spec.agent)?;
    require_receipt_string(receipt, "view_scope", spec.scope)?;
    require_receipt_claim_match(receipt, claim, "artifact", spec.artifact_field)?;
    require_receipt_claim_match(receipt, claim, "digest", spec.digest_field)?;
    reject_private_markers(receipt, spec.agent)
}

fn receipt_specs() -> [ReceiptSpec; 3] {
    [
        ReceiptSpec::new(
            "agent_a",
            "participant-private",
            "agent_a_observation_receipt_artifact",
            "agent_a_observation_receipt_digest",
        ),
        ReceiptSpec::new(
            "agent_b",
            "participant-private",
            "agent_b_observation_receipt_artifact",
            "agent_b_observation_receipt_digest",
        ),
        ReceiptSpec::new(
            "agent_c_verifier",
            "restricted-public",
            "agent_c_verifier_observation_receipt_artifact",
            "agent_c_verifier_observation_receipt_digest",
        ),
    ]
}

struct ReceiptSpec {
    agent: &'static str,
    scope: &'static str,
    artifact_field: &'static str,
    digest_field: &'static str,
}

impl ReceiptSpec {
    const fn new(
        agent: &'static str,
        scope: &'static str,
        artifact_field: &'static str,
        digest_field: &'static str,
    ) -> Self {
        Self {
            agent,
            scope,
            artifact_field,
            digest_field,
        }
    }
}

fn require_receipt_string(receipt: &str, field: &str, expected: &str) -> Result<(), String> {
    if extract_string(receipt, field)? == expected {
        return Ok(());
    }
    Err(format!("three_agent_actor_observation_receipts {field} mismatch"))
}

fn require_receipt_claim_match(
    receipt: &str,
    claim: &ClaimView<'_>,
    receipt_field: &str,
    claim_field: &str,
) -> Result<(), String> {
    if extract_string(receipt, receipt_field)? == extract_string(claim.raw, claim_field)? {
        return Ok(());
    }
    Err(format!(
        "three_agent_actor_observation_receipts {claim_field} mismatch"
    ))
}

fn reject_private_markers(receipt: &str, agent: &str) -> Result<(), String> {
    if receipt.contains("raw_private_payload")
        || receipt.contains("participant_private_view_digest")
    {
        return Err(format!(
            "three_agent_actor_observation_receipts {agent} private marker"
        ));
    }
    Ok(())
}

fn receipt_section<'a>(artifact: &'a str, agent: &str) -> Result<&'a str, String> {
    let outer = "\"three_agent_actor_observation_receipts\":{";
    let outer_index = artifact
        .find(outer)
        .ok_or_else(|| "missing three_agent_actor_observation_receipts".to_owned())?;
    let observation_section = &artifact[outer_index + outer.len()..];
    let marker = format!("\"{agent}\":{{");
    let marker_index = observation_section.find(marker.as_str()).ok_or_else(|| {
        format!("missing three_agent_actor_observation_receipts agent: {agent}")
    })?;
    let start = outer_index + outer.len() + marker_index + marker.len() - 1;
    matching_object(artifact, start, "three_agent_actor_observation_receipts")
}
