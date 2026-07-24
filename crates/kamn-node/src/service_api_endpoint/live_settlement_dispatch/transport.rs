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
#[path = "transport_recovery.rs"]
mod transport_recovery;
use transport_recovery::{
    blockhash_is_valid, reconcile_known_signature, require_resubmittable_blockhash,
    settlement_evidence_with_slot, wait_for_finalized_evidence, SignatureReconciliation,
};

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
    before_submit: &mut dyn FnMut() -> Result<(), String>,
) -> Result<LiveSettlementEvidence, String> {
    #[cfg(test)]
    if let Some(result) = super::test_support::maybe_submit_test_live_settlement(
        config,
        prepared,
        escrow_id,
        before_submit,
    ) {
        return result;
    }
    submit_or_reconcile_live_settlement_via_rpc(config, prepared, escrow_id, before_submit)
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
    before_submit: &mut dyn FnMut() -> Result<(), String>,
) -> Result<LiveSettlementEvidence, String> {
    let (transaction, expected_signature) = validated_transaction(config, prepared, escrow_id)?;
    let client = settlement_rpc_client(config);
    match reconcile_known_signature(&client, &expected_signature, config)? {
        SignatureReconciliation::Finalized => {
            return settlement_evidence_with_slot(&client, config, prepared);
        }
        SignatureReconciliation::Pending => {
            return wait_for_finalized_evidence(&client, config, prepared);
        }
        SignatureReconciliation::Absent => {}
    }
    require_resubmittable_blockhash(blockhash_is_valid(&client, &transaction, config)?)?;
    before_submit()?;
    let signature = submit_live_settlement_transaction(&client, &transaction)?;
    require_expected_signature(&signature, prepared)?;
    wait_for_finalized_evidence(&client, config, prepared)
}

fn validated_transaction(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
) -> Result<
    (
        solana_sdk::transaction::Transaction,
        solana_sdk::signature::Signature,
    ),
    String,
> {
    validate_prepared_config(config, prepared)?;
    let transaction = serde_json::from_str(prepared.signed_transaction_json.as_str())
        .map_err(|error| format!("live solana settlement transaction decode failed: {error}"))?;
    validate_prepared_transaction(&transaction, prepared, escrow_id)?;
    let signature = solana_sdk::signature::Signature::from_str(&prepared.expected_signature)
        .map_err(|error| format!("live solana settlement signature decode failed: {error}"))?;
    Ok((transaction, signature))
}

fn require_expected_signature(
    signature: &solana_sdk::signature::Signature,
    prepared: &PreparedLiveSettlement,
) -> Result<(), String> {
    if signature.to_string() == prepared.expected_signature {
        return Ok(());
    }
    Err("live solana settlement submitted signature mismatch".to_owned())
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

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
