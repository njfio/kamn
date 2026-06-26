use super::*;
use std::path::{Path, PathBuf};
use std::process::Command;

const LIVE_SOLANA_BRIDGE_RPC_URL_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL";
const LIVE_SOLANA_PROOF_SCHEMA_VERSION: &str = "kamn.solana.devnet.live-proof-report.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LiveSolanaBridgeDispatchConfig {
    pub(super) rpc_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LiveBridgeForwardEvidence {
    pub(super) target_message_id: String,
    pub(super) forward_tx_hash: String,
}

pub(super) fn resolve_live_solana_bridge_dispatch_config(
) -> Result<Option<LiveSolanaBridgeDispatchConfig>, String> {
    resolve_live_solana_bridge_dispatch_config_from_env(std::env::var(
        LIVE_SOLANA_BRIDGE_RPC_URL_ENV,
    ))
}

fn resolve_live_solana_bridge_dispatch_config_from_env(
    env_value: Result<String, std::env::VarError>,
) -> Result<Option<LiveSolanaBridgeDispatchConfig>, String> {
    match env_value {
        Ok(value) => Ok(Some(LiveSolanaBridgeDispatchConfig {
            rpc_url: normalize_live_solana_bridge_rpc_url(value.as_str())?,
        })),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(format!(
            "live solana bridge rpc env must be utf-8: {LIVE_SOLANA_BRIDGE_RPC_URL_ENV}"
        )),
    }
}

fn normalize_live_solana_bridge_rpc_url(value: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!(
            "live solana bridge rpc env must not be empty: {LIVE_SOLANA_BRIDGE_RPC_URL_ENV}"
        ));
    }
    if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
        return Err(format!(
            "live solana bridge rpc env must start with http:// or https://: {LIVE_SOLANA_BRIDGE_RPC_URL_ENV}"
        ));
    }
    validate_live_solana_proof_script_path(live_solana_proof_script_path().as_path())?;
    Ok(normalized.to_owned())
}

fn validate_live_solana_proof_script_path(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    Err(format!(
        "live solana bridge proof runner missing: {}",
        path.display()
    ))
}

pub(super) fn collect_live_bridge_forward_evidence(
    config: &LiveSolanaBridgeDispatchConfig,
    bridge_id: &str,
) -> Result<LiveBridgeForwardEvidence, String> {
    let report_path = live_solana_proof_report_path(bridge_id);
    let result = collect_live_bridge_forward_evidence_from_report(config, bridge_id, &report_path);
    let _ = fs::remove_file(report_path);
    result
}

pub(super) fn collect_live_solana_finalized_slot(
    config: &LiveSolanaBridgeDispatchConfig,
    proof_subject: &str,
) -> Result<u64, String> {
    let report_path = live_solana_proof_report_path(proof_subject);
    let result = collect_live_solana_finalized_slot_from_report(config, &report_path);
    let _ = fs::remove_file(report_path);
    result
}

fn collect_live_bridge_forward_evidence_from_report(
    config: &LiveSolanaBridgeDispatchConfig,
    bridge_id: &str,
    report_path: &Path,
) -> Result<LiveBridgeForwardEvidence, String> {
    let finalized_slot = collect_live_solana_finalized_slot_from_report(config, report_path)?;
    Ok(build_live_bridge_forward_evidence(
        bridge_id,
        finalized_slot,
        config.rpc_url.as_str(),
    ))
}

fn collect_live_solana_finalized_slot_from_report(
    config: &LiveSolanaBridgeDispatchConfig,
    report_path: &Path,
) -> Result<u64, String> {
    run_live_solana_devnet_proof(config.rpc_url.as_str(), report_path)?;
    let report = load_live_solana_report(report_path)?;
    finalized_slot_from_report(&report)
}

fn run_live_solana_devnet_proof(rpc_url: &str, report_path: &Path) -> Result<(), String> {
    let output = Command::new("python3")
        .arg(live_solana_proof_script_path())
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

fn load_live_solana_report(path: &Path) -> Result<serde_json::Value, String> {
    let payload = fs::read_to_string(path)
        .map_err(|error| format!("live solana bridge report read failed: {error}"))?;
    serde_json::from_str(payload.as_str())
        .map_err(|error| format!("live solana bridge report parse failed: {error}"))
}

fn finalized_slot_from_report(report: &serde_json::Value) -> Result<u64, String> {
    enforce_report_schema(report)?;
    enforce_report_health(report)?;
    report
        .get("commitment_slots")
        .and_then(|value| value.get("finalized"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "live solana bridge report missing finalized slot".to_owned())
}

fn enforce_report_schema(report: &serde_json::Value) -> Result<(), String> {
    if report
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        == Some(LIVE_SOLANA_PROOF_SCHEMA_VERSION)
    {
        return Ok(());
    }
    Err("live solana bridge report schema version mismatch".to_owned())
}

fn enforce_report_health(report: &serde_json::Value) -> Result<(), String> {
    if report
        .get("health_status")
        .and_then(serde_json::Value::as_str)
        == Some("ok")
    {
        return Ok(());
    }
    Err("live solana bridge report health is not ok".to_owned())
}

fn build_live_bridge_forward_evidence(
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

fn live_solana_proof_report_path(report_subject: &str) -> PathBuf {
    let entropy = deterministic_body_tag(
        format!(
            "{report_subject}:{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
        .as_bytes(),
    );
    std::env::temp_dir().join(format!("kamn-live-bridge-proof-{entropy:016x}.json"))
}

fn live_solana_proof_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/runtime/run_live_solana_devnet_proof.py")
}
