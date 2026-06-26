use super::models::*;
use super::validation::{validate_kamn_did, validate_non_empty};

impl DataLayerM4EscrowTransitionEngine {
    /// Evaluates requester visibility for escrow-scoped messages.
    pub fn authorize_message_visibility(
        &self,
        request: DataLayerM4EscrowVisibilityRequest,
    ) -> Result<DataLayerM4EscrowVisibilityDecision, DataLayerM4SettlementEvidenceRegistryError>
    {
        validate_non_empty(request.escrow_id.as_str(), "escrow_id")?;
        validate_kamn_did(request.requester_did.as_str())?;
        let escrow = self.escrows.get(request.escrow_id.as_str()).ok_or(
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: request.escrow_id,
            },
        )?;
        if is_participant(escrow, request.requester_did.as_str()) {
            return Ok(DataLayerM4EscrowVisibilityDecision::Allow {
                reason_code: DATA_LAYER_M4_ESCROW_PARTICIPANT_SCOPE_ALLOWED_REASON_CODE,
            });
        }
        if escrow.auditor_did.as_deref() == Some(request.requester_did.as_str()) {
            return Ok(auditor_visibility_decision(
                escrow,
                request.reconstructed_auditor_shares,
            ));
        }
        Ok(DataLayerM4EscrowVisibilityDecision::Deny {
            reason_code: DATA_LAYER_M4_ESCROW_SCOPE_DENIED_REASON_CODE,
        })
    }
}

fn is_participant(escrow: &DataLayerM4EscrowRecord, requester_did: &str) -> bool {
    requester_did == escrow.initiator_did || requester_did == escrow.counterparty_did
}

fn auditor_visibility_decision(
    escrow: &DataLayerM4EscrowRecord,
    reconstructed_auditor_shares: Option<u8>,
) -> DataLayerM4EscrowVisibilityDecision {
    if escrow.state != DataLayerM4EscrowState::Disputed {
        return DataLayerM4EscrowVisibilityDecision::Deny {
            reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_DISPUTE_REQUIRED_REASON_CODE,
        };
    }
    let threshold = escrow.auditor_threshold.unwrap_or(0);
    if threshold == 0 {
        return DataLayerM4EscrowVisibilityDecision::Deny {
            reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_CONFIGURED_REASON_CODE,
        };
    }
    if reconstructed_auditor_shares.unwrap_or(0) >= threshold {
        return DataLayerM4EscrowVisibilityDecision::Allow {
            reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE,
        };
    }
    DataLayerM4EscrowVisibilityDecision::Deny {
        reason_code: DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_MET_REASON_CODE,
    }
}
