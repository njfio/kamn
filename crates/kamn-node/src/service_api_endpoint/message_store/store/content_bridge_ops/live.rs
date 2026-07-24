use super::super::super::*;
use crate::service_api_endpoint::live_settlement_dispatch::{
    LiveSettlementEvidence, LiveSolanaSettlementConfig, PreparedLiveSettlement,
};
mod receipt;
use receipt::{build_receipt, validate_receipt_replay};

impl ServiceApiMessageStore {
    pub(crate) fn bridge_transaction_subject(
        &mut self,
        bridge_id: &str,
    ) -> Result<Option<String>, String> {
        self.refresh_from_disk()?;
        Ok(self
            .snapshot
            .bridges
            .get(bridge_id)
            .map(transaction_subject))
    }

    pub(crate) fn get_prepared_bridge_transaction(
        &mut self,
        bridge_id: &str,
        config: &LiveSolanaSettlementConfig,
    ) -> Result<Option<PreparedLiveSettlement>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.bridges.get(bridge_id) else {
            return Ok(None);
        };
        Ok(record
            .prepared_transaction
            .as_ref()
            .map(|prepared| prepared_from_record(prepared, config)))
    }

    pub(crate) fn prepare_bridge_transaction(
        &mut self,
        bridge_id: &str,
        prepared: &PreparedLiveSettlement,
        transaction_subject: &str,
    ) -> Result<(), String> {
        self.refresh_from_disk()?;
        validate_signature_reuse(&self.snapshot, bridge_id, prepared)?;
        let record = self
            .snapshot
            .bridges
            .get_mut(bridge_id)
            .ok_or_else(|| "bridge transaction intent missing".to_owned())?;
        persist_prepared(record, prepared, transaction_subject)?;
        self.persist()
    }

    pub(crate) fn mark_bridge_submitted(&mut self, bridge_id: &str) -> Result<(), String> {
        self.refresh_from_disk()?;
        let record = require_bridge_mut(&mut self.snapshot, bridge_id)?;
        record.submission_attempt_count = record.submission_attempt_count.max(1);
        record.last_error_code = None;
        self.persist()
    }

    pub(crate) fn mark_bridge_ambiguous(&mut self, bridge_id: &str) -> Result<(), String> {
        self.refresh_from_disk()?;
        let record = require_bridge_mut(&mut self.snapshot, bridge_id)?;
        record.last_error_code = Some("BRIDGE_RECONCILIATION_REQUIRED".to_owned());
        self.persist()
    }

    pub(crate) fn finalize_bridge_transaction(
        &mut self,
        bridge_id: &str,
        config: &LiveSolanaSettlementConfig,
        evidence: &LiveSettlementEvidence,
    ) -> Result<Option<ServiceApiBridgeStatusBody>, String> {
        self.refresh_from_disk()?;
        let receipt = build_receipt(&self.snapshot, bridge_id, config, evidence)?;
        validate_receipt_replay(&self.snapshot, bridge_id, receipt.receipt_id.as_str())?;
        let record = require_bridge_mut(&mut self.snapshot, bridge_id)?;
        record.bridge_status = "finalized".to_owned();
        record.target_message_id = format!("msg-solana-{}", receipt.transaction_signature);
        record.forward_tx_hash = receipt.transaction_signature.clone();
        record.bridge_receipt = Some(receipt);
        record.last_error_code = None;
        let response = bridge_status_body(record);
        self.persist()?;
        Ok(Some(response))
    }
}

pub(super) fn bridge_status_body(
    record: &ServiceApiPersistedBridgeRecord,
) -> ServiceApiBridgeStatusBody {
    let receipt = record.bridge_receipt.as_ref();
    ServiceApiBridgeStatusBody {
        bridge_id: record.bridge_id.clone(),
        bridge_status: record.bridge_status.clone(),
        target_message_id: record.target_message_id.clone(),
        forward_tx_hash: record.forward_tx_hash.clone(),
        receipt_id: receipt.map(|value| value.receipt_id.clone()),
        receipt_digest: receipt.map(|value| value.receipt_digest.clone()),
        transaction_signature: receipt.map(|value| value.transaction_signature.clone()),
        finalized_slot: receipt.map(|value| value.finalized_slot),
        bridge_receipt: receipt.cloned(),
    }
}

fn transaction_subject(record: &ServiceApiPersistedBridgeRecord) -> String {
    format!(
        "{}:{}:{}:{}",
        record.bridge_id, record.source_message_id, record.target_network, record.payload_hash
    )
}

fn prepared_from_record(
    record: &ServiceApiPreparedBridgeTransactionRecord,
    config: &LiveSolanaSettlementConfig,
) -> PreparedLiveSettlement {
    PreparedLiveSettlement {
        expected_signature: record.transaction_signature.clone(),
        signed_transaction_digest: record.signed_transaction_digest.clone(),
        signed_transaction_json: record.signed_transaction_json.clone(),
        recipient_pubkey: config.recipient_pubkey(),
        amount_lamports: config.lamports(),
        network: "solana:devnet".to_owned(),
    }
}

fn persist_prepared(
    record: &mut ServiceApiPersistedBridgeRecord,
    prepared: &PreparedLiveSettlement,
    transaction_subject: &str,
) -> Result<(), String> {
    let candidate = ServiceApiPreparedBridgeTransactionRecord {
        transaction_signature: prepared.expected_signature.clone(),
        signed_transaction_digest: prepared.signed_transaction_digest.clone(),
        signed_transaction_json: prepared.signed_transaction_json.clone(),
        transaction_subject: transaction_subject.to_owned(),
    };
    if record
        .prepared_transaction
        .as_ref()
        .is_some_and(|old| old != &candidate)
    {
        return Err("BRIDGE_RECEIPT_REPLAY".to_owned());
    }
    record.prepared_transaction = Some(candidate);
    Ok(())
}

fn validate_signature_reuse(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    bridge_id: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<(), String> {
    let reused = snapshot.bridges.values().any(|record| {
        record.bridge_id != bridge_id
            && record
                .prepared_transaction
                .as_ref()
                .is_some_and(|value| value.transaction_signature == prepared.expected_signature)
    });
    if reused {
        Err("BRIDGE_RECEIPT_REPLAY".to_owned())
    } else {
        Ok(())
    }
}

fn require_bridge_mut<'a>(
    snapshot: &'a mut ServiceApiPersistedMessageStoreSnapshot,
    bridge_id: &str,
) -> Result<&'a mut ServiceApiPersistedBridgeRecord, String> {
    snapshot
        .bridges
        .get_mut(bridge_id)
        .ok_or_else(|| "bridge transaction intent missing".to_owned())
}
