use super::super::models::*;
use super::super::validation::{validate_auditor_threshold, validate_kamn_did, validate_non_empty};

impl DataLayerM4EscrowTransitionEngine {
    /// Creates an empty escrow transition engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates and stores one escrow in `Created` state.
    pub fn create_escrow(
        &mut self,
        input: DataLayerM4EscrowDraftInput,
    ) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
        validate_non_empty(input.escrow_id.as_str(), "escrow_id")?;
        validate_kamn_did(input.initiator_did.as_str())?;
        validate_kamn_did(input.counterparty_did.as_str())?;
        ensure_distinct_parties(
            input.initiator_did.as_str(),
            input.counterparty_did.as_str(),
        )?;
        validate_optional_auditor(input.auditor_did.as_deref())?;
        validate_share_holders(&input.auditor_share_holders)?;
        validate_auditor_threshold(
            input.auditor_did.as_ref(),
            input.auditor_threshold,
            input.auditor_share_holders.len(),
        )?;
        validate_expiration(input.expires_at_epoch_seconds)?;
        ensure_unique_escrow_id(self, input.escrow_id.as_str())?;
        let escrow = build_created_escrow(input);
        self.escrows.insert(escrow.escrow_id.clone(), escrow);
        Ok(())
    }

    /// Returns one stored escrow record by identifier.
    pub fn escrow(&self, escrow_id: &str) -> Option<&DataLayerM4EscrowRecord> {
        self.escrows.get(escrow_id)
    }
}

fn ensure_distinct_parties(
    initiator_did: &str,
    counterparty_did: &str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if initiator_did == counterparty_did {
        return Err(
            DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowParties(
                "initiator and counterparty must be distinct",
            ),
        );
    }
    Ok(())
}

fn validate_optional_auditor(
    auditor_did: Option<&str>,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if let Some(auditor_did) = auditor_did {
        validate_kamn_did(auditor_did)?;
    }
    Ok(())
}

fn validate_share_holders(
    holders: &[String],
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    for holder in holders {
        validate_kamn_did(holder.as_str())?;
    }
    Ok(())
}

fn validate_expiration(
    expires_at_epoch_seconds: Option<u64>,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if expires_at_epoch_seconds == Some(0) {
        return Err(DataLayerM4SettlementEvidenceRegistryError::EmptyField(
            "expires_at_epoch_seconds",
        ));
    }
    Ok(())
}

fn ensure_unique_escrow_id(
    engine: &DataLayerM4EscrowTransitionEngine,
    escrow_id: &str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if engine.escrows.contains_key(escrow_id) {
        return Err(
            DataLayerM4SettlementEvidenceRegistryError::DuplicateEscrowId(escrow_id.to_owned()),
        );
    }
    Ok(())
}

fn build_created_escrow(input: DataLayerM4EscrowDraftInput) -> DataLayerM4EscrowRecord {
    DataLayerM4EscrowRecord {
        escrow_id: input.escrow_id,
        initiator_did: input.initiator_did,
        counterparty_did: input.counterparty_did,
        auditor_did: input.auditor_did,
        auditor_threshold: input.auditor_threshold,
        auditor_share_holders: input.auditor_share_holders,
        state: DataLayerM4EscrowState::Created,
        expires_at_epoch_seconds: input.expires_at_epoch_seconds,
        dispute_opened_at_epoch_seconds: None,
        settled_at_epoch_seconds: None,
        settlement_receipt_hash: None,
    }
}
