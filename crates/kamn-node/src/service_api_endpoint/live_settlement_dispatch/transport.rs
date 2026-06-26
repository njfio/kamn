use super::models::LiveSettlementEvidence;
use super::LiveSolanaSettlementConfig;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::read_keypair_file;
use solana_system_transaction as system_transaction;
use std::time::Duration;

pub(super) fn collect_live_settlement_evidence(
    config: &LiveSolanaSettlementConfig,
    _escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    #[cfg(test)]
    if let Some(result) =
        super::test_support::maybe_collect_test_live_settlement_evidence(config, _escrow_id)
    {
        return result;
    }
    collect_live_settlement_evidence_via_rpc(config)
}

fn collect_live_settlement_evidence_via_rpc(
    config: &LiveSolanaSettlementConfig,
) -> Result<LiveSettlementEvidence, String> {
    let keypair = read_settlement_keypair(config)?;
    let client = settlement_rpc_client(config);
    let transaction = build_live_settlement_transaction(&client, config, &keypair)?;
    let signature = submit_live_settlement_transaction(&client, &transaction)?;
    let confirmed = confirm_live_settlement_signature(&client, &signature, config)?;
    if !confirmed {
        return Err(format!(
            "live solana settlement confirmation missing at {}",
            config.commitment_label
        ));
    }
    Ok(build_live_settlement_evidence(
        signature.to_string(),
        config.commitment_label.as_str(),
    ))
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

fn build_live_settlement_transaction(
    client: &RpcClient,
    config: &LiveSolanaSettlementConfig,
    keypair: &solana_sdk::signer::keypair::Keypair,
) -> Result<solana_sdk::transaction::Transaction, String> {
    let latest_blockhash = client.get_latest_blockhash().map_err(|error| {
        format!("live solana settlement latest blockhash lookup failed: {error}")
    })?;
    Ok(system_transaction::transfer(
        keypair,
        &config.recipient_pubkey,
        config.lamports,
        latest_blockhash,
    ))
}

fn submit_live_settlement_transaction(
    client: &RpcClient,
    transaction: &solana_sdk::transaction::Transaction,
) -> Result<solana_sdk::signature::Signature, String> {
    client
        .send_and_confirm_transaction(transaction)
        .map_err(|error| format!("live solana settlement transaction submit failed: {error}"))
}

fn confirm_live_settlement_signature(
    client: &RpcClient,
    signature: &solana_sdk::signature::Signature,
    config: &LiveSolanaSettlementConfig,
) -> Result<bool, String> {
    client
        .confirm_transaction_with_commitment(signature, config.commitment)
        .map(|response| response.value)
        .map_err(|error| format!("live solana settlement confirmation lookup failed: {error}"))
}

fn build_live_settlement_evidence(
    settlement_tx_signature: String,
    settlement_commitment: &str,
) -> LiveSettlementEvidence {
    LiveSettlementEvidence {
        settlement_receipt_hash: settlement_tx_signature.clone(),
        settlement_tx_signature,
        settlement_network: "solana:devnet".to_owned(),
        settlement_commitment: settlement_commitment.to_owned(),
    }
}
