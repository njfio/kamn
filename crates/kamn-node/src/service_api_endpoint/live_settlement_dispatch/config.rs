use super::super::live_bridge_dispatch::LiveSolanaBridgeDispatchConfig;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LIVE_SOLANA_SETTLEMENT_LAMPORTS_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const LIVE_SOLANA_SETTLEMENT_COMMITMENT_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveSolanaSettlementConfig {
    pub(super) rpc_url: String,
    pub(super) keypair_file: String,
    pub(super) recipient_pubkey: Pubkey,
    pub(super) lamports: u64,
    pub(super) commitment: CommitmentConfig,
    pub(super) commitment_label: String,
}

pub(crate) fn resolve_live_solana_settlement_config(
    bridge_config: Option<&LiveSolanaBridgeDispatchConfig>,
) -> Result<Option<LiveSolanaSettlementConfig>, String> {
    let keypair_file = read_required_env(LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV)?;
    let recipient_pubkey = read_required_env(LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY_ENV)?;
    let lamports = read_required_env(LIVE_SOLANA_SETTLEMENT_LAMPORTS_ENV)?;
    let commitment = read_optional_env(LIVE_SOLANA_SETTLEMENT_COMMITMENT_ENV)?;
    if keypair_file.is_none() && recipient_pubkey.is_none() && lamports.is_none() && commitment.is_none() {
        return Ok(None);
    }
    let Some(bridge_config) = bridge_config else {
        return Err(
            "live solana settlement requires KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL"
                .to_owned(),
        );
    };
    let keypair_file = require_present(
        keypair_file,
        LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
    )?;
    let recipient_pubkey = parse_recipient_pubkey(
        require_present(
            recipient_pubkey,
            LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY_ENV,
        )?,
    )?;
    let lamports = parse_lamports(require_present(
        lamports,
        LIVE_SOLANA_SETTLEMENT_LAMPORTS_ENV,
    )?)?;
    let (commitment, commitment_label) = parse_commitment(commitment)?;
    validate_keypair_file(keypair_file.as_str())?;
    Ok(Some(LiveSolanaSettlementConfig {
        rpc_url: bridge_config.rpc_url.clone(),
        keypair_file,
        recipient_pubkey,
        lamports,
        commitment,
        commitment_label,
    }))
}

fn read_required_env(name: &str) -> Result<Option<String>, String> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(normalize_non_empty_env(name, value)?)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err(format!("live solana settlement env must be utf-8: {name}"))
        }
    }
}

fn read_optional_env(name: &str) -> Result<Option<String>, String> {
    read_required_env(name)
}

fn normalize_non_empty_env(name: &str, value: String) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("live solana settlement env must not be empty: {name}"));
    }
    Ok(normalized.to_owned())
}

fn require_present(value: Option<String>, name: &str) -> Result<String, String> {
    value.ok_or_else(|| format!("live solana settlement env missing: {name}"))
}

fn parse_recipient_pubkey(value: String) -> Result<Pubkey, String> {
    Pubkey::from_str(value.as_str()).map_err(|error| {
        format!(
            "live solana settlement recipient pubkey invalid: {LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY_ENV}: {error}"
        )
    })
}

fn parse_lamports(value: String) -> Result<u64, String> {
    let lamports = value.parse::<u64>().map_err(|error| {
        format!(
            "live solana settlement lamports invalid: {LIVE_SOLANA_SETTLEMENT_LAMPORTS_ENV}: {error}"
        )
    })?;
    if lamports == 0 {
        return Err(format!(
            "live solana settlement lamports invalid: {LIVE_SOLANA_SETTLEMENT_LAMPORTS_ENV}: must be greater than zero"
        ));
    }
    Ok(lamports)
}

fn parse_commitment(value: Option<String>) -> Result<(CommitmentConfig, String), String> {
    let label = value.unwrap_or_else(|| "finalized".to_owned());
    let commitment = CommitmentConfig::from_str(label.as_str()).map_err(|_| {
        format!(
            "live solana settlement commitment invalid: {LIVE_SOLANA_SETTLEMENT_COMMITMENT_ENV}: {label}"
        )
    })?;
    Ok((commitment, label))
}

fn validate_keypair_file(path: &str) -> Result<(), String> {
    read_keypair_file(path).map(|_| ()).map_err(|error| {
        format!(
            "live solana settlement keypair file invalid: {LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV}: {path}: {error}"
        )
    })
}
