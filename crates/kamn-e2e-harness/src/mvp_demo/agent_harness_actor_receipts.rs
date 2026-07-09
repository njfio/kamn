use super::verify_support::{extract_string, extract_u64, require_marker, ClaimView};

pub(crate) fn validate_actor_receipts(artifact: &str, claim: &ClaimView<'_>) -> Result<(), String> {
    require_marker(
        artifact,
        "\"three_agent_actor_tool_receipts\":[",
        "three_agent_actor_tool_receipts",
    )?;
    let report_path = extract_string(artifact, "report_path")?;
    for spec in receipt_specs() {
        validate_receipt(artifact, claim, &report_path, &spec)?;
    }
    Ok(())
}

fn validate_receipt(
    artifact: &str,
    claim: &ClaimView<'_>,
    report_path: &str,
    spec: &ReceiptSpec,
) -> Result<(), String> {
    let receipt = receipt_section(artifact, spec.tool)?;
    require_receipt_u64(receipt, "sequence", spec.sequence)?;
    require_receipt_string(receipt, "tool", spec.tool)?;
    require_receipt_string(receipt, "agent", spec.agent)?;
    require_receipt_string(receipt, "action", spec.action)?;
    require_receipt_string(receipt, "outcome", "PASS")?;
    require_receipt_string(receipt, "report_path", report_path)?;
    require_receipt_string(receipt, "view_scope", spec.scope)?;
    require_receipt_claim_match(receipt, claim, "view_artifact", spec.artifact_field)?;
    require_receipt_claim_match(receipt, claim, "view_digest", spec.digest_field)?;
    reject_private_receipt_markers(receipt, spec.agent)
}

fn receipt_specs() -> [ReceiptSpec; 5] {
    [
        ReceiptSpec::new(
            1,
            "kamn_agent_a_register",
            "agent_a",
            "register",
            "participant-private",
            "agent_a_view_artifact",
            "agent_a_view_digest",
        ),
        ReceiptSpec::new(
            2,
            "kamn_agent_a_invoke_transaction",
            "agent_a",
            "invoke_transaction",
            "participant-private",
            "agent_a_view_artifact",
            "agent_a_view_digest",
        ),
        ReceiptSpec::new(
            3,
            "kamn_agent_b_register",
            "agent_b",
            "register",
            "participant-private",
            "agent_b_view_artifact",
            "agent_b_view_digest",
        ),
        ReceiptSpec::new(
            4,
            "kamn_agent_b_accept_task",
            "agent_b",
            "accept_task",
            "participant-private",
            "agent_b_view_artifact",
            "agent_b_view_digest",
        ),
        ReceiptSpec::new(
            5,
            "kamn_agent_c_verify_three_agent_proof",
            "agent_c_verifier",
            "verify_proof",
            "restricted-public",
            "agent_c_verifier_view_artifact",
            "agent_c_verifier_view_digest",
        ),
    ]
}

struct ReceiptSpec {
    sequence: u64,
    tool: &'static str,
    agent: &'static str,
    action: &'static str,
    scope: &'static str,
    artifact_field: &'static str,
    digest_field: &'static str,
}

impl ReceiptSpec {
    const fn new(
        sequence: u64,
        tool: &'static str,
        agent: &'static str,
        action: &'static str,
        scope: &'static str,
        artifact_field: &'static str,
        digest_field: &'static str,
    ) -> Self {
        Self {
            sequence,
            tool,
            agent,
            action,
            scope,
            artifact_field,
            digest_field,
        }
    }
}

fn require_receipt_string(receipt: &str, field: &str, expected: &str) -> Result<(), String> {
    let actual = extract_string(receipt, field)
        .map_err(|_| format!("missing three_agent_actor_tool_receipts field: {field}"))?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("three_agent_actor_tool_receipts {field} mismatch"))
}

fn require_receipt_u64(receipt: &str, field: &str, expected: u64) -> Result<(), String> {
    let actual = extract_u64(receipt, field)
        .map_err(|_| format!("missing three_agent_actor_tool_receipts field: {field}"))?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("three_agent_actor_tool_receipts {field} mismatch"))
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
        "three_agent_actor_tool_receipts {claim_field} mismatch"
    ))
}

fn reject_private_receipt_markers(receipt: &str, agent: &str) -> Result<(), String> {
    if receipt.contains("raw_private_payload")
        || receipt.contains("participant_private_view_digest")
    {
        return Err(format!(
            "three_agent_actor_tool_receipts {agent} private marker"
        ));
    }
    Ok(())
}

fn receipt_section<'a>(artifact: &'a str, tool: &str) -> Result<&'a str, String> {
    let marker = format!("\"tool\":\"{tool}\"");
    let marker_index = artifact
        .find(marker.as_str())
        .ok_or_else(|| format!("missing three_agent_actor_tool_receipts tool: {tool}"))?;
    let start = artifact[..marker_index]
        .rfind('{')
        .ok_or_else(|| format!("malformed three_agent_actor_tool_receipts tool: {tool}"))?;
    matching_object(artifact, start)
}

fn matching_object(raw: &str, start: usize) -> Result<&str, String> {
    let mut depth = 0_u64;
    for (offset, byte) in raw[start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' if depth == 0 => break,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(&raw[start..start + offset + 1]);
                }
            }
            _ => {}
        }
    }
    Err("malformed three_agent_actor_tool_receipts object".to_owned())
}
