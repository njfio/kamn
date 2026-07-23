use std::path::Path;

use super::devnet_settlement::DevnetSettlementEvidence;
use super::devnet_settlement_build::build_node_binary;
use super::devnet_settlement_node::{launch_and_drive_service_api, ServiceApiRun};
use super::devnet_settlement_solana::{
    balance_pair, capture_confirmation, solana_pubkey, validate_balance_movement,
};
use super::devnet_settlement_state::persisted_signature;
use super::live_task_binding::LiveTaskBinding;

const SETTLEMENT_KEYPAIR_FILE_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const SETTLEMENT_RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const SETTLEMENT_LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const SETTLEMENT_COMMITMENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT";

pub(super) fn collect_live_devnet_settlement(
    rpc_url: &str,
    run_dir: &Path,
    binding: Option<&LiveTaskBinding>,
) -> Result<DevnetSettlementEvidence, String> {
    let config = LiveSettlementConfig::from_env(rpc_url)?;
    build_node_binary(run_dir)?;
    let payer = solana_pubkey(config.keypair_file.as_str())?;
    let before = balance_pair(config.rpc_url.as_str(), payer.as_str(), &config)?;
    let state_file = run_dir.join("state/service-api-state.json");
    let service_api = launch_and_drive_service_api(run_dir, &state_file, &config, binding)?;
    let persisted = persisted_signature(state_file.as_path(), service_api.escrow_id.as_str())?;
    let confirmation = capture_confirmation(
        config.rpc_url.as_str(),
        config.commitment.as_str(),
        persisted.as_str(),
        run_dir,
    )?;
    let after = balance_pair(config.rpc_url.as_str(), payer.as_str(), &config)?;
    validate_balance_movement(&before, &after, &config)?;
    let result = LiveSettlementResult {
        persisted,
        service_api,
        confirmation,
    };
    Ok(evidence(config, payer, before, after, result, binding))
}

pub(super) struct LiveSettlementConfig {
    pub(super) rpc_url: String,
    pub(super) keypair_file: String,
    pub(super) recipient_pubkey: String,
    pub(super) lamports: u64,
    pub(super) commitment: String,
}

struct LiveSettlementResult {
    persisted: String,
    service_api: ServiceApiRun,
    confirmation: std::path::PathBuf,
}

impl LiveSettlementConfig {
    fn from_env(rpc_url: &str) -> Result<Self, String> {
        Ok(Self {
            rpc_url: rpc_url.to_owned(),
            keypair_file: required_env(SETTLEMENT_KEYPAIR_FILE_ENV)?,
            recipient_pubkey: required_env(SETTLEMENT_RECIPIENT_ENV)?,
            lamports: parse_lamports(required_env(SETTLEMENT_LAMPORTS_ENV)?)?,
            commitment: optional_env(SETTLEMENT_COMMITMENT_ENV, "finalized"),
        })
    }
}

fn evidence(
    config: LiveSettlementConfig,
    payer: String,
    before: super::devnet_settlement_solana::BalancePair,
    after: super::devnet_settlement_solana::BalancePair,
    result: LiveSettlementResult,
    binding: Option<&LiveTaskBinding>,
) -> DevnetSettlementEvidence {
    DevnetSettlementEvidence {
        network: "solana:devnet".to_owned(),
        execution_surface: "live-service-api".to_owned(),
        rpc_url: config.rpc_url,
        payer_pubkey: payer,
        recipient_pubkey: config.recipient_pubkey,
        lamports: config.lamports,
        escrow_id: result.service_api.escrow_id,
        task_id: Some(result.service_api.task_id),
        transaction_id: None,
        terms_digest: None,
        task_binding_digest: binding.map(|value| value.digest.clone()),
        settlement_tx_signature: result.persisted.clone(),
        settlement_commitment: config.commitment,
        payer_balance_before: before.payer,
        payer_balance_after: after.payer,
        recipient_balance_before: before.recipient,
        recipient_balance_after: after.recipient,
        fee_lamports: None,
        settlement_receipt_hash: Some(result.persisted.clone()),
        persisted_settlement_tx_signature: result.persisted,
        service_state_digest: None,
        settlement_intent_digest: None,
        receipt_chain_commitment: None,
        service_receipt_commitment: None,
        authoritative_rpc_artifact: Some(result.confirmation.display().to_string()),
    }
}

fn required_env(name: &str) -> Result<String, String> {
    std::env::var(name)
        .map(|value| value.trim().to_owned())
        .map_err(|_| format!("missing required env: {name}"))
        .and_then(|value| reject_empty_env(name, value))
}

fn reject_empty_env(name: &str, value: String) -> Result<String, String> {
    if value.is_empty() {
        return Err(format!("env must not be empty: {name}"));
    }
    Ok(value)
}

fn optional_env(name: &str, fallback: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn parse_lamports(value: String) -> Result<u64, String> {
    let lamports = value.parse::<u64>().map_err(|error| {
        format!("{SETTLEMENT_LAMPORTS_ENV} must be a positive integer: {error}")
    })?;
    if lamports == 0 {
        return Err(format!(
            "{SETTLEMENT_LAMPORTS_ENV} must be greater than zero"
        ));
    }
    Ok(lamports)
}
