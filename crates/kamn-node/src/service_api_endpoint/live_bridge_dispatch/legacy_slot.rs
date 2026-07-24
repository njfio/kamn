use super::*;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveBridgeForwardEvidence {
    pub(crate) target_message_id: String,
    pub(crate) forward_tx_hash: String,
}

pub(crate) fn collect_live_bridge_forward_evidence(
    config: &LiveSolanaBridgeDispatchConfig,
    bridge_id: &str,
) -> Result<LiveBridgeForwardEvidence, String> {
    let finalized_slot = collect_live_solana_finalized_slot(config, bridge_id)?;
    Ok(build_forward_evidence(
        bridge_id,
        finalized_slot,
        config.rpc_url.as_str(),
    ))
}

pub(crate) fn collect_live_solana_finalized_slot(
    config: &LiveSolanaBridgeDispatchConfig,
    proof_subject: &str,
) -> Result<u64, String> {
    let report_path = report_path(proof_subject);
    let result = collect_from_report(config, &report_path);
    let _ = fs::remove_file(report_path);
    result
}

fn build_forward_evidence(
    bridge_id: &str,
    finalized_slot: u64,
    rpc_url: &str,
) -> LiveBridgeForwardEvidence {
    let bridge_tag = deterministic_body_tag(format!("{bridge_id}:{finalized_slot}").as_bytes());
    let rpc_tag = deterministic_body_tag(rpc_url.as_bytes());
    LiveBridgeForwardEvidence {
        target_message_id: format!("msg-solana-devnet-slot-{finalized_slot}-{bridge_tag:016x}"),
        forward_tx_hash: format!("solana-devnet-proof-{rpc_tag:016x}-{finalized_slot:016x}"),
    }
}

fn collect_from_report(
    config: &LiveSolanaBridgeDispatchConfig,
    report_path: &Path,
) -> Result<u64, String> {
    run_proof(config.rpc_url.as_str(), report_path)?;
    finalized_slot(&load_report(report_path)?)
}

fn run_proof(rpc_url: &str, report_path: &Path) -> Result<(), String> {
    let output = Command::new("python3")
        .arg(proof_script_path())
        .arg("--rpc-url")
        .arg(rpc_url)
        .arg("--output-json")
        .arg(report_path)
        .output()
        .map_err(|error| format!("live solana bridge proof runner spawn failed: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "live solana bridge proof runner failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn load_report(path: &Path) -> Result<serde_json::Value, String> {
    let payload = fs::read_to_string(path)
        .map_err(|error| format!("live solana bridge report read failed: {error}"))?;
    serde_json::from_str(payload.as_str())
        .map_err(|error| format!("live solana bridge report parse failed: {error}"))
}

fn finalized_slot(report: &serde_json::Value) -> Result<u64, String> {
    enforce_schema(report)?;
    enforce_health(report)?;
    report
        .get("commitment_slots")
        .and_then(|value| value.get("finalized"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "live solana bridge report missing finalized slot".to_owned())
}

fn enforce_schema(report: &serde_json::Value) -> Result<(), String> {
    let version = report
        .get("schema_version")
        .and_then(serde_json::Value::as_str);
    if version == Some(LIVE_SOLANA_PROOF_SCHEMA_VERSION) {
        Ok(())
    } else {
        Err("live solana bridge report schema version mismatch".to_owned())
    }
}

fn enforce_health(report: &serde_json::Value) -> Result<(), String> {
    let status = report
        .get("health_status")
        .and_then(serde_json::Value::as_str);
    if status == Some("ok") {
        Ok(())
    } else {
        Err("live solana bridge report health is not ok".to_owned())
    }
}

fn report_path(subject: &str) -> PathBuf {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let entropy = deterministic_body_tag(format!("{subject}:{elapsed}").as_bytes());
    std::env::temp_dir().join(format!("kamn-live-bridge-proof-{entropy:016x}.json"))
}

pub(super) fn proof_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/runtime/run_live_solana_devnet_proof.py")
}
