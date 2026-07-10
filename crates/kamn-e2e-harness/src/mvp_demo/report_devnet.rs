use super::devnet_settlement::{
    devnet_no_go_reason, devnet_settlement_claim_json, DevnetSettlementEvidence,
};
use super::report::{escape_json, DemoReportInput, CLAIM_LABEL_DEVNET_BACKED};
use super::three_agent_claim::three_agent_escrow_claim_json;
use super::three_agent_receipts::{
    agent_a_observation_receipt_path, agent_b_observation_receipt_path,
    agent_c_verifier_observation_receipt_path,
};
use super::three_agent_transcript::three_agent_transcript_path;
use super::three_agent_views::{agent_a_view_path, agent_b_view_path, agent_c_verifier_view_path};

pub(super) fn devnet_required_claims(input: &DemoReportInput<'_>) -> Result<Vec<String>, String> {
    match input.devnet_settlement {
        Some(evidence) => devnet_success_claims(input, evidence),
        None => Ok(vec![devnet_no_go_claim_with_reason(input)]),
    }
}

fn devnet_success_claims(
    input: &DemoReportInput<'_>,
    evidence: &DevnetSettlementEvidence,
) -> Result<Vec<String>, String> {
    let settlement_claim = devnet_settlement_claim_json(evidence);
    let Some(binding) = input.live_task_binding else {
        return Ok(vec![settlement_claim]);
    };
    let digests = input
        .three_agent_artifact_digests
        .ok_or_else(|| "missing three-agent artifact digests".to_owned())?;
    Ok(vec![
        settlement_claim,
        bound_three_agent_claim(input, evidence, binding, digests),
    ])
}

fn bound_three_agent_claim(
    input: &DemoReportInput<'_>,
    evidence: &DevnetSettlementEvidence,
    binding: &super::live_task_binding::LiveTaskBinding,
    digests: &super::artifact_digest::ThreeAgentArtifactDigests,
) -> String {
    let views = view_paths(input);
    let receipts = receipt_paths(input);
    three_agent_escrow_claim_json(
        input.run_id,
        evidence,
        binding,
        three_agent_transcript_path(input).as_str(),
        [views[0].as_str(), views[1].as_str(), views[2].as_str()],
        [
            receipts[0].as_str(),
            receipts[1].as_str(),
            receipts[2].as_str(),
        ],
        digests,
    )
}

fn view_paths(input: &DemoReportInput<'_>) -> [String; 3] {
    [
        agent_a_view_path(input),
        agent_b_view_path(input),
        agent_c_verifier_view_path(input),
    ]
}

fn receipt_paths(input: &DemoReportInput<'_>) -> [String; 3] {
    [
        agent_a_observation_receipt_path(input),
        agent_b_observation_receipt_path(input),
        agent_c_verifier_observation_receipt_path(input),
    ]
}

pub(super) fn no_go_json(input: &DemoReportInput<'_>) -> String {
    if input.devnet_mode != "required" || input.devnet_settlement.is_some() {
        return "{\"active\":false,\"reason\":\"\"}".to_owned();
    }
    format!(
        "{{\"active\":true,\"reason\":\"{}\"}}",
        effective_no_go_reason(input).as_str()
    )
}

fn devnet_no_go_claim_with_reason(input: &DemoReportInput<'_>) -> String {
    format!(
        "{{\"id\":\"devnet_settlement_no_go\",\"label\":\"{}\",\"required\":true,\"status\":\"NO-GO\",\"summary\":\"Solana devnet escrow settlement evidence unavailable\",\"network\":\"solana:devnet\",\"rpc_url\":\"{}\",\"no_go_reason\":\"{}\"}}",
        CLAIM_LABEL_DEVNET_BACKED,
        escape_json(input.solana_rpc_url.unwrap_or("")),
        effective_no_go_reason(input).as_str()
    )
}

fn effective_no_go_reason(input: &DemoReportInput<'_>) -> String {
    input
        .devnet_no_go_reason
        .unwrap_or_else(|| devnet_no_go_reason(input.solana_rpc_url))
        .to_owned()
}
