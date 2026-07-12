use std::path::Path;
use std::process::Command;

pub(crate) use super::devnet_settlement_json::devnet_settlement_claim_json;
use super::devnet_settlement_json::parse_devnet_settlement_evidence;
use super::devnet_settlement_live::collect_live_devnet_settlement;
use super::live_task_binding::LiveTaskBinding;
use super::settlement_evidence_artifact::write_settlement_evidence_artifact;

const NO_GO_RPC_MISSING: &str = "devnet_rpc_url_missing";
const NO_GO_KEYPAIR_MISSING: &str = "devnet_keypair_not_configured";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DevnetSettlementAttempt {
    pub(crate) evidence: Option<DevnetSettlementEvidence>,
    pub(crate) no_go_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevnetSettlementEvidence {
    pub(crate) network: String,
    pub(crate) execution_surface: String,
    pub(crate) rpc_url: String,
    pub(crate) payer_pubkey: String,
    pub(crate) recipient_pubkey: String,
    pub(crate) lamports: u64,
    pub(crate) escrow_id: String,
    pub(crate) task_id: Option<String>,
    pub(crate) task_binding_digest: Option<String>,
    pub(crate) settlement_tx_signature: String,
    pub(crate) settlement_commitment: String,
    pub(crate) payer_balance_before: u64,
    pub(crate) payer_balance_after: u64,
    pub(crate) recipient_balance_before: u64,
    pub(crate) recipient_balance_after: u64,
    pub(crate) persisted_settlement_tx_signature: String,
    pub(crate) authoritative_rpc_artifact: Option<String>,
}

pub(crate) struct DevnetSettlementInput<'a> {
    pub(crate) command: Option<&'a [String]>,
    pub(crate) solana_rpc_url: Option<&'a str>,
    pub(crate) run_dir: &'a Path,
    pub(crate) live_task_binding: Option<&'a LiveTaskBinding>,
}

pub(crate) fn collect_devnet_settlement_evidence(
    input: &DevnetSettlementInput<'_>,
) -> Result<DevnetSettlementAttempt, String> {
    if let Some(command) = input.command {
        return collect_override_devnet_settlement(command, input.run_dir, input.live_task_binding);
    }
    match input.solana_rpc_url {
        Some(rpc_url) => collect_live_attempt(rpc_url, input.run_dir, input.live_task_binding),
        None => {
            write_live_no_go_log(input.run_dir, NO_GO_RPC_MISSING)?;
            Ok(no_go(NO_GO_RPC_MISSING))
        }
    }
}

pub(crate) fn devnet_no_go_reason(solana_rpc_url: Option<&str>) -> &'static str {
    match solana_rpc_url {
        Some(value) if !value.trim().is_empty() => NO_GO_KEYPAIR_MISSING,
        _ => NO_GO_RPC_MISSING,
    }
}

fn collect_override_devnet_settlement(
    command: &[String],
    run_dir: &Path,
    binding: Option<&LiveTaskBinding>,
) -> Result<DevnetSettlementAttempt, String> {
    let output = build_command(command)?
        .output()
        .map_err(|error| format!("failed to run devnet settlement MVP proof command: {error}"))?;
    if !output.status.success() {
        write_override_log(run_dir, &output)?;
        return Err("devnet settlement MVP proof command failed".to_owned());
    }
    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    let mut evidence = parse_devnet_settlement_evidence(stdout.as_ref())?;
    apply_binding(&mut evidence, binding);
    write_live_success_log(run_dir, &evidence)?;
    Ok(pass(evidence))
}

fn collect_live_attempt(
    rpc_url: &str,
    run_dir: &Path,
    binding: Option<&LiveTaskBinding>,
) -> Result<DevnetSettlementAttempt, String> {
    match collect_live_devnet_settlement(rpc_url, run_dir, binding) {
        Ok(evidence) => {
            write_live_success_log(run_dir, &evidence)?;
            Ok(pass(evidence))
        }
        Err(error) => {
            write_live_no_go_log(run_dir, error.as_str())?;
            Ok(no_go(classify_no_go_reason(error.as_str())))
        }
    }
}

fn apply_binding(evidence: &mut DevnetSettlementEvidence, binding: Option<&LiveTaskBinding>) {
    evidence.task_id = binding.map(|value| value.task_id.clone());
    evidence.task_binding_digest = binding.map(|value| value.digest.clone());
}

fn pass(evidence: DevnetSettlementEvidence) -> DevnetSettlementAttempt {
    DevnetSettlementAttempt {
        evidence: Some(evidence),
        no_go_reason: None,
    }
}

fn no_go(reason: &str) -> DevnetSettlementAttempt {
    DevnetSettlementAttempt {
        evidence: None,
        no_go_reason: Some(reason.to_owned()),
    }
}

fn build_command(parts: &[String]) -> Result<Command, String> {
    if parts.is_empty() {
        return Err("devnet settlement MVP proof command override is empty".to_owned());
    }
    let mut command = Command::new(parts[0].as_str());
    command.args(&parts[1..]);
    Ok(command)
}

fn write_override_log(run_dir: &Path, output: &std::process::Output) -> Result<(), String> {
    let mut content = String::from("--- stdout ---\n");
    content.push_str(&String::from_utf8_lossy(output.stdout.as_slice()));
    content.push_str("\n--- stderr ---\n");
    content.push_str(&String::from_utf8_lossy(output.stderr.as_slice()));
    write_proof_file(run_dir, "devnet-settlement-output.txt", content.as_str())
}

fn write_live_no_go_log(run_dir: &Path, error: &str) -> Result<(), String> {
    let content = format!("devnet_settlement_no_go_reason={error}\n");
    write_proof_file(run_dir, "devnet-settlement-output.txt", content.as_str())
}

fn write_live_success_log(
    run_dir: &Path,
    evidence: &DevnetSettlementEvidence,
) -> Result<(), String> {
    write_settlement_evidence_artifact(run_dir, evidence)?;
    let content = format!(
        "devnet_settlement_status=PASS\nnetwork={}\nexecution_surface={}\nrpc_url={}\npayer_pubkey={}\nrecipient_pubkey={}\nlamports={}\nescrow_id={}\nsettlement_tx_signature={}\nsettlement_commitment={}\npayer_balance_before={}\npayer_balance_after={}\nrecipient_balance_before={}\nrecipient_balance_after={}\npersisted_settlement_tx_signature={}\ntask_id={}\ntask_binding_digest={}\n",
        evidence.network,
        evidence.execution_surface,
        evidence.rpc_url,
        evidence.payer_pubkey,
        evidence.recipient_pubkey,
        evidence.lamports,
        evidence.escrow_id,
        evidence.settlement_tx_signature,
        evidence.settlement_commitment,
        evidence.payer_balance_before,
        evidence.payer_balance_after,
        evidence.recipient_balance_before,
        evidence.recipient_balance_after,
        evidence.persisted_settlement_tx_signature,
        evidence.task_id.as_deref().unwrap_or("not-bound"),
        evidence.task_binding_digest.as_deref().unwrap_or("not-bound"),
    );
    write_proof_file(run_dir, "devnet-settlement-output.txt", content.as_str())
}

fn write_proof_file(run_dir: &Path, name: &str, content: &str) -> Result<(), String> {
    std::fs::write(run_dir.join("proof").join(name), content).map_err(|error| {
        format!(
            "failed to write devnet settlement proof log {}/proof/{name}: {error}",
            run_dir.display()
        )
    })
}

fn classify_no_go_reason(error: &str) -> &str {
    if error.contains("KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE") {
        return NO_GO_KEYPAIR_MISSING;
    }
    if error.contains("KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY") {
        return "devnet_recipient_not_configured";
    }
    if error.contains("KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS") {
        return "devnet_lamports_not_configured";
    }
    if error.contains("balance") {
        return "devnet_balance_evidence_unavailable";
    }
    "devnet_settlement_evidence_unavailable"
}

#[cfg(test)]
#[path = "devnet_settlement_tests.rs"]
mod tests;
