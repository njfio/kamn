use super::*;
use std::path::Path;

mod legacy_slot;
pub(crate) use legacy_slot::{
    collect_live_bridge_forward_evidence, collect_live_solana_finalized_slot,
};

const LIVE_SOLANA_BRIDGE_RPC_URL_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL";
const LIVE_SOLANA_PROOF_SCHEMA_VERSION: &str = "kamn.solana.devnet.live-proof-report.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LiveSolanaBridgeDispatchConfig {
    pub(super) rpc_url: String,
}

// Live bridge finality is receipt-authoritative: transaction_signature,
// receipt_digest, and finalized_slot are persisted after reconcile.

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
    validate_live_solana_proof_script_path(legacy_slot::proof_script_path().as_path())?;
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

pub(super) fn resolve_prepared_bridge_transaction(
    store: &mut ServiceApiMessageStore,
    config: &super::live_settlement_dispatch::LiveSolanaSettlementConfig,
    bridge_id: &str,
) -> Result<
    Option<(
        super::live_settlement_dispatch::PreparedLiveSettlement,
        String,
    )>,
    String,
> {
    let Some(transaction_subject) = store.bridge_transaction_subject(bridge_id)? else {
        return Ok(None);
    };
    if let Some(prepared) = store.get_prepared_bridge_transaction(bridge_id, config)? {
        return Ok(Some((prepared, transaction_subject)));
    }
    let prepared = super::live_settlement_dispatch::prepare_live_settlement(
        config,
        transaction_subject.as_str(),
    )?;
    store.prepare_bridge_transaction(bridge_id, &prepared, transaction_subject.as_str())?;
    Ok(Some((prepared, transaction_subject)))
}

pub(super) fn submit_or_reconcile_bridge_transaction(
    store: &mut ServiceApiMessageStore,
    config: &super::live_settlement_dispatch::LiveSolanaSettlementConfig,
    prepared: &super::live_settlement_dispatch::PreparedLiveSettlement,
    bridge_id: &str,
    transaction_subject: &str,
) -> Result<super::live_settlement_dispatch::LiveSettlementEvidence, String> {
    let mut before_submit = || store.mark_bridge_submitted(bridge_id);
    let result = super::live_settlement_dispatch::submit_or_reconcile_live_settlement(
        config,
        prepared,
        transaction_subject,
        &mut before_submit,
    );
    if result
        .as_ref()
        .err()
        .is_some_and(|error| error.contains("AMBIGUOUS"))
    {
        store.mark_bridge_ambiguous(bridge_id)?;
        let cause = result.expect_err("ambiguous result should carry an error");
        return Err(format!("BRIDGE_RECONCILIATION_REQUIRED: {cause}"));
    }
    result
}
