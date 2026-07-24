use super::*;

pub(super) fn build_receipt(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    bridge_id: &str,
    config: &LiveSolanaSettlementConfig,
    evidence: &LiveSettlementEvidence,
) -> Result<ServiceApiBridgeReceiptRecord, String> {
    let record = snapshot
        .bridges
        .get(bridge_id)
        .ok_or_else(|| "bridge transaction intent missing".to_owned())?;
    validate_evidence(record, config, evidence)?;
    let mut receipt = receipt_fields(record, bridge_id, evidence);
    receipt.receipt_digest = super::super::super::super::authority_digest::bridge(&receipt);
    Ok(receipt)
}

pub(super) fn validate_receipt_replay(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    bridge_id: &str,
    receipt_id: &str,
) -> Result<(), String> {
    let replayed = snapshot.bridges.values().any(|record| {
        record.bridge_id != bridge_id
            && record
                .bridge_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.receipt_id == receipt_id)
    });
    if replayed {
        Err("BRIDGE_RECEIPT_REPLAY".to_owned())
    } else {
        Ok(())
    }
}

fn validate_evidence(
    record: &ServiceApiPersistedBridgeRecord,
    config: &LiveSolanaSettlementConfig,
    evidence: &LiveSettlementEvidence,
) -> Result<(), String> {
    let prepared = record
        .prepared_transaction
        .as_ref()
        .ok_or_else(|| "bridge prepared transaction missing".to_owned())?;
    let valid = evidence.settlement_tx_signature == prepared.transaction_signature
        && evidence.settlement_receipt_hash == prepared.transaction_signature
        && evidence.settlement_network == "solana:devnet"
        && evidence.settlement_commitment == "finalized"
        && config.commitment_label() == "finalized"
        && evidence.recipient_pubkey == Some(config.recipient_pubkey())
        && evidence.amount_lamports == Some(config.lamports())
        && evidence.finalized_slot.is_some_and(|slot| slot > 0)
        && prepared.transaction_subject == transaction_subject(record);
    if valid {
        Ok(())
    } else {
        Err("BRIDGE_FINALITY_EVIDENCE_INVALID".to_owned())
    }
}

fn receipt_fields(
    record: &ServiceApiPersistedBridgeRecord,
    bridge_id: &str,
    evidence: &LiveSettlementEvidence,
) -> ServiceApiBridgeReceiptRecord {
    ServiceApiBridgeReceiptRecord {
        receipt_id: format!("bridge-receipt-{bridge_id}"),
        receipt_digest: String::new(),
        bridge_id: bridge_id.to_owned(),
        source_message_id: record.source_message_id.clone(),
        target_network: record.target_network.clone(),
        payload_hash: record.payload_hash.clone(),
        transaction_signature: evidence.settlement_tx_signature.clone(),
        network: evidence.settlement_network.clone(),
        commitment: evidence.settlement_commitment.clone(),
        finalized_slot: evidence.finalized_slot.unwrap_or_default(),
        action: "bridge:finalize".to_owned(),
        resource_id: bridge_id.to_owned(),
        state: "finalized".to_owned(),
    }
}
