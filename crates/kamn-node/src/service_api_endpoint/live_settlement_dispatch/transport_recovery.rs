use super::*;

const FINALITY_POLL_ATTEMPTS: usize = 30;
const FINALITY_POLL_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SignatureReconciliation {
    Finalized,
    Pending,
    Absent,
}

fn settlement_evidence(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
) -> LiveSettlementEvidence {
    build_live_settlement_evidence(
        prepared.expected_signature.clone(),
        config.commitment_label.as_str(),
        prepared,
    )
}

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
) -> Result<SignatureReconciliation, String> {
    let response = client
        .get_signature_statuses_with_history(&[*signature])
        .map_err(|error| format!("live solana settlement status lookup failed: {error}"))?;
    let Some(status) = response.value.first().and_then(Option::as_ref) else {
        return Ok(SignatureReconciliation::Absent);
    };
    if let Some(error) = status.err.as_ref() {
        return Err(format!("SETTLEMENT_TRANSACTION_FAILED: {error}"));
    }
    Ok(if status.satisfies_commitment(config.commitment) {
        SignatureReconciliation::Finalized
    } else {
        SignatureReconciliation::Pending
    })
}

pub(super) fn wait_for_finalized_evidence(
    client: &RpcClient,
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
) -> Result<LiveSettlementEvidence, String> {
    for _ in 0..FINALITY_POLL_ATTEMPTS {
        match settlement_evidence_with_slot(client, config, prepared) {
            Ok(evidence) => return Ok(evidence),
            Err(error)
                if error.contains("transaction is not finalized")
                    || error.contains("transaction status missing") =>
            {
                std::thread::sleep(FINALITY_POLL_INTERVAL);
            }
            Err(error) => return Err(error),
        }
    }
    Err(format!(
        "SETTLEMENT_OUTCOME_AMBIGUOUS: confirmation missing at {}",
        config.commitment_label
    ))
}

pub(super) fn settlement_evidence_with_slot(
    client: &RpcClient,
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
) -> Result<LiveSettlementEvidence, String> {
    let signature = solana_sdk::signature::Signature::from_str(&prepared.expected_signature)
        .map_err(|error| format!("live solana settlement signature decode failed: {error}"))?;
    let response = client
        .get_signature_statuses_with_history(&[signature])
        .map_err(|error| format!("SETTLEMENT_OUTCOME_AMBIGUOUS: status lookup failed: {error}"))?;
    let status = response
        .value
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(|| "SETTLEMENT_OUTCOME_AMBIGUOUS: transaction status missing".to_owned())?;
    if status.err.is_some() || !status.satisfies_commitment(config.commitment) {
        return Err("SETTLEMENT_OUTCOME_AMBIGUOUS: transaction is not finalized".to_owned());
    }
    let mut evidence = settlement_evidence(config, prepared);
    evidence.finalized_slot = Some(status.slot);
    Ok(evidence)
}
