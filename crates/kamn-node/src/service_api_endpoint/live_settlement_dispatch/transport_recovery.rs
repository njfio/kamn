use super::*;

pub(super) fn blockhash_is_valid(
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

pub(super) fn require_resubmittable_blockhash(valid: bool) -> Result<(), String> {
    if valid {
        return Ok(());
    }
    Err("SETTLEMENT_TRANSACTION_EXPIRED".to_owned())
}

pub(super) fn reconcile_known_signature(
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
