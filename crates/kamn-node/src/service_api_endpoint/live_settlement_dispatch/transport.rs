use super::models::{
    build_live_settlement_evidence, LiveSettlementEvidence, PreparedLiveSettlement,
};
use super::LiveSolanaSettlementConfig;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::read_keypair_file;
use std::str::FromStr;
use std::time::Duration;

mod transaction;
use transaction::{build_live_settlement_transaction, validate_prepared_transaction};

pub(super) fn prepare_live_settlement(
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<PreparedLiveSettlement, String> {
    #[cfg(test)]
    if let Some(result) = super::test_support::maybe_prepare_test_live_settlement(config, escrow_id)
    {
        return result;
    }
    prepare_live_settlement_via_rpc(config, escrow_id)
}

pub(super) fn submit_or_reconcile_live_settlement(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    #[cfg(test)]
    if let Some(result) =
        super::test_support::maybe_submit_test_live_settlement(config, prepared, escrow_id)
    {
        return result;
    }
    submit_or_reconcile_live_settlement_via_rpc(config, prepared, escrow_id)
}

fn prepare_live_settlement_via_rpc(
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<PreparedLiveSettlement, String> {
    let keypair = read_settlement_keypair(config)?;
    let client = settlement_rpc_client(config);
    let transaction = build_live_settlement_transaction(&client, config, &keypair, escrow_id)?;
    let signature = transaction
        .signatures
        .first()
        .ok_or_else(|| "live solana settlement transaction signature missing".to_owned())?;
    let json = serde_json::to_string(&transaction)
        .map_err(|error| format!("live solana settlement transaction encode failed: {error}"))?;
    Ok(PreparedLiveSettlement {
        expected_signature: signature.to_string(),
        signed_transaction_digest: format!(
            "sha256:{:016x}",
            crate::service_api_endpoint::deterministic_body_tag(json.as_bytes())
        ),
        signed_transaction_json: json,
        recipient_pubkey: config.recipient_pubkey.to_string(),
        amount_lamports: config.lamports,
        network: "solana:devnet".to_owned(),
    })
}

fn submit_or_reconcile_live_settlement_via_rpc(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    validate_prepared_config(config, prepared)?;
    let transaction: solana_sdk::transaction::Transaction =
        serde_json::from_str(prepared.signed_transaction_json.as_str()).map_err(|error| {
            format!("live solana settlement transaction decode failed: {error}")
        })?;
    validate_prepared_transaction(&transaction, prepared, escrow_id)?;
    let client = settlement_rpc_client(config);
    let expected_signature =
        solana_sdk::signature::Signature::from_str(prepared.expected_signature.as_str())
            .map_err(|error| format!("live solana settlement signature decode failed: {error}"))?;
    if reconcile_known_signature(&client, &expected_signature, config)? {
        return Ok(build_live_settlement_evidence(
            prepared.expected_signature.clone(),
            config.commitment_label.as_str(),
            prepared,
        ));
    }
    require_resubmittable_blockhash(blockhash_is_valid(&client, &transaction, config)?)?;
    let signature = submit_live_settlement_transaction(&client, &transaction)?;
    if signature.to_string() != prepared.expected_signature {
        return Err("live solana settlement submitted signature mismatch".to_owned());
    }
    let confirmed = confirm_live_settlement_signature(&client, &signature, config)?;
    require_confirmation(confirmed, config.commitment_label.as_str())?;
    Ok(build_live_settlement_evidence(
        signature.to_string(),
        config.commitment_label.as_str(),
        prepared,
    ))
}

fn blockhash_is_valid(
    client: &RpcClient,
    transaction: &solana_sdk::transaction::Transaction,
    config: &LiveSolanaSettlementConfig,
) -> Result<bool, String> {
    client
        .is_blockhash_valid(&transaction.message.recent_blockhash, config.commitment)
        .map_err(|error| {
            format!("SETTLEMENT_OUTCOME_AMBIGUOUS: blockhash validity lookup failed: {error}")
        })
}

fn require_resubmittable_blockhash(valid: bool) -> Result<(), String> {
    if valid {
        return Ok(());
    }
    Err("SETTLEMENT_TRANSACTION_EXPIRED".to_owned())
}

fn require_confirmation(confirmed: bool, commitment: &str) -> Result<(), String> {
    if confirmed {
        return Ok(());
    }
    Err(format!(
        "SETTLEMENT_OUTCOME_AMBIGUOUS: confirmation missing at {commitment}"
    ))
}

fn reconcile_known_signature(
    client: &RpcClient,
    signature: &solana_sdk::signature::Signature,
    config: &LiveSolanaSettlementConfig,
) -> Result<bool, String> {
    let status = client
        .get_signature_status_with_commitment_and_history(signature, config.commitment, true)
        .map_err(|error| format!("live solana settlement status lookup failed: {error}"))?;
    match status {
        Some(Ok(())) => Ok(true),
        Some(Err(error)) => Err(format!("SETTLEMENT_TRANSACTION_FAILED: {error}")),
        None => Ok(false),
    }
}

fn validate_prepared_config(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
) -> Result<(), String> {
    if prepared.recipient_pubkey != config.recipient_pubkey.to_string()
        || prepared.amount_lamports != config.lamports
        || prepared.network != "solana:devnet"
    {
        return Err("SETTLEMENT_AGREEMENT_MISMATCH".to_owned());
    }
    Ok(())
}

fn read_settlement_keypair(
    config: &LiveSolanaSettlementConfig,
) -> Result<solana_sdk::signer::keypair::Keypair, String> {
    read_keypair_file(config.keypair_file.as_str()).map_err(|error| {
        format!(
            "live solana settlement keypair file read failed: {}: {error}",
            config.keypair_file
        )
    })
}

fn settlement_rpc_client(config: &LiveSolanaSettlementConfig) -> RpcClient {
    RpcClient::new_with_timeout_and_commitment(
        config.rpc_url.clone(),
        Duration::from_secs(30),
        config.commitment,
    )
}

fn submit_live_settlement_transaction(
    client: &RpcClient,
    transaction: &solana_sdk::transaction::Transaction,
) -> Result<solana_sdk::signature::Signature, String> {
    client
        .send_transaction(transaction)
        .map_err(|error| format!("SETTLEMENT_OUTCOME_AMBIGUOUS: submit failed: {error}"))
}

fn confirm_live_settlement_signature(
    client: &RpcClient,
    signature: &solana_sdk::signature::Signature,
    config: &LiveSolanaSettlementConfig,
) -> Result<bool, String> {
    client
        .confirm_transaction_with_commitment(signature, config.commitment)
        .map(|response| response.value)
        .map_err(|error| {
            format!("SETTLEMENT_OUTCOME_AMBIGUOUS: confirmation lookup failed: {error}")
        })
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
