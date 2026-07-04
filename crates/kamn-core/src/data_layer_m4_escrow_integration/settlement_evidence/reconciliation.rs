use super::super::models::*;
use super::super::validation::validate_non_empty;

impl DataLayerM4SettlementEvidenceRegistry {
    /// Reconciles terminal escrow projection with the latest settlement evidence row.
    pub fn reconcile_against_escrow(
        &self,
        escrow: &DataLayerM4EscrowRecord,
    ) -> Result<
        DataLayerM4SettlementEvidenceReconciliationReport,
        DataLayerM4SettlementEvidenceRegistryError,
    > {
        validate_terminal_escrow(escrow)?;
        let escrow_settlement_receipt_hash =
            escrow.settlement_receipt_hash.as_ref().cloned().ok_or(
                DataLayerM4SettlementEvidenceRegistryError::EmptyField(
                    "escrow_settlement_receipt_hash",
                ),
            )?;
        if let Some(latest) = latest_record(self, escrow.escrow_id.as_str())? {
            return Ok(build_reconciliation_report(
                escrow,
                escrow_settlement_receipt_hash,
                latest,
            ));
        }
        Ok(missing_reconciliation_report(
            escrow,
            escrow_settlement_receipt_hash,
        ))
    }
}

fn validate_terminal_escrow(
    escrow: &DataLayerM4EscrowRecord,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    validate_non_empty(escrow.escrow_id.as_str(), "escrow_id")?;
    if escrow.state != DataLayerM4EscrowState::Released
        && escrow.state != DataLayerM4EscrowState::Refunded
    {
        return Err(
            DataLayerM4SettlementEvidenceRegistryError::UnsupportedSettlementState(escrow.state),
        );
    }
    Ok(())
}

fn latest_record<'a>(
    registry: &'a DataLayerM4SettlementEvidenceRegistry,
    escrow_id: &str,
) -> Result<
    Option<&'a DataLayerM4SettlementEvidenceRecord>,
    DataLayerM4SettlementEvidenceRegistryError,
> {
    if !registry.records_by_escrow.contains_key(escrow_id) {
        return Ok(None);
    }
    registry.verify_escrow_integrity(escrow_id)?;
    Ok(registry
        .records_by_escrow
        .get(escrow_id)
        .and_then(|records| records.last()))
}

fn build_reconciliation_report(
    escrow: &DataLayerM4EscrowRecord,
    escrow_settlement_receipt_hash: String,
    latest: &DataLayerM4SettlementEvidenceRecord,
) -> DataLayerM4SettlementEvidenceReconciliationReport {
    let (decision, reason_code) =
        reconciliation_decision(escrow, escrow_settlement_receipt_hash.as_str(), latest);
    DataLayerM4SettlementEvidenceReconciliationReport {
        escrow_id: escrow.escrow_id.clone(),
        decision,
        reason_code,
        escrow_state: escrow.state,
        escrow_settlement_receipt_hash,
        evidence_sequence: Some(latest.sequence),
        evidence_state: Some(latest.escrow_state),
        evidence_settlement_receipt_hash: Some(latest.settlement_receipt_hash.clone()),
        evidence_settlement_payload_hash: Some(latest.settlement_payload_hash.clone()),
    }
}

fn reconciliation_decision(
    escrow: &DataLayerM4EscrowRecord,
    receipt_hash: &str,
    latest: &DataLayerM4SettlementEvidenceRecord,
) -> (
    DataLayerM4SettlementEvidenceReconciliationDecision,
    &'static str,
) {
    if latest.escrow_state != escrow.state || latest.settlement_receipt_hash != receipt_hash {
        return (
            DataLayerM4SettlementEvidenceReconciliationDecision::Mismatch,
            DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MISMATCH_REASON_CODE,
        );
    }
    (
        DataLayerM4SettlementEvidenceReconciliationDecision::Match,
        DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MATCH_REASON_CODE,
    )
}

fn missing_reconciliation_report(
    escrow: &DataLayerM4EscrowRecord,
    escrow_settlement_receipt_hash: String,
) -> DataLayerM4SettlementEvidenceReconciliationReport {
    DataLayerM4SettlementEvidenceReconciliationReport {
        escrow_id: escrow.escrow_id.clone(),
        decision: DataLayerM4SettlementEvidenceReconciliationDecision::Mismatch,
        reason_code: DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MISMATCH_REASON_CODE,
        escrow_state: escrow.state,
        escrow_settlement_receipt_hash,
        evidence_sequence: None,
        evidence_state: None,
        evidence_settlement_receipt_hash: None,
        evidence_settlement_payload_hash: None,
    }
}
