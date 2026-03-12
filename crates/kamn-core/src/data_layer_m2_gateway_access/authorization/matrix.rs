use super::models::{
    DataLayerM2AuthorizationDecision, DataLayerM2NegativeAuthorizationAuditFixture,
    DataLayerM2NegativeAuthorizationCase, DataLayerM2NegativeAuthorizationMatrixDecision,
};
use crate::data_layer_m2_gateway_access::audit::{
    DataLayerM2AccessAuditInput, DataLayerM2AccessAuditLedger,
};
use crate::data_layer_m2_gateway_access::{
    DataLayerM2GatewayError, DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
    DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
};

pub(super) fn validate_cases(
    cases: &[DataLayerM2NegativeAuthorizationCase],
) -> Result<(), DataLayerM2GatewayError> {
    if cases.is_empty() {
        return Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
            "cases",
        ));
    }
    Ok(())
}

pub(super) fn validate_case(
    case: &DataLayerM2NegativeAuthorizationCase,
) -> Result<(), DataLayerM2GatewayError> {
    if case.case_id.trim().is_empty() {
        return Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
            "case_id",
        ));
    }
    if case.event_epoch_seconds == 0 {
        return Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
            "event_epoch_seconds",
        ));
    }
    Ok(())
}

pub(super) fn build_fixture(
    case: &DataLayerM2NegativeAuthorizationCase,
    decision: DataLayerM2AuthorizationDecision,
    audit_ledger: &mut DataLayerM2AccessAuditLedger,
) -> Result<DataLayerM2NegativeAuthorizationAuditFixture, DataLayerM2GatewayError> {
    let (denied, decision_reason_code) = decision_outcome(decision);
    let audit_record = audit_ledger.append(DataLayerM2AccessAuditInput {
        requester_did: case.requester_did.clone(),
        action: format!("m2_negative_matrix:{}", case.case_id),
        resource_id: case.scope.message_id.clone(),
        reason_code: decision_reason_code.to_owned(),
        event_epoch_seconds: case.event_epoch_seconds,
    })?;
    Ok(DataLayerM2NegativeAuthorizationAuditFixture {
        case_id: case.case_id.clone(),
        denied,
        expected_denied: case.expected_denied,
        mismatch: case.expected_denied != denied,
        decision_reason_code,
        audit_record,
    })
}

fn decision_outcome(decision: DataLayerM2AuthorizationDecision) -> (bool, &'static str) {
    match decision {
        DataLayerM2AuthorizationDecision::Allow { reason_code } => (false, reason_code),
        DataLayerM2AuthorizationDecision::Deny { reason_code } => (true, reason_code),
    }
}

pub(super) fn matrix_decision(
    fixtures: &[DataLayerM2NegativeAuthorizationAuditFixture],
) -> DataLayerM2NegativeAuthorizationMatrixDecision {
    if fixtures.iter().all(|fixture| fixture.denied && !fixture.mismatch) {
        return DataLayerM2NegativeAuthorizationMatrixDecision::AllDenied {
            reason_code: DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
        };
    }
    DataLayerM2NegativeAuthorizationMatrixDecision::DriftDetected {
        reason_code: DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
    }
}
