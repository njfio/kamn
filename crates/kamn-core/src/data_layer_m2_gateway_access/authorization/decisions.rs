use super::engine::DataLayerM2AbacEngine;
use super::models::{DataLayerM2AuthorizationDecision, DataLayerM2MessageScopeValidated};
use crate::data_layer_m2_gateway_access::{
    DataLayerM2GatewayError, DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
    DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
    DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED, DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
};

pub(super) fn validate_escrow_id(escrow_id: &str) -> Result<(), DataLayerM2GatewayError> {
    if escrow_id.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField("escrow_id"));
    }
    Ok(())
}

pub(super) fn agent_decision(
    requester: &str,
    scope: &DataLayerM2MessageScopeValidated,
) -> DataLayerM2AuthorizationDecision {
    if requester == scope.sender_did.as_str() || requester == scope.recipient_did.as_str() {
        return DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
        };
    }
    deny_decision()
}

pub(super) fn owner_decision(
    requester: &str,
    scope: &DataLayerM2MessageScopeValidated,
) -> DataLayerM2AuthorizationDecision {
    if requester == scope.owner_sender_did.as_str()
        || requester == scope.owner_recipient_did.as_str()
    {
        return DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
        };
    }
    deny_decision()
}

pub(super) fn auditor_decision(
    engine: &DataLayerM2AbacEngine,
    requester: &str,
    scope: &DataLayerM2MessageScopeValidated,
) -> DataLayerM2AuthorizationDecision {
    let escrow_id = scope.escrow_id.as_deref().unwrap_or_default();
    if escrow_id.is_empty() {
        return deny_decision();
    }
    let auditor_allowed = engine
        .escrow_auditors_by_escrow
        .get(escrow_id)
        .is_some_and(|auditors| auditors.contains(requester));
    if auditor_allowed && engine.disputed_escrows.contains(escrow_id) {
        return DataLayerM2AuthorizationDecision::Allow {
            reason_code: DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED,
        };
    }
    deny_decision()
}

pub(super) fn deny_decision() -> DataLayerM2AuthorizationDecision {
    DataLayerM2AuthorizationDecision::Deny {
        reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
    }
}
