use super::models::*;
use crate::data_layer_hashing::tagged_sha256;

pub(super) fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if value.trim().is_empty() {
        return Err(DataLayerM4SettlementEvidenceRegistryError::EmptyField(field_name));
    }
    Ok(())
}

pub(super) fn validate_non_zero_timestamp(
    value: u64,
    field_name: &'static str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if value == 0 {
        return Err(DataLayerM4SettlementEvidenceRegistryError::EmptyField(field_name));
    }
    Ok(())
}

pub(super) fn validate_kamn_did(
    value: &str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

pub(super) fn validate_hash_token(
    hash: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    let trimmed = hash.trim();
    if trimmed.is_empty() || !trimmed.starts_with("sha256:") {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidHashToken(field_name));
    }
    Ok(())
}

pub(super) fn validate_auditor_threshold(
    auditor_did: Option<&String>,
    threshold: Option<u8>,
    share_holder_count: usize,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if let Some(threshold) = threshold {
        if threshold == 0 || auditor_did.is_none() || share_holder_count < threshold as usize {
            return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidAuditorThreshold {
                threshold,
                share_holder_count,
            });
        }
    } else if auditor_did.is_some() && share_holder_count > 0 {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidAuditorThreshold {
            threshold: 0,
            share_holder_count,
        });
    }
    Ok(())
}

pub(super) fn ensure_transition_allowed(
    escrow_id: &str,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
    allowed_states: &[DataLayerM4EscrowState],
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if !allowed_states.contains(&from) {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowTransition {
            escrow_id: escrow_id.to_owned(),
            from,
            action: action.marker(),
        });
    }
    Ok(())
}

pub(super) fn reason_code_for_transition(
    action: &DataLayerM4EscrowTransitionAction,
) -> &'static str {
    match action {
        DataLayerM4EscrowTransitionAction::Fund { .. } => DATA_LAYER_M4_ESCROW_FUNDED_REASON_CODE,
        DataLayerM4EscrowTransitionAction::Activate { .. } => {
            DATA_LAYER_M4_ESCROW_ACTIVE_REASON_CODE
        }
        DataLayerM4EscrowTransitionAction::OpenDispute { .. } => {
            DATA_LAYER_M4_ESCROW_DISPUTED_REASON_CODE
        }
        DataLayerM4EscrowTransitionAction::ResolveRelease { .. } => {
            DATA_LAYER_M4_ESCROW_RELEASED_REASON_CODE
        }
        DataLayerM4EscrowTransitionAction::ResolveRefund { .. } => {
            DATA_LAYER_M4_ESCROW_REFUNDED_REASON_CODE
        }
        DataLayerM4EscrowTransitionAction::Expire { .. } => DATA_LAYER_M4_ESCROW_EXPIRED_REASON_CODE,
    }
}

pub(super) fn compute_evidence_hash(
    sequence: u64,
    input: &DataLayerM4SettlementEvidenceInput,
    hash_chain_prev: &str,
) -> String {
    tagged_digest(
        format!(
            "m4-evidence|escrow:{}|seq:{sequence}|state:{}|receipt:{}|payload:{}|recorded:{}|prev:{}",
            input.escrow_id,
            input.escrow_state.as_marker(),
            input.settlement_receipt_hash,
            input.settlement_payload_hash,
            input.recorded_at_epoch_seconds,
            hash_chain_prev
        )
        .as_str(),
    )
}

fn tagged_digest(value: &str) -> String {
    tagged_sha256(value, DATA_LAYER_M4_HASH_ALGORITHM)
}
